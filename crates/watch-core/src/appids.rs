//! AppID -> name map for the menu editor.
//!
//! These names are reverse-engineered (from the community `AppID_mapping.csv`)
//! and are hints, not guaranteed labels. They can differ on a given firmware.
//! "(network)" means the screen needs a network connection; "(pkg)" means it
//! depends on an installed `.pkg` app.

/// Look up a display name for an AppID. Returns None if unknown.
pub fn name_for(app_id: u32) -> Option<&'static str> {
    MAP.iter().find(|(id, _)| *id == app_id).map(|(_, n)| *n)
}

/// The known AppIDs. Kept in sync with the community menu-editor mapping.
pub static MAP: &[(u32, &str)] = &[
    (0xC392, "Shutdown screen"),
    (0xC3A8, "Brightness"),
    (0xC3B6, "Device info"),
    (0xC3CE, "SOS"),
    (0xC40E, "Call screen"),
    (0xC43B, "Power-save settings"),
    (0xC444, "Vibration strength"),
    (0xC460, "Calculator"),
    (0xC461, "Sports list"),
    (0xC462, "Contacts"),
    (0xC463, "Stopwatch"),
    (0xC464, "Calendar"),
    (0xC465, "Settings"),
    (0xC466, "Flashlight"),
    (0xC467, "Sleep"),
    (0xC468, "Weather"),
    (0xC469, "Messages"),
    (0xC45F, "Dial pad"),
    (0xC45D, "Alarm clock"),
    (0xC46A, "Heart rate"),
    (0xC46B, "Blood pressure"),
    (0xC46C, "Blood oxygen"),
    (0xC47A, "History data"),
    (0xC48B, "Reboot"),
    (0xC48C, "Reset"),
    (0xC491, "USB mass storage mode"),
    (0xC4A8, "Idle clock style"),
    (0xC4BE, "Screen-off settings"),
    (0xC4BF, "Always-on time settings"),
    (0xC4DB, "Recorder"),
    (0xC4DF, "View mode settings"),
    (0xC4E4, "Timer"),
    (0xC581, "QR code"),
    (0xC584, "English teacher (network)"),
    (0xC5A3, "Recent calls"),
    (0xC5A7, "Always-on screen"),
    (0xC60A, "Music mode"),
    (0xC612, "Screen-lock password"),
    (0xC617, "Game: 2048"),
    (0xC619, "Game: Flappy Bird"),
    (0xC61B, "Game: Tower of Hanoi"),
    (0xC6F4, "Step detail"),
    (0xC71C, "Call reminder"),
    (0xC748, "Cube demo"),
    (0xC749, "UI effect settings"),
    (0xC759, "Video"),
    (0xC75A, "Power video"),
    (0xC760, "Dial market"),
    (0xC779, "System storage"),
    (0xC782, "NFC"),
    (0xC74F, "Compass"),
    (0xC804, "Map (network)"),
    (0xC824, "NetEase Cloud Music (pkg)"),
    (0xC825, "Himalaya (network)"),
    (0xC826, "Permanent calendar (pkg)"),
    (0xC827, "Weather tile (network)"),
    (0xC828, "Weibo hot search (network)"),
    (0xC829, "Sina News (network)"),
    (0xC82A, "Phoenix Finance (network)"),
    (0xC82B, "Buddha beads (pkg)"),
    (0xC82C, "Wooden Fish (pkg)"),
    (0xC82D, "Audio book (network)"),
    (0xC82E, "Audio blogs (network)"),
    (0xC82F, "Stock (network)"),
    (0xC839, "Decibel meter"),
    (0xC83C, "World time"),
    (0xC840, "Wallet"),
    (0xC841, "Period tracking"),
    (0xC843, "Translate (network)"),
    (0xC92B, "Travel (network)"),
    (0xC92C, "Movie (network)"),
    (0xC92D, "History (network)"),
    (0xC940, "BBC (network)"),
    (0xC941, "ChatGPT (network)"),
    (0xC9BA, "App market (network)"),
    (0xC9EE, "Games"),
    (0xC9F7, "Screen off"),
    (0xC9F8, "Sport courses"),
    (0xCA1B, "Gesture control"),
    (0xCA27, "Somatosensory sports"),
    (0xCA70, "Font size"),
    (0xCA72, "Unknown (0xCA72)"),
    (0xCA8A, "Life camera"),
    (0xCA96, "Breathe detail"),
    (0xCAA2, "Picture"),
    (0xCAA3, "GPS demo video"),
    (0xCAAE, "Offline map"),
    (0xCAB4, "Mobile voice assistant"),
    (0xCAB5, "Music player"),
    (0xCAB8, "Business cards"),
    (0xCADB, "Pressure"),
    (0xCB39, "Altitude / barometer"),
    (0xCB42, "Watch tips"),
    (0xC439, "Find device"),
];

/// A stock `.pkg`-based menu slot a custom app can take over. Each tuple is
/// (AppID, stock package id, English label). The firmware binds the AppID to
/// this fixed package name, so a custom app that reuses the slot must be built
/// with this exact package id and placed at `gui/apps/<package-id>.pkg`.
///
/// Only the local (non-network) slots are listed: these launch purely from the
/// package on disk. Source: the community `AppID_mapping.csv`.
pub struct PkgSlot {
    pub app_id: u32,
    pub package_id: &'static str,
    pub label: &'static str,
}

/// The stock slots a custom app may replace. `com.xfaith.woodenfish` (Wooden
/// Fish, AppID 0xC82C) is also the slot the PurrLaunch launcher takes over.
pub static PKG_SLOTS: &[PkgSlot] = &[
    PkgSlot { app_id: 0xC82C, package_id: "com.xfaith.woodenfish", label: "Wooden Fish" },
    PkgSlot { app_id: 0xC826, package_id: "com.xfaith.calendar", label: "Permanent calendar" },
    PkgSlot { app_id: 0xC82B, package_id: "com.xfaith.buddhabeads", label: "Buddha beads" },
    PkgSlot { app_id: 0xC824, package_id: "com.xfaith.netease-cloud-music", label: "NetEase Cloud Music" },
];

/// The AppID of the slot the launcher takes over (Wooden Fish).
pub const LAUNCHER_APP_ID: u32 = 0xC82C;
/// The package id the launcher must be installed as.
pub const LAUNCHER_PACKAGE_ID: &str = "com.xfaith.woodenfish";

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn known_ids_resolve() {
        assert_eq!(name_for(0xC759), Some("Video"));
        assert_eq!(name_for(0xC9EE), Some("Games"));
        assert_eq!(name_for(0xCAA2), Some("Picture"));
        assert_eq!(name_for(0x0000), None);
    }
}
