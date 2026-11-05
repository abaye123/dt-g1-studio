//! Access to the watch's hidden system partition (`/1:/`), a safe open
//! replacement for the vendor's closed `ser_set_real_u_py2.exe`.
//!
//! Flow: in Charging mode, send `AT+WRX=realUpan` then `AT+WRX=mass`; the OS
//! then mounts the two watch disks. The hidden partition is the one that
//! contains a `gui/apps` folder. This module exposes it and lists it read-only.

use std::path::{Path, PathBuf};

use crate::serial::{self, SafeCommand};
use crate::Error;

/// One directory entry.
#[derive(Clone, Debug)]
pub struct Entry {
    pub name: String,
    pub is_dir: bool,
    pub size: u64,
}

/// Send the two commands that expose the real disks. Requires Charging mode.
/// After this the serial port disappears (the watch switches to mass storage),
/// so the caller then polls [`find`] until the drive mounts.
pub fn expose() -> Result<(), Error> {
    let port = serial::find_port().ok_or_else(|| Error::Serial("not in charging mode".into()))?;
    serial::send(&port, SafeCommand::RealUpan)?;
    // `Mass` drops the serial port; its reply is empty by design.
    serial::send(&port, SafeCommand::Mass)?;
    Ok(())
}

/// Find the mounted hidden partition by its signature (`gui/apps`).
/// Scans drive letters D..Z (C is the system disk).
pub fn find() -> Option<PathBuf> {
    for letter in b'D'..=b'Z' {
        let root = PathBuf::from(format!("{}:\\", letter as char));
        if root.join("gui").join("apps").is_dir() {
            return Some(root);
        }
    }
    None
}

/// Find the mounted USER partition (the large FAT disk with a `MAP` folder,
/// where offline maps, music and photos live). It is distinct from the hidden
/// system partition (which has `gui/apps`). Mounted in normal Mass Storage mode.
pub fn find_user() -> Option<PathBuf> {
    for letter in b'D'..=b'Z' {
        let root = PathBuf::from(format!("{}:\\", letter as char));
        if root.join("gui").join("apps").is_dir() {
            continue; // that is the hidden partition
        }
        if root.join("MAP").is_dir() {
            return Some(root);
        }
    }
    None
}

/// Reject a relative path that could escape the partition root.
fn safe_join(root: &Path, rel: &str) -> Result<PathBuf, Error> {
    let mut p = root.to_path_buf();
    for part in rel.split(['/', '\\']) {
        if part.is_empty() || part == "." {
            continue;
        }
        if part == ".." || part.contains(':') {
            return Err(Error::Path(rel.to_string()));
        }
        p.push(part);
    }
    // The result must still be under root.
    if !p.starts_with(root) {
        return Err(Error::Path(rel.to_string()));
    }
    Ok(p)
}

/// List a directory under the partition root, read-only. `rel` is a path
/// relative to the root (use "" for the root itself). Folders first, then files,
/// each sorted by name.
pub fn list_dir(root: &Path, rel: &str) -> Result<Vec<Entry>, Error> {
    let dir = safe_join(root, rel)?;
    let mut out = Vec::new();
    for entry in std::fs::read_dir(&dir).map_err(|e| Error::Io(e.to_string()))? {
        let entry = entry.map_err(|e| Error::Io(e.to_string()))?;
        let meta = match entry.metadata() {
            Ok(m) => m,
            Err(_) => continue,
        };
        out.push(Entry {
            name: entry.file_name().to_string_lossy().into_owned(),
            is_dir: meta.is_dir(),
            size: if meta.is_dir() { 0 } else { meta.len() },
        });
    }
    out.sort_by(|a, b| b.is_dir.cmp(&a.is_dir).then(a.name.to_lowercase().cmp(&b.name.to_lowercase())));
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn safe_join_blocks_escape() {
        let root = Path::new("E:\\");
        assert!(safe_join(root, "..").is_err());
        assert!(safe_join(root, "gui/../..").is_err());
        assert!(safe_join(root, "C:\\Windows").is_err());
        assert_eq!(safe_join(root, "gui/apps").unwrap(), Path::new("E:\\gui\\apps"));
        assert_eq!(safe_join(root, "").unwrap(), Path::new("E:\\"));
    }
}
