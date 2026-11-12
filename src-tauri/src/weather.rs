//! Weather for the watch, fetched from Open-Meteo (free, no API key) and
//! packed into the four weather commands WearPro sends (8001/0016, 000f, 000d,
//! 0010). Field layout decoded from capture #1; unknown fields reuse the values
//! WearPro sent.

use serde::{Deserialize, Serialize};

/// Watch icon codes, verified on the watch: 0 sunny, 1 partly cloudy,
/// 2 cloudy, 3 rain, 4 snow.
pub fn icon_for_wmo(code: i64) -> u8 {
    match code {
        0 => 0,
        1 | 2 => 1,
        3 | 45 | 48 => 2,
        71..=77 | 85 | 86 => 4,
        51..=67 | 80..=82 | 95..=99 => 3,
        _ => 2,
    }
}

fn t8(v: f64) -> u8 {
    v.round().clamp(-128.0, 127.0) as i8 as u8
}

#[derive(Debug, Clone, Serialize)]
pub struct Forecast {
    pub icon: u8,
    pub temp: f64,
    pub feels: f64,
    pub humidity: f64,
    /// UV index (shown by the watch as a level: moderate, strong...).
    pub uv: f64,
    pub pressure: f64,
    /// 7 days, today first: (icon, min, max, precipitation probability %).
    pub days: Vec<(u8, f64, f64, f64)>,
    /// Next 12 hours: (icon, temp).
    pub hours: Vec<(u8, f64)>,
}

/// City name as the watch expects it: length byte + UTF-16LE.
fn city_bytes(city: &str) -> Vec<u8> {
    let mut u: Vec<u8> = city.encode_utf16().take(60).flat_map(|c| c.to_le_bytes()).collect();
    u.truncate(120);
    let mut out = vec![u.len() as u8];
    out.extend(u);
    out
}

/// Build (command, body) pairs for the watch, in the order WearPro sends them.
pub fn payloads(f: &Forecast, city: &str) -> Vec<(u16, Vec<u8>)> {
    let day = |i: usize| f.days.get(i).or(f.days.last()).copied().unwrap_or((f.icon, f.temp, f.temp, 0.0));
    let hour = |i: usize| f.hours.get(i).or(f.hours.last()).copied().unwrap_or((f.icon, f.temp));
    let (_, tmin, tmax, _) = day(0);

    // 8001/0016: detailed (header 13 B, 4 x 7 daily, 7 x 12 hourly, city).
    // Header: icon, temp, min, max, feels-like, UV, then unknown fields.
    let mut p16 = vec![
        f.icon, t8(f.temp), t8(tmin), t8(tmax), t8(f.feels), f.uv.round().clamp(0.0, 20.0) as u8, 0, 0, 0, 0, 10, t8(f.humidity), 20,
    ];
    p16.extend((0..7).map(|i| day(i).0));
    p16.extend((0..7).map(|i| t8(day(i).1)));
    p16.extend((0..7).map(|i| t8(day(i).2)));
    p16.extend((0..7).map(|i| day(i).3.round().clamp(0.0, 100.0) as u8));
    p16.extend((0..12).map(|i| hour(i).0));
    p16.extend((0..12).map(|i| t8(hour(i).1)));
    p16.extend(std::iter::repeat(0u8).take(36));
    p16.extend(std::iter::repeat(10u8).take(12));
    p16.extend(std::iter::repeat(45u8).take(12));
    p16.extend(city_bytes(city));

    // 8001/000f: 00, pressure*10 (u16), unknown i16.
    let pr = ((f.pressure * 10.0).round() as i64).clamp(0, 0xffff) as u16;
    let mut p0f = vec![0u8];
    p0f.extend(pr.to_le_bytes());
    p0f.extend(0i16.to_le_bytes());

    // 8001/000d: today summary + city.
    let mut p0d = vec![t8(tmin), t8(tmax), f.icon, t8(f.temp), t8(f.humidity), 6, 4, 16, 42];
    p0d.extend(city_bytes(city));

    // 8001/0010: next 6 days (min, max, icon).
    let mut p10 = Vec::new();
    for i in 1..7 {
        let (ic, mn, mx, _) = day(i);
        p10.extend([t8(mn), t8(mx), ic]);
    }

    vec![(0x0016, p16), (0x000f, p0f), (0x000d, p0d), (0x0010, p10)]
}

fn get_json(url: &str) -> Result<serde_json::Value, String> {
    let resp = crate::mapcompile::agent()?
        .get(url)
        .timeout(std::time::Duration::from_secs(20))
        .call()
        .map_err(|e| format!("weather service: {e}"))?;
    let text = resp.into_string().map_err(|e| format!("weather service: {e}"))?;
    serde_json::from_str(&text).map_err(|e| format!("weather service: {e}"))
}

fn enc(s: &str) -> String {
    s.bytes()
        .map(|b| match b {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'.' | b'_' => (b as char).to_string(),
            _ => format!("%{b:02X}"),
        })
        .collect()
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Place {
    pub name: String,
    pub detail: String,
    pub lat: f64,
    pub lon: f64,
}

pub fn search_city(query: &str, lang: &str) -> Result<Vec<Place>, String> {
    let url = format!(
        "https://geocoding-api.open-meteo.com/v1/search?name={}&count=8&language={}&format=json",
        enc(query),
        enc(lang)
    );
    let v = get_json(&url)?;
    let mut out = Vec::new();
    for r in v["results"].as_array().cloned().unwrap_or_default() {
        let detail = [r["admin1"].as_str(), r["country"].as_str()]
            .iter()
            .flatten()
            .cloned()
            .collect::<Vec<_>>()
            .join(", ");
        out.push(Place {
            name: r["name"].as_str().unwrap_or("").to_string(),
            detail,
            lat: r["latitude"].as_f64().unwrap_or(0.0),
            lon: r["longitude"].as_f64().unwrap_or(0.0),
        });
    }
    Ok(out)
}

pub fn fetch(lat: f64, lon: f64) -> Result<Forecast, String> {
    let url = format!(
        "https://api.open-meteo.com/v1/forecast?latitude={lat:.4}&longitude={lon:.4}\
         &current=temperature_2m,apparent_temperature,relative_humidity_2m,weather_code,uv_index,pressure_msl\
         &hourly=temperature_2m,weather_code\
         &daily=weather_code,temperature_2m_max,temperature_2m_min,precipitation_probability_max\
         &timezone=auto&forecast_days=7"
    );
    parse(&get_json(&url)?)
}

pub fn parse(v: &serde_json::Value) -> Result<Forecast, String> {
    let c = &v["current"];
    let num = |x: &serde_json::Value| x.as_f64().unwrap_or(0.0);
    let arr = |x: &serde_json::Value| x.as_array().cloned().unwrap_or_default();
    if c.is_null() {
        return Err("weather service: unexpected answer".into());
    }
    let d = &v["daily"];
    let (dc, dmax, dmin, dp) = (arr(&d["weather_code"]), arr(&d["temperature_2m_max"]), arr(&d["temperature_2m_min"]), arr(&d["precipitation_probability_max"]));
    let days = (0..dc.len().min(7))
        .map(|i| {
            (
                icon_for_wmo(dc[i].as_i64().unwrap_or(2)),
                dmin.get(i).map(num).unwrap_or(0.0),
                dmax.get(i).map(num).unwrap_or(0.0),
                dp.get(i).map(num).unwrap_or(0.0),
            )
        })
        .collect();

    // Hourly: start at the current local hour.
    let h = &v["hourly"];
    let (ht, htemp, hcode) = (arr(&h["time"]), arr(&h["temperature_2m"]), arr(&h["weather_code"]));
    let now = c["time"].as_str().unwrap_or("");
    let prefix = now.get(..13).unwrap_or(now);
    let start = ht.iter().position(|t| t.as_str().map(|s| s.starts_with(prefix)).unwrap_or(false)).unwrap_or(0);
    let hours = (start..(start + 12).min(ht.len()))
        .map(|i| (icon_for_wmo(hcode.get(i).and_then(|x| x.as_i64()).unwrap_or(2)), htemp.get(i).map(num).unwrap_or(0.0)))
        .collect();

    Ok(Forecast {
        icon: icon_for_wmo(c["weather_code"].as_i64().unwrap_or(2)),
        temp: num(&c["temperature_2m"]),
        feels: num(&c["apparent_temperature"]),
        humidity: num(&c["relative_humidity_2m"]),
        uv: num(&c["uv_index"]),
        pressure: num(&c["pressure_msl"]),
        days,
        hours,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn city_is_utf16_with_length() {
        assert_eq!(city_bytes("ירושלים"), {
            let mut v = vec![14u8];
            v.extend([0xd9, 0x05, 0xe8, 0x05, 0xd5, 0x05, 0xe9, 0x05, 0xdc, 0x05, 0xd9, 0x05, 0xdd, 0x05]);
            v
        });
    }

    #[test]
    fn payload_sizes_match_capture() {
        let f = Forecast {
            icon: 1, temp: 15.0, feels: 14.0, humidity: 54.0, uv: 5.0, pressure: 1023.0,
            days: vec![(1, 11.0, 19.0, 0.0); 7],
            hours: vec![(1, 11.0); 12],
        };
        let p = payloads(&f, "ירושלים");
        assert_eq!(p[0].1.len(), 140);
        assert_eq!(p[1].1, vec![0x00, 0xf6, 0x27, 0x00, 0x00]);
        assert_eq!(p[2].1.len(), 24);
        assert_eq!(p[3].1.len(), 18);
        assert_eq!(&p[2].1[..5], &[11, 19, 1, 15, 54]);
    }

    #[test]
    fn negative_temps_are_signed() {
        assert_eq!(t8(-3.4), 0xfd);
        assert_eq!(icon_for_wmo(73), 4);
        assert_eq!(icon_for_wmo(95), 3);
    }

    #[test]
    fn parse_open_meteo() {
        let v: serde_json::Value = serde_json::from_str(r#"{
          "current":{"time":"2026-11-12T23:30","temperature_2m":12.6,"apparent_temperature":11.0,
                     "relative_humidity_2m":70,"weather_code":3,"uv_index":4.6,"pressure_msl":923.4},
          "hourly":{"time":["2026-11-12T22:00","2026-11-12T23:00","2026-11-13T00:00"],
                    "temperature_2m":[13,12.5,12],"weather_code":[0,61,71]},
          "daily":{"weather_code":[3,0],"temperature_2m_max":[19,21],"temperature_2m_min":[10,11],
                   "precipitation_probability_max":[40,0]}}"#).unwrap();
        let f = parse(&v).unwrap();
        assert_eq!(f.icon, 2);
        assert_eq!(f.hours, vec![(3, 12.5), (4, 12.0)]);
        assert_eq!(payloads(&f, "x")[0].1[5], 5);
        assert_eq!(f.days[1], (0, 11.0, 21.0, 0.0));
        assert_eq!(f.uv, 4.6);
    }
}
