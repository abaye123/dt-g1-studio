//! Tool resolution with an offline-first, update-when-possible policy.
//!
//! The app ships bundled copies of its external tools (the Glyphix `gx`
//! toolchain, the OSM map compiler, and the PurrLaunch launcher) under
//! `resources/` so it works with no network. At runtime, for the updatable
//! tools (map compiler, launcher) a newer GitHub release is fetched when
//! reachable; on any failure or offline, the bundled copy is used. `gx` is
//! bundled-only for now.
//!
//! - `gx` runs in place from its (read-only) bundled dir.
//! - the map compiler needs a writable working dir, so it is staged into
//!   app-data; updates download there.
//! - the launcher is a single `.pkg`, staged into app-data.

use std::path::{Path, PathBuf};
use tauri::Manager;

/// Dev fallback for the gx toolchain when no bundled `resources/gx` exists
/// (i.e. running from `cargo run` before `scripts/vendor.ps1` has assembled it).
const GX_DEV_WIN: &str =
    r"C:\dev\dt-g1-watch\dt-g1-research\tools\glyphix-toolchain\node_modules\@glyphix\cli-win32-x64";

const MAP_REPO: &str = "purrrock/dtg1-map-tools";
const LAUNCHER_REPO: &str = "purrrock/PurrLaunch";
pub const LAUNCHER_PKG_NAME: &str = "com.xfaith.woodenfish.pkg";

/// The bundled resources directory: next to the executable in a packaged build,
/// or the crate's `resources/` dir in development.
pub fn resources_root(app: &tauri::AppHandle) -> PathBuf {
    if let Ok(rd) = app.path().resource_dir() {
        let p = rd.join("resources");
        if p.is_dir() {
            return p;
        }
    }
    // Portable layout: resources sit next to the executable.
    if let Ok(exe) = std::env::current_exe() {
        if let Some(dir) = exe.parent() {
            let p = dir.join("resources");
            if p.is_dir() {
                return p;
            }
        }
    }
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("resources")
}

fn read_manifest(app: &tauri::AppHandle) -> serde_json::Value {
    let p = resources_root(app).join("manifest.json");
    std::fs::read(p)
        .ok()
        .and_then(|b| {
            let s = String::from_utf8_lossy(&b);
            let s = s.trim_start_matches('\u{feff}'); // tolerate a UTF-8 BOM
            serde_json::from_str::<serde_json::Value>(s).ok()
        })
        .unwrap_or_else(|| serde_json::json!({}))
}

/// The bundled version recorded in the manifest for `key` (gx | map-compiler | launcher).
pub fn bundled_version(app: &tauri::AppHandle, key: &str) -> Option<String> {
    read_manifest(app)
        .get(key)
        .and_then(|v| v.as_str())
        .filter(|s| !s.is_empty())
        .map(String::from)
}

fn tools_dir(app: &tauri::AppHandle) -> Option<PathBuf> {
    Some(crate::data_root(app).join("tools"))
}

fn version_file(dir: &Path) -> PathBuf {
    dir.join(".version")
}
fn write_version(dir: &Path, v: Option<&str>) {
    if let Some(v) = v {
        let _ = std::fs::write(version_file(dir), v);
    }
}
fn read_version(dir: &Path) -> Option<String> {
    std::fs::read_to_string(version_file(dir))
        .ok()
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
}

fn copy_dir(src: &Path, dst: &Path) -> std::io::Result<()> {
    std::fs::create_dir_all(dst)?;
    for entry in std::fs::read_dir(src)? {
        let entry = entry?;
        let from = entry.path();
        let to = dst.join(entry.file_name());
        if from.is_dir() {
            copy_dir(&from, &to)?;
        } else {
            std::fs::copy(&from, &to)?;
        }
    }
    Ok(())
}

// --- gx ---------------------------------------------------------------------

fn gx_exe_name() -> &'static str {
    if cfg!(windows) {
        "gx.exe"
    } else {
        "gx"
    }
}

/// The gx toolchain root (containing `bin/`), bundled or the dev fallback.
/// gx runs in place; the device profile is bundled under `share/glyphix/devices`.
pub fn gx_dir(app: &tauri::AppHandle) -> Option<PathBuf> {
    let candidates = [resources_root(app).join("gx"), PathBuf::from(GX_DEV_WIN)];
    candidates
        .into_iter()
        .find(|c| c.join("bin").join(gx_exe_name()).is_file())
}

// --- map compiler -----------------------------------------------------------

/// The writable map-compiler dir, staging the bundled copy into app-data on
/// first use. Returns None when neither a staged nor a bundled copy exists
/// (e.g. on Linux, where there is no upstream compiler build).
pub fn map_compiler_dir(app: &tauri::AppHandle) -> Option<PathBuf> {
    let active = tools_dir(app)?.join("map-compiler");
    if crate::mapcompile::is_ready(&active) {
        return Some(active);
    }
    let bundled = resources_root(app).join("map-compiler");
    if crate::mapcompile::is_ready(&bundled) && copy_dir(&bundled, &active).is_ok() {
        write_version(&active, bundled_version(app, "map-compiler").as_deref());
        return Some(active);
    }
    None
}

// --- launcher ---------------------------------------------------------------

/// The launcher `.pkg`, staging the bundled copy into app-data on first use.
pub fn launcher_pkg(app: &tauri::AppHandle) -> Option<PathBuf> {
    let active = tools_dir(app)?.join("launcher");
    let pkg = active.join(LAUNCHER_PKG_NAME);
    if pkg.is_file() {
        return Some(pkg);
    }
    let bundled = resources_root(app).join("launcher").join(LAUNCHER_PKG_NAME);
    if bundled.is_file() {
        std::fs::create_dir_all(&active).ok()?;
        std::fs::copy(&bundled, &pkg).ok()?;
        write_version(&active, bundled_version(app, "launcher").as_deref());
        return Some(pkg);
    }
    None
}

// --- updates ----------------------------------------------------------------

fn parse_ver(s: &str) -> Vec<u64> {
    s.trim_start_matches('v')
        .split('.')
        .map(|p| {
            p.chars()
                .take_while(|c| c.is_ascii_digit())
                .collect::<String>()
                .parse()
                .unwrap_or(0)
        })
        .collect()
}

/// True if `a` is a strictly newer version string than `b`.
fn newer(a: &str, b: &str) -> bool {
    parse_ver(a) > parse_ver(b)
}

fn latest_tag(repo: &str) -> Option<String> {
    let http = crate::mapcompile::agent().ok()?;
    let url = format!("https://api.github.com/repos/{repo}/releases/latest");
    let body = http
        .get(&url)
        .set("User-Agent", "dt-g1-studio")
        .call()
        .ok()?
        .into_string()
        .ok()?;
    let j: serde_json::Value = serde_json::from_str(&body).ok()?;
    j.get("tag_name")
        .and_then(|v| v.as_str())
        .map(String::from)
}

/// Best-effort: if a newer map compiler release exists, download it into the
/// active dir. Returns the new version on success, None otherwise (incl. offline).
pub fn update_map_compiler(app: &tauri::AppHandle) -> Option<String> {
    let active = map_compiler_dir(app)?; // ensure a working copy exists first
    let cur = read_version(&active).or_else(|| bundled_version(app, "map-compiler"));
    let latest = latest_tag(MAP_REPO)?;
    let do_update = cur.as_deref().map(|c| newer(&latest, c)).unwrap_or(true);
    if do_update && crate::mapcompile::setup(&active).is_ok() {
        write_version(&active, Some(&latest));
        return Some(latest);
    }
    None
}

/// Best-effort: if a newer launcher release exists, download its `.pkg` into the
/// active dir. Returns the new version on success, None otherwise.
pub fn update_launcher(app: &tauri::AppHandle) -> Option<String> {
    let pkg = launcher_pkg(app)?; // ensure a staged copy exists first
    let active = pkg.parent()?.to_path_buf();
    let cur = read_version(&active).or_else(|| bundled_version(app, "launcher"));
    let latest = latest_tag(LAUNCHER_REPO)?;
    let do_update = cur.as_deref().map(|c| newer(&latest, c)).unwrap_or(true);
    if do_update && crate::appbuild::setup_launcher(&active).is_ok() {
        write_version(&active, Some(&latest));
        return Some(latest);
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn version_compare() {
        assert!(newer("v1.6.0", "v1.5.9"));
        assert!(newer("v1.2.10", "v1.2.9"));
        assert!(!newer("v1.2.2", "v1.2.2"));
        assert!(!newer("v1.0.0", "v2.0.0"));
        assert!(newer("0.9.0", "0.8.0-rc")); // trailing -rc ignored on the patch part
    }
}
