//! Core logic for the DT G1 Studio tool.
//!
//! This crate holds the hardware-independent logic (menu file parsing, the
//! AppID map) and the serial AT channel to the watch. It is used by the Tauri
//! app, and its pure parts are unit-tested without any hardware.
//!
//! Safety: the serial layer exposes only an allow-list of read or reversible
//! mode-change commands. It never sends a flash write, erase, or partition
//! command.

pub mod appids;
pub mod menu;
pub mod partition;
pub mod serial;

/// Errors from the core logic.
#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("menu_list.cfg must be exactly 1600 bytes, got {0}")]
    MenuSize(usize),

    #[error("menu slot index {0} is out of range (0..100)")]
    MenuIndex(usize),

    #[error("serial error: {0}")]
    Serial(String),

    #[error("unsafe or invalid path: {0}")]
    Path(String),

    #[error("io error: {0}")]
    Io(String),
}

/// The library version, for the About screen.
pub const VERSION: &str = env!("CARGO_PKG_VERSION");
