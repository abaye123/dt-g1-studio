//! Wireless (Bluetooth LE) link to the watch, speaking the same protocol as the
//! WearPro phone app. Decoded from an HCI snoop capture; see
//! dt-g1-research/docs/ble-capture-1.md.
//!
//! Frame (both directions):
//! `magic(0x55 phone / 0xAA watch) flags seq:u16 service:u16 cmd:u16 crc:u16 len:u16 body`
//! where crc is CRC-8 (poly 0x1D reflected, init 0) of the body. The watch does
//! not require the 32-byte token WearPro sends on connect.

use std::sync::OnceLock;
use std::time::Duration;

use btleplug::api::{Central, Manager as _, Peripheral as _, ScanFilter, WriteType};
use btleplug::platform::{Manager, Peripheral};
use futures::StreamExt;
use serde::Serialize;
use tokio::sync::{mpsc, Mutex};
use uuid::Uuid;

/// Vendor characteristic used for both writes and notifications.
const CHAR_UUID: Uuid = Uuid::from_u128(0x26078ae1_dfe6_4657_9427_178458b911a1);
/// Manufacturer id the watch advertises (data = its MAC address).
const MANUFACTURER_ID: u16 = 0x1880;
const ADV_NAME: &str = "GPS WATCH";

pub const SVC_SET: u16 = 0x8001;
pub const SVC_GET: u16 = 0x8002;

pub fn crc8(data: &[u8]) -> u8 {
    let mut c = 0u8;
    for &b in data {
        c ^= b;
        for _ in 0..8 {
            c = if c & 1 != 0 { (c >> 1) ^ 0xB8 } else { c >> 1 };
        }
    }
    c
}

pub fn encode_frame(seq: u16, svc: u16, cmd: u16, body: &[u8]) -> Vec<u8> {
    let mut f = Vec::with_capacity(12 + body.len());
    f.push(0x55);
    f.push(0x00);
    f.extend_from_slice(&seq.to_le_bytes());
    f.extend_from_slice(&svc.to_le_bytes());
    f.extend_from_slice(&cmd.to_le_bytes());
    f.extend_from_slice(&(crc8(body) as u16).to_le_bytes());
    f.extend_from_slice(&(body.len() as u16).to_le_bytes());
    f.extend_from_slice(body);
    f
}

#[derive(Debug, Clone, PartialEq)]
pub struct Reply {
    pub seq: u16,
    pub svc: u16,
    pub cmd: u16,
    pub body: Vec<u8>,
}

pub fn decode_frame(d: &[u8]) -> Option<Reply> {
    if d.len() < 12 || d[0] != 0xAA {
        return None;
    }
    let u = |i: usize| u16::from_le_bytes([d[i], d[i + 1]]);
    let len = u(10) as usize;
    let body = d.get(12..12 + len)?.to_vec();
    Some(Reply { seq: u(2), svc: u(4), cmd: u(6), body })
}

struct Session {
    peripheral: Peripheral,
    ch: btleplug::api::Characteristic,
    rx: mpsc::UnboundedReceiver<Vec<u8>>,
    seq: u16,
    name: String,
    address: String,
}

fn session() -> &'static Mutex<Option<Session>> {
    static S: OnceLock<Mutex<Option<Session>>> = OnceLock::new();
    S.get_or_init(|| Mutex::new(None))
}

impl Session {
    async fn request(&mut self, svc: u16, cmd: u16, body: &[u8]) -> Result<Vec<u8>, String> {
        while self.rx.try_recv().is_ok() {}
        self.seq = self.seq.wrapping_add(1).max(1);
        let seq = self.seq;
        let frame = encode_frame(seq, svc, cmd, body);
        self.peripheral
            .write(&self.ch, &frame, WriteType::WithoutResponse)
            .await
            .map_err(|e| format!("write failed: {e}"))?;
        let deadline = tokio::time::Instant::now() + Duration::from_secs(5);
        loop {
            let left = deadline.saturating_duration_since(tokio::time::Instant::now());
            match tokio::time::timeout(left, self.rx.recv()).await {
                Ok(Some(d)) => {
                    if let Some(r) = decode_frame(&d) {
                        if r.seq == seq && r.svc == svc && r.cmd == cmd {
                            return Ok(r.body);
                        }
                    }
                }
                Ok(None) => return Err("connection closed".into()),
                Err(_) => return Err(format!("no answer from the watch ({svc:04x}/{cmd:04x})")),
            }
        }
    }
}

fn is_watch(name: Option<&str>, mfg: &std::collections::HashMap<u16, Vec<u8>>) -> bool {
    name == Some(ADV_NAME) || mfg.contains_key(&MANUFACTURER_ID)
}

#[derive(Serialize)]
pub struct LinkInfo {
    pub name: String,
    pub address: String,
    pub battery: Option<u8>,
}

/// Scan for the watch and connect. The watch only advertises while it is not
/// connected to the phone.
pub async fn connect() -> Result<LinkInfo, String> {
    let mut guard = session().lock().await;
    if let Some(s) = guard.as_ref() {
        if s.peripheral.is_connected().await.unwrap_or(false) {
            return Ok(LinkInfo { name: s.name.clone(), address: s.address.clone(), battery: None });
        }
    }
    *guard = None;

    let manager = Manager::new().await.map_err(|e| format!("bluetooth: {e}"))?;
    let adapter = manager
        .adapters()
        .await
        .map_err(|e| format!("bluetooth: {e}"))?
        .into_iter()
        .next()
        .ok_or("no Bluetooth adapter found")?;
    adapter.start_scan(ScanFilter::default()).await.map_err(|e| format!("scan: {e}"))?;

    let mut found = None;
    for _ in 0..40 {
        tokio::time::sleep(Duration::from_millis(500)).await;
        for p in adapter.peripherals().await.unwrap_or_default() {
            if let Ok(Some(props)) = p.properties().await {
                if is_watch(props.local_name.as_deref(), &props.manufacturer_data) {
                    let name = props.local_name.clone().unwrap_or_else(|| ADV_NAME.to_string());
                    found = Some((p, name, props.address.to_string()));
                    break;
                }
            }
        }
        if found.is_some() {
            break;
        }
    }
    let _ = adapter.stop_scan().await;
    let (peripheral, name, address) = found.ok_or("watch not found")?;

    peripheral.connect().await.map_err(|e| format!("connect: {e}"))?;
    peripheral.discover_services().await.map_err(|e| format!("services: {e}"))?;
    let ch = peripheral
        .characteristics()
        .into_iter()
        .find(|c| c.uuid == CHAR_UUID)
        .ok_or("this device does not expose the watch protocol")?;
    peripheral.subscribe(&ch).await.map_err(|e| format!("subscribe: {e}"))?;

    let mut stream = peripheral.notifications().await.map_err(|e| format!("notify: {e}"))?;
    let (tx, rx) = mpsc::unbounded_channel();
    tauri::async_runtime::spawn(async move {
        while let Some(n) = stream.next().await {
            if n.uuid == CHAR_UUID && tx.send(n.value).is_err() {
                break;
            }
        }
    });

    let mut s = Session { peripheral, ch, rx, seq: 0x100, name, address };
    let battery = s.request(SVC_GET, 0x0003, &[]).await.ok().and_then(|b| b.first().copied());
    let info = LinkInfo { name: s.name.clone(), address: s.address.clone(), battery };
    *guard = Some(s);
    Ok(info)
}

pub async fn disconnect() -> Result<(), String> {
    let mut guard = session().lock().await;
    if let Some(s) = guard.take() {
        let _ = s.peripheral.disconnect().await;
    }
    Ok(())
}

pub async fn is_connected() -> bool {
    let guard = session().lock().await;
    match guard.as_ref() {
        Some(s) => s.peripheral.is_connected().await.unwrap_or(false),
        None => false,
    }
}

/// Run one request on the open session.
pub async fn request(svc: u16, cmd: u16, body: &[u8]) -> Result<Vec<u8>, String> {
    let mut guard = session().lock().await;
    let s = guard.as_mut().ok_or("not connected")?;
    let r = s.request(svc, cmd, body).await;
    if r.is_err() && !s.peripheral.is_connected().await.unwrap_or(false) {
        *guard = None;
        return Err("the watch disconnected".into());
    }
    r
}

pub async fn battery() -> Result<u8, String> {
    let b = request(SVC_GET, 0x0003, &[]).await?;
    b.first().copied().ok_or_else(|| "empty battery reply".into())
}

/// Body of the set-time command (8001/0001):
/// `yy mm dd hh mm ss 00 00 <utc offset hours> 00 <len> <IANA zone name>`.
pub fn time_body(
    y: i32, mo: u32, d: u32, h: u32, mi: u32, s: u32, offset_hours: i32, zone: &str,
) -> Vec<u8> {
    let mut b = vec![
        (y - 2000) as u8, mo as u8, d as u8, h as u8, mi as u8, s as u8, 0, 0, offset_hours as i8 as u8, 0,
    ];
    let z = zone.as_bytes();
    b.push(z.len() as u8);
    b.extend_from_slice(z);
    b
}

pub async fn sync_time() -> Result<String, String> {
    use chrono::{Datelike, Local, Timelike};
    let now = Local::now();
    let offset = now.offset().local_minus_utc() / 3600;
    let zone = iana_time_zone::get_timezone().unwrap_or_else(|_| "UTC".into());
    let body = time_body(
        now.year(), now.month(), now.day(), now.hour(), now.minute(), now.second(), offset, &zone,
    );
    request(SVC_SET, 0x0001, &body).await?;
    Ok(now.format("%H:%M:%S").to_string())
}

/// Watch UI language (8001/0012), a BCP-47 tag such as "he-IL" or "en-US".
pub async fn set_language(tag: &str) -> Result<(), String> {
    if tag.is_empty() || tag.len() > 16 || !tag.bytes().all(|b| b.is_ascii_alphanumeric() || b == b'-') {
        return Err("invalid language tag".into());
    }
    request(SVC_SET, 0x0012, tag.as_bytes()).await.map(|_| ())
}

/// Daily step goal (8001/0003, u32 LE).
pub async fn set_step_goal(steps: u32) -> Result<(), String> {
    request(SVC_SET, 0x0003, &steps.to_le_bytes()).await.map(|_| ())
}

#[derive(Serialize, Debug, PartialEq)]
pub struct Activity {
    pub steps: u32,
    /// Calories in kcal.
    pub calories: f64,
    /// Distance in meters.
    pub distance: u32,
}

/// Today's totals (8004/000a with body 03): steps, calories (cal), distance (m),
/// each u32 LE.
pub fn parse_activity(b: &[u8]) -> Option<Activity> {
    let u = |i: usize| b.get(i..i + 4).map(|x| u32::from_le_bytes([x[0], x[1], x[2], x[3]]));
    Some(Activity { steps: u(0)?, calories: u(4)? as f64 / 1000.0, distance: u(8)? })
}

pub async fn activity() -> Result<Activity, String> {
    let b = request(0x8004, 0x000a, &[0x03]).await?;
    parse_activity(&b).ok_or_else(|| "unexpected activity reply".into())
}

/// Make the watch ring/vibrate (8003/0000).
pub async fn find_watch() -> Result<(), String> {
    request(0x8003, 0x0000, &[]).await.map(|_| ())
}

/// One of the watch's 5 alarm slots. `days` is a bit mask, bit 0 = Monday ...
/// bit 6 = Sunday; 0 means "once".
#[derive(Serialize, serde::Deserialize, Debug, Clone, PartialEq)]
pub struct Alarm {
    pub hour: u8,
    pub minute: u8,
    pub days: u8,
    pub enabled: bool,
}

pub const ALARM_SLOTS: usize = 5;

/// Alarm table: 5 records of `hour minute days 01 enabled` (the 4th byte is
/// always 1 when WearPro writes).
pub fn parse_alarms(b: &[u8]) -> Vec<Alarm> {
    b.chunks_exact(5)
        .take(ALARM_SLOTS)
        .map(|r| Alarm { hour: r[0], minute: r[1], days: r[2] & 0x7f, enabled: r[4] != 0 })
        .collect()
}

pub fn encode_alarms(alarms: &[Alarm]) -> Result<Vec<u8>, String> {
    let mut out = Vec::with_capacity(ALARM_SLOTS * 5);
    for i in 0..ALARM_SLOTS {
        let a = alarms.get(i).cloned().unwrap_or(Alarm { hour: 0, minute: 0, days: 0, enabled: false });
        if a.hour > 23 || a.minute > 59 {
            return Err("invalid alarm time".into());
        }
        out.extend([a.hour, a.minute, a.days & 0x7f, 1, a.enabled as u8]);
    }
    Ok(out)
}

/// Read the alarms from the settings snapshot (8002/0002, first 25 bytes).
pub async fn get_alarms() -> Result<Vec<Alarm>, String> {
    let b = request(SVC_GET, 0x0002, &[]).await?;
    if b.len() < ALARM_SLOTS * 5 {
        return Err("unexpected settings reply".into());
    }
    Ok(parse_alarms(&b[..ALARM_SLOTS * 5]))
}

pub async fn set_alarms(alarms: &[Alarm]) -> Result<(), String> {
    request(SVC_SET, 0x0002, &encode_alarms(alarms)?).await.map(|_| ())
}

/// Wake the screen when the wrist is raised (8001/000c).
pub async fn set_raise_to_wake(on: bool) -> Result<(), String> {
    let v = on as u8;
    request(SVC_SET, 0x000c, &[0, v, v]).await.map(|_| ())
}

fn check_hm(h: u8, m: u8) -> Result<(), String> {
    if h > 23 || m > 59 { Err("invalid time".into()) } else { Ok(()) }
}

/// Do-not-disturb window (8001/0008): on, start h:m, end h:m.
pub async fn set_dnd(on: bool, sh: u8, sm: u8, eh: u8, em: u8) -> Result<(), String> {
    check_hm(sh, sm)?;
    check_hm(eh, em)?;
    request(SVC_SET, 0x0008, &[on as u8, sh, sm, eh, em]).await.map(|_| ())
}

/// Automatic heart-rate monitoring (8001/000a): on, start h:m, end h:m
/// (00:00-00:00 = all day), interval in minutes.
pub async fn set_heart_monitor(on: bool, sh: u8, sm: u8, eh: u8, em: u8, interval: u8) -> Result<(), String> {
    check_hm(sh, sm)?;
    check_hm(eh, em)?;
    if interval == 0 {
        return Err("invalid interval".into());
    }
    request(SVC_SET, 0x000a, &[on as u8, sh, sm, eh, em, interval]).await.map(|_| ())
}

/// Units (8001/0000): imperial distance, Fahrenheit temperature.
pub async fn set_units(imperial: bool, fahrenheit: bool) -> Result<(), String> {
    request(SVC_SET, 0x0000, &[imperial as u8, fahrenheit as u8]).await.map(|_| ())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn crc_matches_capture() {
        assert_eq!(crc8(b"he-IL"), 0x47);
        assert_eq!(crc8(&[0x88, 0x13, 0, 0]), 0x28);
        assert_eq!(crc8(&[0x03]), 0xac);
        assert_eq!(crc8(&[]), 0);
    }

    #[test]
    fn frame_matches_capture() {
        // 8001/0012 "he-IL" seq 0x048d as sent by WearPro.
        let f = encode_frame(0x048d, 0x8001, 0x0012, b"he-IL");
        assert_eq!(hex(&f), "55008d04018012004700050068652d494c");
        let r = decode_frame(&[0xaa, 0x10, 0x92, 0x04, 0x02, 0x80, 0x03, 0x00, 0x90, 0x00, 0x02, 0x00, 0x64, 0x64]).unwrap();
        assert_eq!(r, Reply { seq: 0x0492, svc: 0x8002, cmd: 0x0003, body: vec![100, 100] });
    }

    #[test]
    fn activity_matches_capture() {
        let b = [0xef, 0x08, 0, 0, 0x57, 0xdf, 0, 0, 0xee, 0x05, 0, 0];
        assert_eq!(parse_activity(&b), Some(Activity { steps: 2287, calories: 57.175, distance: 1518 }));
    }

    #[test]
    fn alarms_roundtrip_capture() {
        let b = [7, 10, 0x5f, 1, 1, 4, 10, 0x10, 1, 1, 8, 20, 0x5f, 1, 1, 0, 0, 0, 1, 0, 0, 0, 0, 1, 0];
        let a = parse_alarms(&b);
        assert_eq!(a.len(), 5);
        assert_eq!(a[1], Alarm { hour: 4, minute: 10, days: 0x10, enabled: true });
        assert!(!a[3].enabled);
        assert_eq!(encode_alarms(&a).unwrap(), b.to_vec());
    }

    #[test]
    fn time_body_matches_capture() {
        let b = time_body(2026, 11, 12, 23, 8, 2, 2, "Asia/Jerusalem");
        assert_eq!(hex(&b), "1a0b0c170802000002000e417369612f4a65727573616c656d");
    }

    fn hex(b: &[u8]) -> String {
        b.iter().map(|x| format!("{x:02x}")).collect()
    }
}
