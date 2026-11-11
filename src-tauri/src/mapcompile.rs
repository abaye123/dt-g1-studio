//! OSM map compilation using the official dtg1-map-tools compiler.
//!
//! Flow: download the standalone Windows compiler (exe + features.csv) from the
//! project's GitHub release, download OSM data for a bounding box from the
//! Overpass API, run the compiler, and collect the watch map files. Pure path
//! based functions; the Tauri layer wires app data dirs and device paths.

use std::io::Read;
use std::path::{Path, PathBuf};

const RELEASES_API: &str = "https://api.github.com/repos/purrrock/dtg1-map-tools/releases/latest";
const UA: &str = "dt-g1-studio";

/// OSM data sources, tried in order. The OSM API is reliable for small areas
/// (it caps at ~0.25 deg^2 / 50k nodes); the Overpass mirrors handle larger.
const OSM_ENDPOINTS: &[&str] = &[
    "https://api.openstreetmap.org/api/0.6/map",
    "https://overpass-api.de/api/map",
    "https://overpass.kumi.systems/api/map",
];

/// An HTTP agent that uses the OS certificate store (schannel on Windows), so
/// it trusts corporate/antivirus TLS-inspection roots the way the system does.
pub(crate) fn agent() -> Result<ureq::Agent, String> {
    let connector = native_tls::TlsConnector::new().map_err(|e| e.to_string())?;
    Ok(ureq::AgentBuilder::new()
        .tls_connector(std::sync::Arc::new(connector))
        .timeout(std::time::Duration::from_secs(180))
        .build())
}

/// The map files the compiler can produce.
pub const OUTPUTS: &[&str] = &[
    "roads.mlp", "roads.idx", "roads.db",
    "landuse.mlp", "landuse.idx", "landuse.db",
    "water.mlp", "water.idx", "water.db",
    "pois.idx", "pois.db",
    "map.name",
];

pub fn exe_path(compiler_dir: &Path) -> PathBuf {
    compiler_dir.join("dtg1_map_compiler.exe")
}

/// True if the compiler is downloaded and ready.
pub fn is_ready(compiler_dir: &Path) -> bool {
    exe_path(compiler_dir).is_file() && compiler_dir.join("features.csv").is_file()
}

/// Download and extract the compiler release into `compiler_dir`.
pub fn setup(compiler_dir: &Path) -> Result<(), String> {
    let http = agent()?;
    // Find the .zip asset of the latest release.
    let meta = http
        .get(RELEASES_API)
        .set("User-Agent", UA)
        .call()
        .map_err(|e| format!("release lookup failed: {e}"))?
        .into_string()
        .map_err(|e| e.to_string())?;
    let json: serde_json::Value = serde_json::from_str(&meta).map_err(|e| e.to_string())?;
    let url = json["assets"]
        .as_array()
        .and_then(|a| {
            a.iter()
                .filter_map(|x| x["browser_download_url"].as_str())
                .find(|u| u.ends_with(".zip"))
        })
        .ok_or_else(|| "no zip asset in the latest release".to_string())?
        .to_string();

    // Download the zip into memory.
    let mut buf = Vec::new();
    http.get(&url)
        .set("User-Agent", UA)
        .call()
        .map_err(|e| format!("download failed: {e}"))?
        .into_reader()
        .read_to_end(&mut buf)
        .map_err(|e| e.to_string())?;

    // Extract exe, features.csv and any osm_optimizer into compiler_dir (flattened).
    std::fs::create_dir_all(compiler_dir).map_err(|e| e.to_string())?;
    let mut zip = zip::ZipArchive::new(std::io::Cursor::new(buf)).map_err(|e| e.to_string())?;
    for i in 0..zip.len() {
        let mut f = zip.by_index(i).map_err(|e| e.to_string())?;
        if f.is_dir() {
            continue;
        }
        let base = match f.enclosed_name().and_then(|p| p.file_name().map(|n| n.to_owned())) {
            Some(n) => n,
            None => continue,
        };
        let name = base.to_string_lossy().to_lowercase();
        let keep = name.ends_with(".exe") || name == "features.csv" || name.ends_with(".dll");
        if !keep {
            continue;
        }
        let mut out = std::fs::File::create(compiler_dir.join(&base)).map_err(|e| e.to_string())?;
        std::io::copy(&mut f, &mut out).map_err(|e| e.to_string())?;
    }
    if !is_ready(compiler_dir) {
        return Err("compiler files missing after extract".into());
    }
    // A new release brings a new features.csv; drop the cached pristine copy so
    // the next compile snapshots the new one.
    let _ = std::fs::remove_file(compiler_dir.join(BASE_LUT));
    Ok(())
}

/// Pristine copy of the release's features.csv. Each compile writes its own
/// features.csv (the compiler reads it from beside the exe) derived from this.
const BASE_LUT: &str = "features.base.csv";

/// Road style for the compiled map.
/// - "bold": the compiler's own table, which promotes most streets to the
///   primary-road code 5113 so they are drawn at every zoom level.
/// - "factory": the true hardware codes (e.g. residential = 5122), as in the
///   official WearPro maps; local streets then appear only when zoomed in.
pub fn road_table(base: &str, style: &str) -> String {
    if style != "factory" {
        return base.to_string();
    }
    let mut out = String::with_capacity(base.len());
    for (i, line) in base.lines().enumerate() {
        let cols: Vec<&str> = line.split(';').collect();
        // Code;fclass;Color;LOD;Layer;OSM_Tags;Description;Remap_Code;Remap_Color;Remap_LOD;...
        if i > 0 && cols.len() >= 11 && cols[4].trim() == "roads" {
            let mut c: Vec<String> = cols.iter().map(|s| s.to_string()).collect();
            // keep the user GPX track special-case as the author set it
            if c[1].trim() != "gpx_track" {
                c[7] = c[0].trim().to_string();
                c[8] = c[2].trim().to_string();
                c[9] = c[3].trim().to_string();
            }
            out.push_str(&c.join(";"));
        } else {
            out.push_str(line);
        }
        out.push('\n');
    }
    out
}

/// Prepare features.csv for one compile from the pristine base table: apply the
/// road style, then append `extra_rows` at the end (the loader reads top to
/// bottom, so later rows win).
fn write_lut(compiler_dir: &Path, style: &str, extra_rows: &[&str]) -> Result<(), String> {
    let base_path = compiler_dir.join(BASE_LUT);
    if !base_path.is_file() {
        std::fs::copy(compiler_dir.join("features.csv"), &base_path).map_err(|e| format!("lut base: {e}"))?;
    }
    let base = std::fs::read_to_string(&base_path).map_err(|e| e.to_string())?;
    let mut lut = road_table(&base, style);
    if !lut.ends_with('\n') {
        lut.push('\n');
    }
    for row in extra_rows {
        lut.push_str(row);
        lut.push('\n');
    }
    std::fs::write(compiler_dir.join("features.csv"), lut).map_err(|e| e.to_string())
}

/// The compiler stores names with spaces turned into '_'. The official maps
/// keep real spaces (the watch renders them), so put the spaces back in the
/// `name` field of a `.db` file and refresh the YZL header MD5
/// (bytes 0x10..0x20 = MD5 of everything after the 32-byte header).
pub fn restore_name_spaces(db_path: &Path) -> Result<bool, String> {
    use md5::{Digest, Md5};
    let mut b = std::fs::read(db_path).map_err(|e| e.to_string())?;
    if b.len() < 32 + 32 || &b[0..3] != b"YZL" {
        return Ok(false);
    }
    let d = 32; // dBase starts after the YZL header
    let rec_count = u32::from_le_bytes(b[d + 4..d + 8].try_into().unwrap()) as usize;
    let hdr_len = u16::from_le_bytes(b[d + 8..d + 10].try_into().unwrap()) as usize;
    let rec_len = u16::from_le_bytes(b[d + 10..d + 12].try_into().unwrap()) as usize;
    // Field descriptors: 32 bytes each from offset 32, terminated by 0x0D.
    let (mut off, mut field) = (1usize, None); // 1 = deletion flag byte
    let mut p = d + 32;
    while p + 32 <= b.len() && b[p] != 0x0D {
        let fname: Vec<u8> = b[p..p + 11].iter().copied().take_while(|&c| c != 0).collect();
        let flen = b[p + 16] as usize;
        if fname.eq_ignore_ascii_case(b"name") {
            field = Some((off, flen));
        }
        off += flen;
        p += 32;
    }
    let Some((foff, flen)) = field else { return Ok(false) };
    let mut changed = false;
    for i in 0..rec_count {
        let start = d + hdr_len + i * rec_len + foff;
        if start + flen > b.len() {
            break;
        }
        for c in &mut b[start..start + flen] {
            if *c == b'_' {
                *c = b' ';
                changed = true;
            }
        }
    }
    if changed {
        let digest = Md5::digest(&b[32..]);
        b[16..32].copy_from_slice(&digest);
        std::fs::write(db_path, &b).map_err(|e| e.to_string())?;
    }
    Ok(changed)
}

/// Download OSM XML for a bounding box into `out`, trying several providers.
/// Returns bytes written. The bbox is west,south,east,north (lon/lat degrees).
pub fn download_osm(w: f64, s: f64, e: f64, n: f64, out: &Path) -> Result<u64, String> {
    if let Some(p) = out.parent() {
        std::fs::create_dir_all(p).map_err(|e| e.to_string())?;
    }
    let bbox = format!("{w},{s},{e},{n}");
    let http = agent()?;
    let mut last = String::from("no provider reached");
    let mut too_large = false;
    for base in OSM_ENDPOINTS {
        let url = format!("{base}?bbox={bbox}");
        match http.get(&url).set("User-Agent", UA).call() {
            Ok(resp) => {
                let mut file = std::fs::File::create(out).map_err(|e| e.to_string())?;
                let bytes = std::io::copy(&mut resp.into_reader(), &mut file).map_err(|e| e.to_string())?;
                if bytes >= 200 {
                    return Ok(bytes);
                }
                last = "empty response".into();
            }
            Err(ureq::Error::Status(code, _)) => {
                // 400/413 = area too large for that provider; try the next (Overpass handles more).
                if code == 400 || code == 413 {
                    too_large = true;
                }
                last = format!("provider returned {code}");
            }
            Err(other) => last = format!("request failed: {other}"),
        }
    }
    if too_large {
        return Err("area_too_large".into());
    }
    Err(format!("osm_download_failed: {last}"))
}

/// Run the compiler on `osm` and move the produced map files into `staging/<_>`.
/// Returns (filename, size) for each produced file. `compiler_dir` holds the exe
/// and features.csv; it is used as the working directory. `road_style` is
/// "bold" or "factory" (see [`road_table`]).
pub fn compile(
    compiler_dir: &Path,
    osm: &Path,
    staging: &Path,
    region_name: &str,
    poi_mode: &str,
    road_style: &str,
    extra_lut_rows: &[&str],
) -> Result<Vec<(String, u64)>, String> {
    // Clean any stale inputs/outputs in the compiler dir.
    let map_osm = compiler_dir.join("map.osm");
    let _ = std::fs::remove_file(&map_osm);
    for o in OUTPUTS {
        let _ = std::fs::remove_file(compiler_dir.join(o));
    }
    write_lut(compiler_dir, road_style, extra_lut_rows)?;
    std::fs::copy(osm, &map_osm).map_err(|e| format!("copy osm: {e}"))?;

    let out = std::process::Command::new(exe_path(compiler_dir))
        .current_dir(compiler_dir)
        .env("REGION_NAME", region_name)
        .arg("-p")
        .arg(poi_mode)
        .output()
        .map_err(|e| format!("run compiler: {e}"))?;
    let _ = std::fs::remove_file(&map_osm);
    if !out.status.success() {
        let log = String::from_utf8_lossy(&out.stdout);
        let tail: String = log.lines().rev().take(4).collect::<Vec<_>>().join(" | ");
        return Err(format!("compiler failed: {tail}"));
    }

    // Collect outputs into a fresh staging folder.
    if staging.exists() {
        std::fs::remove_dir_all(staging).ok();
    }
    std::fs::create_dir_all(staging).map_err(|e| e.to_string())?;
    let mut files = Vec::new();
    for o in OUTPUTS {
        let src = compiler_dir.join(o);
        if src.is_file() {
            let size = src.metadata().map(|m| m.len()).unwrap_or(0);
            let dst = staging.join(o);
            std::fs::rename(&src, &dst).or_else(|_| {
                std::fs::copy(&src, &dst).map(|_| ()).and_then(|_| std::fs::remove_file(&src))
            }).map_err(|e| e.to_string())?;
            if o.ends_with(".db") {
                restore_name_spaces(&dst)?;
            }
            files.push((o.to_string(), size));
        }
    }
    if files.is_empty() {
        return Err("compiler produced no map files".into());
    }
    Ok(files)
}

#[cfg(test)]
mod tests {
    use super::*;

    const LUT: &str = "Code;fclass;Color;LOD;Layer;OSM_Tags;Description;Remap_Code;Remap_Color;Remap_LOD;Enabled;Shape\n\
5111;gpx_track;Orange;1000;roads;route=tracks;t;5111;Orange;1000;1;\n\
5122;residential;Gray;100;roads;highway=residential;r;5113;Yellow;100;1;\n\
7201;park;Green;500;landuse;leisure=park;p;7202;Green;500;1;\n";

    #[test]
    fn factory_roads_use_true_codes() {
        let bold = road_table(LUT, "bold");
        assert_eq!(bold, LUT);
        let f = road_table(LUT, "factory");
        assert!(f.contains("5122;residential;Gray;100;roads;highway=residential;r;5122;Gray;100;1;"));
        // non-road rows and the GPX track row are untouched
        assert!(f.contains("7201;park;Green;500;landuse;leisure=park;p;7202;Green;500;1;"));
        assert!(f.contains("5111;gpx_track;Orange;1000;roads;route=tracks;t;5111;Orange;1000;1;"));
    }
}

#[cfg(test)]
mod itest {
    use super::*;

    // Network + subprocess integration test. Run explicitly:
    //   cargo test -p dt-g1-studio-app --lib full_flow -- --ignored --nocapture
    #[test]
    #[ignore]
    fn full_flow() {
        let base = std::env::temp_dir().join("dtg1_mapc_test");
        let compiler = base.join("compiler");
        setup(&compiler).expect("setup");
        assert!(is_ready(&compiler), "compiler ready");
        let osm = base.join("map.osm");
        // A tiny area (a few central Tel Aviv blocks) keeps OSM small and fast.
        let bytes = download_osm(34.770, 32.070, 34.778, 32.076, &osm).expect("download");
        println!("osm bytes = {bytes}");
        let staging = base.join("staging");
        let files = compile(&compiler, &osm, &staging, "TestArea", "landuse", "bold", &[]).expect("compile");
        println!("map files = {files:?}");
        assert!(files.iter().any(|(n, _)| n == "map.name"));
    }
}
