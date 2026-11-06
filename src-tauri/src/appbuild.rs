//! Build simple Glyphix apps for the watch from fill-in templates, and fetch the
//! PurrLaunch launcher. The output is a `bundle.pkg` the Tauri layer installs
//! onto the hidden partition (copy to `gui/apps`, register in `pkgs.db`, point a
//! menu slot) - the same stage-then-copy shape as the map compiler.
//!
//! The heavy lifting (JS bytecode, UX compilation, PGF images) is done by the
//! official `gx` toolchain; we only generate the project tree and shell out.

use std::io::Read as _;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

/// Where the Glyphix toolchain lives (installed once, locally). `gx` and
/// `image-forge` are under `<root>/node_modules/.bin`, which must be on PATH
/// for a build (the build calls `image-forge` by bare name).
const TOOLCHAIN_ROOT: &str = r"C:\dev\dt-g1-watch\dt-g1-research\tools\glyphix-toolchain";
const DEVICE: &str = "dtg1-c175r";

/// PurrLaunch (purrrock, MIT) - a launcher/package-manager built for this watch.
/// Its release zip ships a ready `com.xfaith.woodenfish.pkg`.
const PURRLAUNCH_RELEASE: &str =
    "https://api.github.com/repos/purrrock/PurrLaunch/releases/latest";
const UA: &str = "dt-g1-studio";

/// The `node_modules/.bin` directory of the toolchain.
fn toolchain_bin() -> PathBuf {
    Path::new(TOOLCHAIN_ROOT).join("node_modules").join(".bin")
}

/// The gx launcher script (a Windows `.cmd` wrapper).
fn gx_cmd() -> PathBuf {
    toolchain_bin().join("gx.cmd")
}

/// Is the toolchain present and runnable?
pub fn gx_ready() -> bool {
    gx_cmd().is_file() && toolchain_bin().join("image-forge.cmd").is_file()
}

/// A human-readable toolchain path for the UI.
pub fn gx_location() -> String {
    gx_cmd().to_string_lossy().into_owned()
}

/// An HTTP agent that uses the OS certificate store, like the map compiler, so
/// corporate/antivirus TLS inspection is trusted the way the system trusts it.
fn agent() -> Result<ureq::Agent, String> {
    let connector = native_tls::TlsConnector::new().map_err(|e| e.to_string())?;
    Ok(ureq::AgentBuilder::new()
        .tls_connector(std::sync::Arc::new(connector))
        .timeout(std::time::Duration::from_secs(180))
        .build())
}

// ---------------------------------------------------------------------------
// Templates
// ---------------------------------------------------------------------------

/// A field the UI collects for a template.
pub struct Field {
    pub key: &'static str,
    pub kind: &'static str, // "text" | "multiline" | "list" | "color" | "phone"
    pub he: &'static str,
    pub en: &'static str,
}

/// A template definition shown in the gallery.
pub struct Template {
    pub id: &'static str,
    pub he: &'static str,
    pub en: &'static str,
    pub desc_he: &'static str,
    pub desc_en: &'static str,
    pub fields: &'static [Field],
}

pub static TEMPLATES: &[Template] = &[
    Template {
        id: "info_card",
        he: "כרטיס מידע",
        en: "Info card",
        desc_he: "מסך תצוגה: כותרת, שורות, שם מודגש וטלפון. למשל 'למי שייך השעון'.",
        desc_en: "A display screen: title, lines, a highlighted name and a phone.",
        fields: &[
            Field { key: "title", kind: "text", he: "כותרת", en: "Title" },
            Field { key: "line1", kind: "text", he: "שורה 1", en: "Line 1" },
            Field { key: "line2", kind: "text", he: "שורה 2", en: "Line 2" },
            Field { key: "name", kind: "text", he: "שם מודגש", en: "Highlighted name" },
            Field { key: "phone", kind: "phone", he: "טלפון", en: "Phone" },
            Field { key: "bg", kind: "color", he: "צבע רקע", en: "Background" },
            Field { key: "accent", kind: "color", he: "צבע הדגשה", en: "Accent" },
        ],
    },
    Template {
        id: "text_reader",
        he: "קורא טקסט",
        en: "Text reader",
        desc_he: "מסך גלילה לטקסט ארוך בעברית: תהילים, ברכות, תפילת הדרך.",
        desc_en: "A scrolling screen for long Hebrew text: psalms, blessings, prayers.",
        fields: &[
            Field { key: "title", kind: "text", he: "כותרת", en: "Title" },
            Field { key: "body", kind: "multiline", he: "טקסט", en: "Body text" },
            Field { key: "bg", kind: "color", he: "צבע רקע", en: "Background" },
            Field { key: "accent", kind: "color", he: "צבע הדגשה", en: "Accent" },
        ],
    },
    Template {
        id: "list",
        he: "רשימה",
        en: "List",
        desc_he: "רשימת פריטים ניתנת לגלילה: מטלות, מספרי טלפון, רשימת קניות.",
        desc_en: "A scrollable list of items: tasks, phone numbers, a shopping list.",
        fields: &[
            Field { key: "title", kind: "text", he: "כותרת", en: "Title" },
            Field { key: "items", kind: "list", he: "פריטים (שורה לכל פריט)", en: "Items (one per line)" },
            Field { key: "bg", kind: "color", he: "צבע רקע", en: "Background" },
            Field { key: "accent", kind: "color", he: "צבע הדגשה", en: "Accent" },
        ],
    },
];

pub fn template(id: &str) -> Option<&'static Template> {
    TEMPLATES.iter().find(|t| t.id == id)
}

// ---------------------------------------------------------------------------
// Project generation
// ---------------------------------------------------------------------------

fn str_field(fields: &serde_json::Value, key: &str) -> String {
    fields.get(key).and_then(|v| v.as_str()).unwrap_or("").trim().to_string()
}

/// Validate a `#rrggbb` color, falling back to `default` on anything else, so a
/// stray value can never break the generated CSS.
fn color(fields: &serde_json::Value, key: &str, default: &str) -> String {
    let v = str_field(fields, key);
    let ok = v.len() == 7
        && v.starts_with('#')
        && v[1..].chars().all(|c| c.is_ascii_hexdigit());
    if ok {
        v
    } else {
        default.to_string()
    }
}

/// JSON-encode a value for embedding inside a `<script>` block. The `</` guard
/// stops user text that contains `</script>` from closing the tag early.
fn js_json(v: &serde_json::Value) -> String {
    serde_json::to_string(v).unwrap_or_else(|_| "null".into()).replace("</", "<\\/")
}

/// Split a multiline textarea into non-empty trimmed lines.
fn lines(s: &str) -> Vec<String> {
    s.lines().map(|l| l.trim_end().to_string()).collect()
}

/// A monotonically increasing version code (seconds since 2020), so reinstalls
/// always count as an upgrade.
fn version_code() -> u64 {
    let now = SystemTime::now().duration_since(UNIX_EPOCH).map(|d| d.as_secs()).unwrap_or(0);
    now.saturating_sub(1_577_836_800) // 2020-01-01
}

fn manifest(package_id: &str, name: &str) -> String {
    let name = if name.is_empty() { "App" } else { name };
    let name = js_json(&serde_json::Value::String(name.to_string()));
    format!(
        r#"{{
  "package": "{package_id}",
  "name": {name},
  "icon": "/assets/icon.png",
  "versionName": "1.0.0",
  "versionCode": {code},
  "features": [],
  "router": {{ "entry": "main", "pages": {{ "main": {{ "component": "index" }} }} }},
  "config": {{ "designWidth": 466 }}
}}
"#,
        code = version_code()
    )
}

/// Build the `index.ux` source for a template. Colors are baked into the CSS;
/// all user text goes into the `data` object and is interpolated, so markup can
/// never be broken by the content.
fn index_ux(template_id: &str, fields: &serde_json::Value) -> String {
    let bg = color(fields, "bg", "#101322");
    let accent = color(fields, "accent", "#ffd65a");
    let title = str_field(fields, "title");

    match template_id {
        "text_reader" => {
            let data = serde_json::json!({
                "title": title,
                "paras": lines(&str_field(fields, "body")),
            });
            format!(
                r#"<template>
  <scroll class="page">
    <div class="pad"></div>
    <text class="title" if="{{{{title}}}}">{{{{title}}}}</text>
    <text class="para" for="p in paras">{{{{p}}}}</text>
    <div class="pad"></div>
  </scroll>
</template>
<style>
.page {{ width: 466px; height: 466px; background-color: {bg}; flex-direction: column; align-items: center; }}
.pad {{ width: 10px; height: 64px; }}
.title {{ font-size: 34px; color: {accent}; text-align: center; direction: rtl; margin-bottom: 16px; }}
.para {{ width: 380px; font-size: 28px; color: #ffffff; text-align: center; direction: rtl; margin-bottom: 10px; }}
</style>
<script>
export default {{ data: {data} }}
</script>
"#,
                data = js_json(&data)
            )
        }
        "list" => {
            let data = serde_json::json!({
                "title": title,
                "items": lines(&str_field(fields, "items")).into_iter().filter(|l| !l.is_empty()).collect::<Vec<_>>(),
            });
            format!(
                r#"<template>
  <scroll class="page">
    <div class="pad"></div>
    <text class="title" if="{{{{title}}}}">{{{{title}}}}</text>
    <div class="row" for="it in items">
      <text class="row-t">{{{{it}}}}</text>
    </div>
    <div class="pad"></div>
  </scroll>
</template>
<style>
.page {{ width: 466px; height: 466px; background-color: {bg}; flex-direction: column; align-items: center; }}
.pad {{ width: 10px; height: 56px; }}
.title {{ font-size: 32px; color: {accent}; text-align: center; direction: rtl; margin-bottom: 14px; }}
.row {{ width: 360px; min-height: 68px; margin-bottom: 8px; border-radius: 34px; background-color: #2a2250; justify-content: center; align-items: center; }}
.row-t {{ width: 320px; font-size: 28px; color: #ffffff; text-align: center; direction: rtl; }}
</style>
<script>
export default {{ data: {data} }}
</script>
"#,
                data = js_json(&data)
            )
        }
        // info_card (default)
        _ => {
            let infos: Vec<String> = [str_field(fields, "line1"), str_field(fields, "line2")]
                .into_iter()
                .filter(|l| !l.is_empty())
                .collect();
            let data = serde_json::json!({
                "title": title,
                "lines": infos,
                "name": str_field(fields, "name"),
                "phone": str_field(fields, "phone"),
            });
            format!(
                r#"<template>
  <div class="page">
    <text class="title" if="{{{{title}}}}">{{{{title}}}}</text>
    <text class="line" for="l in lines">{{{{l}}}}</text>
    <div class="badge" if="{{{{name}}}}">
      <text class="badge-t">{{{{name}}}}</text>
    </div>
    <text class="phone" if="{{{{phone}}}}">{{{{phone}}}}</text>
  </div>
</template>
<style>
.page {{ width: 466px; height: 466px; background-color: {bg}; flex-direction: column; justify-content: center; align-items: center; }}
.title {{ font-size: 40px; color: {accent}; text-align: center; direction: rtl; margin-bottom: 18px; }}
.line {{ width: 400px; font-size: 28px; color: #ffffff; text-align: center; direction: rtl; margin-bottom: 8px; }}
.badge {{ margin-top: 10px; padding: 10px 28px; border-radius: 40px; background-color: {accent}; justify-content: center; align-items: center; }}
.badge-t {{ font-size: 34px; color: {bg}; text-align: center; direction: rtl; }}
.phone {{ font-size: 36px; color: #ffffff; text-align: center; direction: ltr; margin-top: 16px; }}
</style>
<script>
export default {{ data: {data} }}
</script>
"#,
                data = js_json(&data)
            )
        }
    }
}

/// Write the project source tree under `project_dir/src`. `icon_png` is a
/// 192x192 PNG (rendered by the UI canvas, so it carries real fonts/glyphs).
/// Returns the project directory.
pub fn generate_project(
    project_dir: &Path,
    template_id: &str,
    package_id: &str,
    name: &str,
    fields: &serde_json::Value,
    icon_png: &[u8],
) -> Result<(), String> {
    if project_dir.exists() {
        std::fs::remove_dir_all(project_dir).ok();
    }
    let src = project_dir.join("src");
    let main = src.join("main");
    let assets = src.join("assets");
    std::fs::create_dir_all(&main).map_err(|e| e.to_string())?;
    std::fs::create_dir_all(&assets).map_err(|e| e.to_string())?;

    std::fs::write(src.join("manifest.json"), manifest(package_id, name))
        .map_err(|e| e.to_string())?;
    std::fs::write(src.join("app.js"), "export default { onCreate() {} }\n")
        .map_err(|e| e.to_string())?;
    std::fs::write(main.join("index.ux"), index_ux(template_id, fields))
        .map_err(|e| e.to_string())?;
    std::fs::write(assets.join("icon.png"), icon_png).map_err(|e| e.to_string())?;
    Ok(())
}

/// The `bundle.pkg` path `gx build` produces for a package under `project_dir`.
pub fn dist_bundle(project_dir: &Path, package_id: &str) -> PathBuf {
    project_dir
        .join(".glyphix-work")
        .join("dist")
        .join(DEVICE)
        .join(package_id)
        .join("bundle.pkg")
}

/// Run `gx build` on a generated project and return the produced `bundle.pkg`
/// path and its size. The toolchain `.bin` is put on PATH so the build's
/// `image-forge`/`pngquant` calls resolve.
pub fn build(project_dir: &Path, package_id: &str) -> Result<(PathBuf, u64), String> {
    if !gx_ready() {
        return Err("gx_not_ready".into());
    }
    let bin = toolchain_bin();
    let path_env = match std::env::var("PATH") {
        Ok(p) => format!("{};{}", bin.to_string_lossy(), p),
        Err(_) => bin.to_string_lossy().into_owned(),
    };

    let out = std::process::Command::new("cmd")
        .arg("/C")
        .arg("gx")
        .arg("build")
        .arg("-d")
        .arg(DEVICE)
        .arg("-f")
        .current_dir(project_dir)
        .env("PATH", path_env)
        .output()
        .map_err(|e| format!("run gx: {e}"))?;

    if !out.status.success() {
        let log = String::from_utf8_lossy(&out.stdout);
        let errlog = String::from_utf8_lossy(&out.stderr);
        let tail: String = log
            .lines()
            .chain(errlog.lines())
            .filter(|l| !l.trim().is_empty())
            .rev()
            .take(5)
            .collect::<Vec<_>>()
            .into_iter()
            .rev()
            .collect::<Vec<_>>()
            .join(" | ");
        return Err(format!("build_failed: {tail}"));
    }

    let pkg = dist_bundle(project_dir, package_id);
    if !pkg.is_file() {
        return Err("bundle_not_produced".into());
    }
    let size = pkg.metadata().map(|m| m.len()).unwrap_or(0);
    Ok((pkg, size))
}

// ---------------------------------------------------------------------------
// Launcher (PurrLaunch)
// ---------------------------------------------------------------------------

/// Local path where the downloaded launcher package is cached.
pub fn launcher_pkg(dir: &Path) -> PathBuf {
    dir.join("com.xfaith.woodenfish.pkg")
}

pub fn launcher_ready(dir: &Path) -> bool {
    launcher_pkg(dir).is_file()
}

/// Download the latest PurrLaunch release and extract its `.pkg` into `dir`.
pub fn setup_launcher(dir: &Path) -> Result<(), String> {
    let http = agent()?;
    let meta = http
        .get(PURRLAUNCH_RELEASE)
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

    let mut buf = Vec::new();
    http.get(&url)
        .set("User-Agent", UA)
        .call()
        .map_err(|e| format!("download failed: {e}"))?
        .into_reader()
        .read_to_end(&mut buf)
        .map_err(|e| e.to_string())?;

    std::fs::create_dir_all(dir).map_err(|e| e.to_string())?;
    let mut zip = zip::ZipArchive::new(std::io::Cursor::new(buf)).map_err(|e| e.to_string())?;
    let mut found = false;
    for i in 0..zip.len() {
        let mut f = zip.by_index(i).map_err(|e| e.to_string())?;
        if f.is_dir() {
            continue;
        }
        let base = match f.enclosed_name().and_then(|p| p.file_name().map(|n| n.to_owned())) {
            Some(n) => n,
            None => continue,
        };
        if base.to_string_lossy().to_lowercase().ends_with(".pkg") {
            let mut outf = std::fs::File::create(launcher_pkg(dir)).map_err(|e| e.to_string())?;
            std::io::copy(&mut f, &mut outf).map_err(|e| e.to_string())?;
            found = true;
            break;
        }
    }
    if !found {
        return Err("no pkg in launcher release".into());
    }
    Ok(())
}
