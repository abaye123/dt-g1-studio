//! The serial AT-command channel to the watch (Charging mode).
//!
//! The watch exposes a USB serial port only in "Charging mode"
//! (`VID_10D6&PID_B00A`), at 2000000 baud, 8N1. Every command is ASCII text
//! followed by the four bytes `00 64 00 61` (copied from the vendor tool;
//! without them the watch only echoes "OK"). This module sends only read or
//! mode-change commands; it never sends a write or erase command.

use std::time::Duration;

use serialport::SerialPortType;

use crate::Error;

/// Actions Semiconductor USB vendor id.
pub const VID: u16 = 0x10D6;
/// Product id in Charging mode (serial).
pub const PID_CHARGING: u16 = 0xB00A;
/// Product id in mass-storage mode.
pub const PID_MASS: u16 = 0xB00B;
/// Product id in ADFU boot-ROM download mode.
pub const PID_ADFU: u16 = 0x10D6;

pub const BAUD: u32 = 2_000_000;

/// The suffix appended to every AT command.
const SUFFIX: [u8; 4] = [0x00, 0x64, 0x00, 0x61];

/// What the watch currently looks like on USB.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum UsbMode {
    /// Charging mode: a serial port is available (we can send AT commands).
    Charging,
    /// Mass storage: the user disk is mounted, no serial port.
    MassStorage,
    /// ADFU boot-ROM download mode.
    Adfu,
    /// No Actions device is connected.
    Absent,
}

/// Find the watch's serial port name (COMx) if it is in Charging mode.
pub fn find_port() -> Option<String> {
    let ports = serialport::available_ports().ok()?;
    for p in ports {
        if let SerialPortType::UsbPort(info) = &p.port_type {
            if info.vid == VID && info.pid == PID_CHARGING {
                return Some(p.port_name);
            }
        }
    }
    None
}

/// Detect the current USB mode by enumerating USB devices by VID/PID.
/// Charging wins over the others (it is the only mode we can talk to), then
/// ADFU, then mass storage. Falls back to the serial port list if enumeration
/// is unavailable.
pub fn detect_mode() -> UsbMode {
    if let Ok(devices) = nusb::list_devices() {
        let (mut charging, mut adfu, mut mass) = (false, false, false);
        for d in devices {
            if d.vendor_id() == VID {
                match d.product_id() {
                    PID_CHARGING => charging = true,
                    PID_ADFU => adfu = true,
                    PID_MASS => mass = true,
                    _ => {}
                }
            }
        }
        if charging {
            return UsbMode::Charging;
        }
        if adfu {
            return UsbMode::Adfu;
        }
        if mass {
            return UsbMode::MassStorage;
        }
        return UsbMode::Absent;
    }
    // Fallback: the serial list only reveals Charging vs not.
    if find_port().is_some() {
        UsbMode::Charging
    } else {
        UsbMode::Absent
    }
}

/// A command that is safe to send: read-only or a reversible mode change.
/// This is an allow-list; nothing outside it can be sent through `send`.
#[derive(Clone, Copy, Debug)]
pub enum SafeCommand {
    /// Connectivity check, returns "OK".
    Ping,
    /// Write the current menu to `/1:/custom/menu_list.cfg`.
    SaveMenu,
    /// Select the real disks (step 1 of exposing the hidden partition).
    RealUpan,
    /// Start USB mass storage (step 2; the serial port then disappears).
    Mass,
    /// Enter ADFU download mode (advanced; reversible by a power-cycle).
    Adfu,
}

impl SafeCommand {
    fn text(self) -> &'static str {
        match self {
            SafeCommand::Ping => "AT+WRX=always",
            SafeCommand::SaveMenu => "AT+WRX=saveMenu",
            SafeCommand::RealUpan => "AT+WRX=realUpan",
            SafeCommand::Mass => "AT+WRX=mass",
            SafeCommand::Adfu => "AT+WRX=adfu",
        }
    }

    /// True if sending this drops the serial port (so no reply can be read).
    fn drops_port(self) -> bool {
        matches!(self, SafeCommand::Mass | SafeCommand::Adfu)
    }
}

/// Open the watch serial port.
fn open(port: &str) -> Result<Box<dyn serialport::SerialPort>, Error> {
    serialport::new(port, BAUD)
        .data_bits(serialport::DataBits::Eight)
        .parity(serialport::Parity::None)
        .stop_bits(serialport::StopBits::One)
        .timeout(Duration::from_millis(500))
        .open()
        .map_err(|e| Error::Serial(e.to_string()))
}

/// Send one safe command and return the text reply (empty if the command drops
/// the port, e.g. `Mass` or `Adfu`).
pub fn send(port: &str, cmd: SafeCommand) -> Result<String, Error> {
    let mut sp = open(port)?;
    sp.write_data_terminal_ready(true).ok();
    sp.write_request_to_send(true).ok();
    std::thread::sleep(Duration::from_millis(300));
    // drain anything pending
    let mut scratch = [0u8; 256];
    let _ = sp.read(&mut scratch);

    let mut payload = cmd.text().as_bytes().to_vec();
    payload.extend_from_slice(&SUFFIX);
    sp.write_all(&payload).map_err(|e| Error::Serial(e.to_string()))?;
    sp.flush().ok();

    if cmd.drops_port() {
        return Ok(String::new());
    }

    std::thread::sleep(Duration::from_millis(400));
    let mut reply = Vec::new();
    let mut buf = [0u8; 256];
    loop {
        match sp.read(&mut buf) {
            Ok(0) => break,
            Ok(n) => {
                reply.extend_from_slice(&buf[..n]);
                if reply.len() > 4096 {
                    break;
                }
            }
            Err(_) => break,
        }
    }
    Ok(String::from_utf8_lossy(&reply).into_owned())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn command_text_is_stable() {
        assert_eq!(SafeCommand::SaveMenu.text(), "AT+WRX=saveMenu");
        assert!(SafeCommand::Mass.drops_port());
        assert!(!SafeCommand::Ping.drops_port());
    }
}
