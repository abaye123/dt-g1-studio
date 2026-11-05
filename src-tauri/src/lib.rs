//! Tauri shell for DT G1 Studio. Exposes a small set of commands over
//! `watch-core` to the web UI. Only read or reversible mode-change actions.

use std::time::{Duration, Instant};

use serde::Serialize;
use watch_core::partition;
use watch_core::serial::{self, SafeCommand, UsbMode};

#[derive(Serialize)]
struct Connection {
    /// "charging" | "mass" | "adfu" | "absent"
    mode: String,
    /// The serial port name when in Charging mode.
    port: Option<String>,
}

fn mode_str(m: UsbMode) -> &'static str {
    match m {
        UsbMode::Charging => "charging",
        UsbMode::MassStorage => "mass",
        UsbMode::Adfu => "adfu",
        UsbMode::Absent => "absent",
    }
}

/// Detect how the watch is currently connected.
#[tauri::command]
fn detect_connection() -> Connection {
    let mode = serial::detect_mode();
    Connection {
        mode: mode_str(mode).to_string(),
        port: serial::find_port(),
    }
}

/// Send a harmless ping (`AT+WRX=always`) and return the reply. Used by the
/// connection wizard to confirm the serial channel really works.
#[tauri::command]
fn ping_watch() -> Result<String, String> {
    let port = serial::find_port().ok_or_else(|| "not_charging".to_string())?;
    serial::send(&port, SafeCommand::Ping).map_err(|e| e.to_string())
}

/// The app version, for the About area.
#[tauri::command]
fn app_version() -> String {
    env!("CARGO_PKG_VERSION").to_string()
}

#[derive(Serialize)]
struct DirEntry {
    name: String,
    is_dir: bool,
    size: u64,
}

/// Expose the hidden partition (sends realUpan + mass, then waits for the drive
/// to mount) and return its root path, e.g. "E:\\". Requires Charging mode.
#[tauri::command]
fn expose_partition() -> Result<String, String> {
    // If it is already mounted, just return it.
    if let Some(root) = partition::find() {
        return Ok(root.to_string_lossy().into_owned());
    }
    partition::expose().map_err(|e| e.to_string())?;
    let deadline = Instant::now() + Duration::from_secs(40);
    while Instant::now() < deadline {
        if let Some(root) = partition::find() {
            return Ok(root.to_string_lossy().into_owned());
        }
        std::thread::sleep(Duration::from_millis(800));
    }
    Err("partition_not_found".into())
}

/// List a directory under the hidden partition, read-only. `rel` is relative to
/// the partition root ("" for the root).
#[tauri::command]
fn partition_list(rel: String) -> Result<Vec<DirEntry>, String> {
    let root = partition::find().ok_or_else(|| "partition_not_mounted".to_string())?;
    let entries = partition::list_dir(&root, &rel).map_err(|e| e.to_string())?;
    Ok(entries
        .into_iter()
        .map(|e| DirEntry { name: e.name, is_dir: e.is_dir, size: e.size })
        .collect())
}

pub fn run() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![
            detect_connection,
            ping_watch,
            app_version,
            expose_partition,
            partition_list
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
