//! Tauri shell for DT G1 Studio. Exposes a small set of commands over
//! `watch-core` to the web UI. Only read or reversible mode-change actions.

use std::path::{Path, PathBuf};
use std::sync::Mutex;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

mod mapcompile;

use base64::Engine as _;
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
    // Sample offline maps (so the Maps panel is previewable in demo mode).
    ("MAP/map/Tel_Aviv/roads.mlp", 234000),
    ("MAP/map/Tel_Aviv/roads.idx", 88000),
    ("MAP/map/Tel_Aviv/landuse.db", 41000),
    ("MAP/map/Tel_Aviv/water.mlp", 2700),
    ("MAP/map/Tel_Aviv/map.name", 0),
    ("MAP/map/Jerusalem_OSM/roads.mlp", 310000),
    ("MAP/map/Jerusalem_OSM/roads.idx", 120000),
    ("MAP/map/Jerusalem_OSM/landuse.db", 52000),
    ("MAP/map/Jerusalem_OSM/map.name", 0),
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
        if p.file_name().and_then(|n| n.to_str()) == Some("map.name") {
            // Give map.name a readable display name (from the parent folder).
            let disp = p
                .parent()
                .and_then(|d| d.file_name())
                .map(|n| n.to_string_lossy().replace('_', " "))
                .unwrap_or_default();
            std::fs::write(&p, disp.as_bytes())?;
        } else {
            std::fs::write(&p, vec![0u8; *size as usize])?;
        }
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

/// Render an input image to a 466x466 JPEG. `fit` is "cover" (crop to fill) or
/// "contain" (fit inside, pad with black).
fn render_square(input: &[u8], fit: &str) -> Result<Vec<u8>, String> {
    use image::imageops::FilterType;
    let side = watch_core::background::SIDE;
    let img = image::load_from_memory(input).map_err(|e| e.to_string())?;
    let out = if fit == "contain" {
        let resized = img.resize(side, side, FilterType::Lanczos3);
        let mut canvas = image::RgbImage::from_pixel(side, side, image::Rgb([0, 0, 0]));
        let x = ((side - resized.width()) / 2) as i64;
        let y = ((side - resized.height()) / 2) as i64;
        image::imageops::overlay(&mut canvas, &resized.to_rgb8(), x, y);
        canvas
    } else {
        img.resize_to_fill(side, side, FilterType::Lanczos3).to_rgb8()
    };
    let mut buf = Vec::new();
    image::codecs::jpeg::JpegEncoder::new_with_quality(&mut buf, 90)
        .encode(out.as_raw(), side, side, image::ExtendedColorType::Rgb8)
        .map_err(|e| e.to_string())?;
    Ok(buf)
}

/// Render a preview of a background and return it as a base64 JPEG (data for an
/// <img> tag). Does not touch the watch.
#[tauri::command]
fn preview_background(image_b64: String, fit: String) -> Result<String, String> {
    let input = base64::engine::general_purpose::STANDARD
        .decode(image_b64)
        .map_err(|e| e.to_string())?;
    let jpeg = render_square(&input, &fit)?;
    Ok(base64::engine::general_purpose::STANDARD.encode(jpeg))
}

/// Write a custom clock background to the watch (res/user_clock_bg.bin and the
/// thumbnail). The partition must be mounted (or demo mode). Returns nothing.
#[tauri::command]
fn apply_background(state: State<'_, Demo>, image_b64: String, fit: String) -> Result<(), String> {
    let root = demo_root(&state)
        .or_else(partition::find)
        .ok_or_else(|| "partition_not_mounted".to_string())?;
    let input = base64::engine::general_purpose::STANDARD
        .decode(image_b64)
        .map_err(|e| e.to_string())?;
    let jpeg = render_square(&input, &fit)?;
    let file = watch_core::background::wrap_jpeg(&jpeg);
    let res_dir = root.join("res");
    std::fs::create_dir_all(&res_dir).map_err(|e| e.to_string())?;
    std::fs::write(res_dir.join("user_clock_bg.bin"), &file).map_err(|e| e.to_string())?;
    std::fs::write(res_dir.join("user_clock_thumb.bin"), &file).map_err(|e| e.to_string())?;
    Ok(())
}

/// Write an already-rendered 466x466 JPEG (from the editor canvas) as the watch
/// background. This is the WYSIWYG path: the canvas is the exact output.
#[tauri::command]
fn apply_background_jpeg(state: State<'_, Demo>, jpeg_b64: String) -> Result<(), String> {
    let root = demo_root(&state)
        .or_else(partition::find)
        .ok_or_else(|| "partition_not_mounted".to_string())?;
    let jpeg = base64::engine::general_purpose::STANDARD
        .decode(jpeg_b64)
        .map_err(|e| e.to_string())?;
    if jpeg.len() < 4 || jpeg[0..3] != [0xFF, 0xD8, 0xFF] {
        return Err("not_jpeg".into());
    }
    let file = watch_core::background::wrap_jpeg(&jpeg);
    let res_dir = root.join("res");
    std::fs::create_dir_all(&res_dir).map_err(|e| e.to_string())?;
    std::fs::write(res_dir.join("user_clock_bg.bin"), &file).map_err(|e| e.to_string())?;
    std::fs::write(res_dir.join("user_clock_thumb.bin"), &file).map_err(|e| e.to_string())?;
    Ok(())
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

#[derive(Serialize)]
struct MapInfo {
    name: String,
    display: String,
    size: u64,
}

/// The MAP/map directory on the user partition (or the demo tree).
fn maps_dir(state: &State<'_, Demo>) -> Option<PathBuf> {
    if let Some(d) = demo_root(state) {
        return Some(d.join("MAP").join("map"));
    }
    partition::find_user().map(|r| r.join("MAP").join("map"))
}

fn dir_size(p: &Path) -> u64 {
    let mut total = 0;
    if let Ok(rd) = std::fs::read_dir(p) {
        for e in rd.flatten() {
            let path = e.path();
            if path.is_dir() {
                total += dir_size(&path);
            } else if let Ok(m) = e.metadata() {
                total += m.len();
            }
        }
    }
    total
}

fn safe_name(name: &str) -> bool {
    !name.is_empty()
        && !name.contains("..")
        && !name.contains('/')
        && !name.contains('\\')
        && !name.contains(':')
}

/// List the offline maps installed on the watch (each folder under MAP/map).
#[tauri::command]
fn list_maps(state: State<'_, Demo>) -> Result<Vec<MapInfo>, String> {
    let dir = maps_dir(&state).ok_or_else(|| "user_partition_not_mounted".to_string())?;
    let mut out = Vec::new();
    if let Ok(rd) = std::fs::read_dir(&dir) {
        for e in rd.flatten() {
            let path = e.path();
            if !path.is_dir() {
                continue;
            }
            let name = e.file_name().to_string_lossy().into_owned();
            let display = std::fs::read(path.join("map.name"))
                .ok()
                .and_then(|b| String::from_utf8(b).ok())
                .map(|s| s.lines().next().unwrap_or("").trim().to_string())
                .filter(|s| !s.is_empty())
                .unwrap_or_else(|| name.clone());
            out.push(MapInfo { name, display, size: dir_size(&path) });
        }
    }
    out.sort_by(|a, b| a.display.to_lowercase().cmp(&b.display.to_lowercase()));
    Ok(out)
}

/// Delete one installed map folder. Backs up nothing (maps are reproducible).
#[tauri::command]
fn delete_map(state: State<'_, Demo>, name: String) -> Result<(), String> {
    if !safe_name(&name) {
        return Err("bad_name".into());
    }
    let dir = maps_dir(&state).ok_or_else(|| "user_partition_not_mounted".to_string())?;
    let target = dir.join(&name);
    if !target.starts_with(&dir) || !target.is_dir() {
        return Err("not_found".into());
    }
    std::fs::remove_dir_all(&target).map_err(|e| e.to_string())?;
    Ok(())
}

/// Open the MAP/map folder in Windows File Explorer (creating it if needed), so
/// the user can drop compiled map folders in.
#[tauri::command]
fn reveal_maps(state: State<'_, Demo>) -> Result<(), String> {
    let dir = maps_dir(&state).ok_or_else(|| "user_partition_not_mounted".to_string())?;
    std::fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
    std::process::Command::new("explorer")
        .arg(&dir)
        .spawn()
        .map_err(|e| e.to_string())?;
    Ok(())
}

#[derive(Serialize)]
struct FileSize {
    name: String,
    size: u64,
}

#[derive(Serialize)]
struct CompileResult {
    osm_bytes: u64,
    total: u64,
    files: Vec<FileSize>,
}

fn app_dir(app: &tauri::AppHandle, sub: &str) -> Result<PathBuf, String> {
    let d = app.path().app_data_dir().map_err(|e| e.to_string())?.join(sub);
    Ok(d)
}

/// Is the OSM map compiler downloaded and ready?
#[tauri::command]
fn map_compiler_ready(app: tauri::AppHandle) -> bool {
    app_dir(&app, "map-compiler")
        .map(|d| mapcompile::is_ready(&d))
        .unwrap_or(false)
}

/// Download the official dtg1-map-tools compiler (one time, from its GitHub
/// release). About 13 MB. Returns when ready.
#[tauri::command]
fn map_compiler_setup(app: tauri::AppHandle) -> Result<(), String> {
    let dir = app_dir(&app, "map-compiler")?;
    mapcompile::setup(&dir)
}

/// Download OSM data for a bounding box and compile it to a watch map, staged
/// locally. Returns the OSM size and the produced map files with sizes.
#[tauri::command]
fn compile_region(
    app: tauri::AppHandle,
    west: f64,
    south: f64,
    east: f64,
    north: f64,
    name: String,
    poi_mode: String,
) -> Result<CompileResult, String> {
    if !safe_name(&name) {
        return Err("bad_name".into());
    }
    let poi = match poi_mode.as_str() {
        "native" | "landuse" | "none" => poi_mode.as_str(),
        _ => "landuse",
    };
    let compiler_dir = app_dir(&app, "map-compiler")?;
    if !mapcompile::is_ready(&compiler_dir) {
        return Err("compiler_not_ready".into());
    }
    let osm = app_dir(&app, "tmp")?.join("map.osm");
    let osm_bytes = mapcompile::download_osm(west, south, east, north, &osm)?;
    let staging = app_dir(&app, "map-staging")?.join(&name);
    let files = mapcompile::compile(&compiler_dir, &osm, &staging, &name, poi)?;
    let _ = std::fs::remove_file(&osm);
    let total = files.iter().map(|(_, s)| *s).sum();
    Ok(CompileResult {
        osm_bytes,
        total,
        files: files.into_iter().map(|(name, size)| FileSize { name, size }).collect(),
    })
}

/// Copy a staged compiled map onto the watch (MAP/map/<name>).
#[tauri::command]
fn install_compiled_map(
    app: tauri::AppHandle,
    state: State<'_, Demo>,
    name: String,
) -> Result<(), String> {
    if !safe_name(&name) {
        return Err("bad_name".into());
    }
    let staging = app_dir(&app, "map-staging")?.join(&name);
    if !staging.is_dir() {
        return Err("not_compiled".into());
    }
    let dir = maps_dir(&state).ok_or_else(|| "user_partition_not_mounted".to_string())?;
    let dest = dir.join(&name);
    std::fs::create_dir_all(&dest).map_err(|e| e.to_string())?;
    for entry in std::fs::read_dir(&staging).map_err(|e| e.to_string())? {
        let entry = entry.map_err(|e| e.to_string())?;
        if entry.path().is_file() {
            std::fs::copy(entry.path(), dest.join(entry.file_name())).map_err(|e| e.to_string())?;
        }
    }
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
            reveal_in_explorer,
            preview_background,
            apply_background,
            apply_background_jpeg,
            list_maps,
            delete_map,
            reveal_maps,
            map_compiler_ready,
            map_compiler_setup,
            compile_region,
            install_compiled_map
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
