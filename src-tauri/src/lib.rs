//! Tauri shell for DT G1 Studio. Exposes a small set of commands over
//! `watch-core` to the web UI. Only read or reversible mode-change actions.

use std::path::{Path, PathBuf};
use std::sync::Mutex;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use serde::{Deserialize, Serialize};
use tauri::{Manager, State};
use watch_core::menu::{Menu, Slot};
use watch_core::partition;
use watch_core::serial::{self, SafeCommand, UsbMode};

/// When `Some(path)`, the app is in demo mode and serves the sample data under
/// `path` instead of a connected watch. Managed as Tauri state.
type Demo = Mutex<Option<PathBuf>>;

fn demo_root(state: &State<'_, Demo>) -> Option<PathBuf> {
    state.lock().ok().and_then(|g| g.clone())
}

/// The sample menu (a real factory `menu_list.cfg`, just AppIDs, no secrets).
const DEMO_MENU: &[u8] = include_bytes!("../../samples/menu_list.cfg");

/// A representative hidden-partition tree for demo mode: (relative path, size).
/// Sizes are modest on purpose so the sample stays light.
const SAMPLE_TREE: &[(&str, u64)] = &[
    ("gui/apps/com.xfaith.woodenfish.pkg", 33632),
    ("gui/apps/com.xfaith.weather.pkg", 71856),
    ("gui/apps/com.xfaith.calendar.pkg", 43731),
    ("gui/apps/com.xfaith.translate.pkg", 13394),
    ("gui/apps/com.dtno1.audio-book.pkg", 90120),
    ("gui/apps/launcher.app.pkg", 100027),
    ("gui/fonts/harmony.ttf", 120000),
    ("gui/fonts/AliPuHui-Regular.ttf", 85320),
    ("gui/font-faces", 115),
    ("data/g_clock_ids", 2000),
    ("data/g_clock_grp", 4800),
    ("data/sports_layout.hex", 10450),
    ("data/wrist_band_data", 1256),
    ("data/mass/APIConfig.json", 2173),
    ("res/user_clock_bg.bin", 116345),
    ("res/user_clock_thumb.bin", 11634),
    ("ring/ding.mp3", 24364),
    ("ring/ring0.mp3", 25279),
    ("ring/ring1.mp3", 25100),
    ("icoe/ephemeris.pgl", 84059),
    ("logs/glyphix.log", 17837),
    ("battery/battery.txt", 64),
    ("temp/placeholder", 8),
];

/// Create the sample partition tree (and the demo menu) under `root`.
fn build_sample(root: &Path) -> std::io::Result<()> {
    if root.exists() {
        std::fs::remove_dir_all(root).ok();
    }
    for (rel, size) in SAMPLE_TREE {
        let p = root.join(rel);
        if let Some(parent) = p.parent() {
            std::fs::create_dir_all(parent)?;
        }
        std::fs::write(&p, vec![0u8; *size as usize])?;
    }
    let custom = root.join("custom");
    std::fs::create_dir_all(&custom)?;
    std::fs::write(custom.join("menu_list.cfg"), DEMO_MENU)?;
    Ok(())
}

/// Turn demo mode on. With no `path`, a bundled sample tree is generated in the
/// app data folder; with a `path`, that folder is used as the partition root
/// (so a real backup folder can be previewed). Returns the root path.
#[tauri::command]
fn enable_demo(
    app: tauri::AppHandle,
    state: State<'_, Demo>,
    path: Option<String>,
) -> Result<String, String> {
    let root = match path {
        Some(p) if !p.trim().is_empty() => PathBuf::from(p),
        _ => {
            let root = app
                .path()
                .app_data_dir()
                .map_err(|e| e.to_string())?
                .join("demo-partition");
            build_sample(&root).map_err(|e| e.to_string())?;
            root
        }
    };
    *state.lock().map_err(|e| e.to_string())? = Some(root.clone());
    Ok(root.to_string_lossy().into_owned())
}

/// Turn demo mode off (back to the real watch).
#[tauri::command]
fn disable_demo(state: State<'_, Demo>) -> Result<(), String> {
    *state.lock().map_err(|e| e.to_string())? = None;
    Ok(())
}

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

/// Detect how the watch is currently connected (or report demo mode).
#[tauri::command]
fn detect_connection(state: State<'_, Demo>) -> Connection {
    if demo_root(&state).is_some() {
        return Connection { mode: "demo".into(), port: None };
    }
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

/// Mount the hidden partition and return its root, exposing it first if needed.
fn mount_partition() -> Result<PathBuf, String> {
    if let Some(root) = partition::find() {
        return Ok(root);
    }
    partition::expose().map_err(|e| e.to_string())?;
    let deadline = Instant::now() + Duration::from_secs(40);
    while Instant::now() < deadline {
        if let Some(root) = partition::find() {
            return Ok(root);
        }
        std::thread::sleep(Duration::from_millis(800));
    }
    Err("partition_not_found".into())
}

/// Expose the hidden partition (sends realUpan + mass, then waits for the drive
/// to mount) and return its root path, e.g. "E:\\". Requires Charging mode.
/// In demo mode this returns the sample root.
#[tauri::command]
fn expose_partition(state: State<'_, Demo>) -> Result<String, String> {
    if let Some(root) = demo_root(&state) {
        return Ok(root.to_string_lossy().into_owned());
    }
    mount_partition().map(|p| p.to_string_lossy().into_owned())
}

/// List a directory under the hidden partition, read-only. `rel` is relative to
/// the partition root ("" for the root).
#[tauri::command]
fn partition_list(state: State<'_, Demo>, rel: String) -> Result<Vec<DirEntry>, String> {
    let root = demo_root(&state)
        .or_else(partition::find)
        .ok_or_else(|| "partition_not_mounted".to_string())?;
    let entries = partition::list_dir(&root, &rel).map_err(|e| e.to_string())?;
    Ok(entries
        .into_iter()
        .map(|e| DirEntry { name: e.name, is_dir: e.is_dir, size: e.size })
        .collect())
}

#[derive(Serialize)]
struct MenuSlot {
    app_id: u32,
    hex: String,
    name: String,
    icon_id: u32,
    name_id: u32,
    visible: bool,
}

#[derive(Deserialize)]
struct SlotInput {
    app_id: u32,
    icon_id: u32,
    name_id: u32,
    visible: bool,
}

fn menu_file(root: &std::path::Path) -> PathBuf {
    root.join("custom").join("menu_list.cfg")
}

/// Load the watch menu: ask the watch to dump it (`saveMenu`), expose the
/// partition, read `custom/menu_list.cfg`, and return the used slots with names.
/// Requires Charging mode (switches the watch to Mass Storage as a side effect).
#[tauri::command]
fn menu_load(state: State<'_, Demo>) -> Result<Vec<MenuSlot>, String> {
    let bytes = if let Some(root) = demo_root(&state) {
        // Demo mode: read the sample menu directly, no watch.
        std::fs::read(menu_file(&root)).map_err(|e| e.to_string())?
    } else {
        let port = serial::find_port().ok_or_else(|| "not_charging".to_string())?;
        serial::send(&port, SafeCommand::SaveMenu).map_err(|e| e.to_string())?;
        let root = mount_partition()?;
        let path = menu_file(&root);
        // The dump may take a moment to appear; retry briefly.
        let mut bytes = None;
        let deadline = Instant::now() + Duration::from_secs(10);
        while Instant::now() < deadline {
            if let Ok(b) = std::fs::read(&path) {
                if b.len() == watch_core::menu::MENU_SIZE {
                    bytes = Some(b);
                    break;
                }
            }
            std::thread::sleep(Duration::from_millis(500));
        }
        bytes.ok_or_else(|| "menu_not_found".to_string())?
    };
    let menu = Menu::parse(&bytes).map_err(|e| e.to_string())?;

    Ok(menu
        .used_slots()
        .into_iter()
        .map(|s| MenuSlot {
            app_id: s.app_id,
            hex: format!("0x{:X}", s.app_id),
            name: watch_core::appids::name_for(s.app_id)
                .unwrap_or("?")
                .to_string(),
            icon_id: s.icon_id,
            name_id: s.name_id,
            visible: s.flags == 1,
        })
        .collect())
}

/// Save an edited menu back to the watch. Backs up the current file first, then
/// writes the rebuilt 1600-byte file. The partition must already be mounted
/// (run menu_load first). Returns the backup file path. The user then restarts
/// the watch to apply it.
#[tauri::command]
fn menu_save(
    app: tauri::AppHandle,
    state: State<'_, Demo>,
    slots: Vec<SlotInput>,
) -> Result<String, String> {
    let root = demo_root(&state)
        .or_else(partition::find)
        .ok_or_else(|| "partition_not_mounted".to_string())?;
    let path = menu_file(&root);

    let built = Menu::from_slots(
        &slots
            .iter()
            .map(|s| Slot {
                app_id: s.app_id,
                icon_id: s.icon_id,
                name_id: s.name_id,
                flags: if s.visible { 1 } else { 0 },
            })
            .collect::<Vec<_>>(),
    )
    .map_err(|e| e.to_string())?;
    let out = built.to_bytes();

    // Back up the current file before overwriting.
    let backup_dir = app
        .path()
        .app_data_dir()
        .map_err(|e| e.to_string())?
        .join("menu-backups");
    std::fs::create_dir_all(&backup_dir).map_err(|e| e.to_string())?;
    let ts = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    let backup = backup_dir.join(format!("menu_list_{ts}.cfg"));
    if let Ok(cur) = std::fs::read(&path) {
        std::fs::write(&backup, &cur).map_err(|e| e.to_string())?;
    }

    std::fs::write(&path, out).map_err(|e| e.to_string())?;
    Ok(backup.to_string_lossy().into_owned())
}

/// Open the current partition folder in Windows File Explorer. `rel` is the
/// path relative to the partition root ("" for the root / drive).
#[tauri::command]
fn reveal_in_explorer(state: State<'_, Demo>, rel: String) -> Result<(), String> {
    let root = demo_root(&state)
        .or_else(partition::find)
        .ok_or_else(|| "partition_not_mounted".to_string())?;
    let mut p = root;
    for part in rel.split(['/', '\\']) {
        if part.is_empty() || part == "." {
            continue;
        }
        if part == ".." || part.contains(':') {
            return Err("bad_path".into());
        }
        p.push(part);
    }
    std::process::Command::new("explorer")
        .arg(&p)
        .spawn()
        .map_err(|e| e.to_string())?;
    Ok(())
}

pub fn run() {
    tauri::Builder::default()
        .manage(Demo::default())
        .invoke_handler(tauri::generate_handler![
            detect_connection,
            ping_watch,
            app_version,
            expose_partition,
            partition_list,
            menu_load,
            menu_save,
            enable_demo,
            disable_demo,
            reveal_in_explorer
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
