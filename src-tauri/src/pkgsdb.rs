//! Reader/writer for the watch package database `gui/pkgs.db`.
//!
//! Despite the `.db` name this is a small JSON text file that registers the
//! installed applets so the firmware can launch them by `pkg://` uri. Verified
//! format (both stock-empty and gx-generated):
//!
//! ```json
//! { "apps": { "<package-id>": { "uri": "pkg://<package-id>" } }, "dials": {} }
//! ```
//!
//! There is no signature or checksum, so adding an entry is a plain JSON merge.

use serde_json::{json, Map, Value};
use std::path::{Path, PathBuf};

/// Path to `gui/pkgs.db` under a mounted hidden-partition root.
pub fn path(root: &Path) -> PathBuf {
    root.join("gui").join("pkgs.db")
}

/// Read the database, or return an empty default when the file is missing or
/// unparsable (the firmware logs "Failed to open package database" and boots
/// fine, so a fresh default is a safe baseline).
pub fn read(root: &Path) -> Value {
    match std::fs::read(path(root)) {
        Ok(bytes) => serde_json::from_slice(&bytes).unwrap_or_else(|_| empty()),
        Err(_) => empty(),
    }
}

fn empty() -> Value {
    json!({ "apps": {}, "dials": {} })
}

/// True if `package_id` is already registered under `apps`.
pub fn has_app(db: &Value, package_id: &str) -> bool {
    db.get("apps")
        .and_then(|a| a.as_object())
        .map(|o| o.contains_key(package_id))
        .unwrap_or(false)
}

/// Add (or overwrite) the registration for `package_id`. Returns the new db.
pub fn add_app(mut db: Value, package_id: &str) -> Value {
    ensure_shape(&mut db);
    if let Some(apps) = db.get_mut("apps").and_then(|a| a.as_object_mut()) {
        apps.insert(
            package_id.to_string(),
            json!({ "uri": format!("pkg://{package_id}") }),
        );
    }
    db
}

/// Remove the registration for `package_id` if present. Returns the new db.
#[allow(dead_code)]
pub fn remove_app(mut db: Value, package_id: &str) -> Value {
    ensure_shape(&mut db);
    if let Some(apps) = db.get_mut("apps").and_then(|a| a.as_object_mut()) {
        apps.remove(package_id);
    }
    db
}

/// Make sure the value has the expected `apps` and `dials` objects.
fn ensure_shape(db: &mut Value) {
    if !db.is_object() {
        *db = empty();
        return;
    }
    let obj = db.as_object_mut().unwrap();
    obj.entry("apps").or_insert_with(|| Value::Object(Map::new()));
    obj.entry("dials").or_insert_with(|| Value::Object(Map::new()));
}

/// Write the database back as pretty JSON (matching the gx-generated layout).
pub fn write(root: &Path, db: &Value) -> Result<(), String> {
    let gui = root.join("gui");
    std::fs::create_dir_all(&gui).map_err(|e| e.to_string())?;
    let text = serde_json::to_string_pretty(db).map_err(|e| e.to_string())?;
    std::fs::write(path(root), text.as_bytes()).map_err(|e| e.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn add_and_detect() {
        let db = add_app(empty(), "com.abaye.hello");
        assert!(has_app(&db, "com.abaye.hello"));
        assert_eq!(
            db["apps"]["com.abaye.hello"]["uri"],
            json!("pkg://com.abaye.hello")
        );
        assert!(db.get("dials").is_some());
    }

    #[test]
    fn repairs_broken_shape() {
        let db = add_app(json!("garbage"), "x");
        assert!(has_app(&db, "x"));
    }

    #[test]
    fn remove_is_idempotent() {
        let db = remove_app(empty(), "not.there");
        assert!(!has_app(&db, "not.there"));
    }
}
