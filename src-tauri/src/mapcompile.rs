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
    Ok(())
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
/// and features.csv; it is used as the working directory.
pub fn compile(
    compiler_dir: &Path,
    osm: &Path,
    staging: &Path,
    region_name: &str,
    poi_mode: &str,
) -> Result<Vec<(String, u64)>, String> {
    // Clean any stale inputs/outputs in the compiler dir.
    let map_osm = compiler_dir.join("map.osm");
    let _ = std::fs::remove_file(&map_osm);
    for o in OUTPUTS {
        let _ = std::fs::remove_file(compiler_dir.join(o));
    }
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
            std::fs::rename(&src, staging.join(o)).or_else(|_| {
                std::fs::copy(&src, staging.join(o)).map(|_| ()).and_then(|_| std::fs::remove_file(&src))
            }).map_err(|e| e.to_string())?;
            files.push((o.to_string(), size));
        }
    }
    if files.is_empty() {
        return Err("compiler produced no map files".into());
    }
    Ok(files)
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
        let files = compile(&compiler, &osm, &staging, "TestArea", "landuse").expect("compile");
        println!("map files = {files:?}");
        assert!(files.iter().any(|(n, _)| n == "map.name"));
    }
}
