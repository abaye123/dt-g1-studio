# DT G1 Studio - design and roadmap

This document is the plan. It records the chosen architecture, the watch operations the tool wraps, and a summary of the Glyphix app API for the future app builder. The underlying research is in the sibling `dt-g1-research` repo (`docs/DT-G1-notes.en.md` / `.he.md`).

## 1. Goals

- One application, not several tools, with a guided wizard for each task.
- Bilingual Hebrew and English, correct right-to-left layout.
- Safe by default: read-only or clearly-marked mode changes; writes are explicit and backed up; flash-erase and partition commands are never exposed.
- Usable by a non-expert: it explains each physical step (dock, USB mode, reconnect).

## 2. Technology

- Tauri (v2): a Rust backend with a web UI in a WebView2 window.
  - Why: the web layer handles Hebrew and RTL perfectly, wizards are easy to build and style, and the same codebase signs with the existing code-signing certificate.
- Rust backend crates:
  - `serialport` - the AT command channel to the watch (Charging mode, `VID_10D6&PID_B00A`, 2000000 baud, payload suffix `00 64 00 61`).
  - `nusb` or `rusb` - USB enumeration and, on Linux, the ADFU reader.
  - `tauri` - the shell, commands, and events to the UI.
- Frontend: a small framework or plain TypeScript. Keep it light. i18n with a simple message map and `dir="rtl"` toggling.

## 3. Repository layout (planned)

```
dt-g1-studio/
  src-tauri/            # Rust backend
    src/
      main.rs
      watch/            # device logic (modular)
        serial.rs       # find port, send AT, mode transitions
        partition.rs    # expose and read partition 2
        menu.rs         # parse/edit menu_list.cfg (1600 bytes)
        appids.rs       # AppID -> name map
      commands.rs       # Tauri command handlers the UI calls
    tauri.conf.json
    Cargo.toml
  src/                  # web UI (wizards, editor, i18n)
  package.json
```

The device logic is split by concern so each feature is independent, while the app ships as one binary.

## 4. Watch operations to wrap (from the research)

All of these are proven in the `dt-g1-research` repo's PowerShell/Python tools; the tool reimplements them in Rust.

### 4.1 Connection and USB modes
- Charging mode: `VID_10D6&PID_B00A`, exposes a serial COM port. Needed to send AT commands.
- Mass storage: `VID_10D6&PID_B00B`, one disk (user storage).
- Real disks: after `AT+WRX=realUpan` then `AT+WRX=mass`, two disks appear (user + hidden `/1:/`).
- The USB mode returns to Mass Storage after every restart, so the wizard always re-checks and guides the user to set Charging mode.

### 4.2 AT channel
- Serial 2000000 8N1, DTR+RTS on. Every command is ASCII text followed by the bytes `00 64 00 61`.
- Safe commands used: `AT+WRX=always` (ping), `AT+WRX=realUpan`, `AT+WRX=mass`, `AT+WRX=saveMenu` (writes `/1:/custom/menu_list.cfg`).
- Never send: `partUpan`, `mergeUpan`, `testWrite`, `testReplaceData`, `delClock`, `clearInfo`, `ota`, `ramdump`, ADFU flash-with-erase.

### 4.3 Hidden partition browser
- Send `realUpan` + `mass`, then the OS mounts partition 2. Read it (gui/, data/, res/, ring/, custom/, logs/). Present a read-only file tree, with explicit opt-in for any write.
- Do not surface `data/mass/APIConfig.json` contents (vendor secret keys).

### 4.4 Menu editor (`menu_list.cfg`)
- Exactly 1600 bytes, 100 slots of 16 bytes, little-endian: `AppID`, `IconID`, `NameID`, `Flags` (1 visible, 0 hidden).
- Flow: `AT+WRX=saveMenu` writes the current menu to `/1:/custom/menu_list.cfg`; read it, show a visual list (names from the AppID map), let the user hide/reorder/show, validate the size stays 1600, back up, write it back, prompt a restart.
- Risk: a corrupt file can cause a boot loop. The editor only edits known fields and keeps the byte length fixed; it never writes raw bytes the user typed.
- Keep a bundled `AppID_mapping.csv` (from the community menu editor) for names; allow it to be updated.

### 4.5 ADFU dump (advanced, Linux only)
- `AT+WRX=adfu` enters boot-ROM download mode (`10d6:10d6`). Read-only memory read is CBW `0xCD 0x93`; flash is XIP at `0x10000000`.
- On Windows the 6-second USB timeout and the WinUSB bind cannot be landed reliably (see `dt-g1-research/docs/firmware-analysis/adfu-dump-attempt.md`). On Linux, libusb opens the device directly, so one script can send `adfu` and grab it immediately. The tool exposes this only on Linux, clearly marked advanced.

## 5. Glyphix app API (for the v2 app builder)

Source: the reference docs in `dt-g1-research/docs/reference/` (oppo-watch-s-sideload DEV_GUIDE and PKG_FORMAT, dtg1-glyphix-docs), and what we observed in real apps on this watch. The full API reference (framework, UI components, system API, tutorials) is in the `dtg1-glyphix-docs` repo's `ai_context` files; pull and summarize them when building the app builder.

### 5.1 App model
- An app is a "quick-app" style package (like a Chinese mini-program): a `manifest.json`, pages as `.ux` files, and JavaScript logic.
- A `.ux` page is one file with three blocks, like a Vue single-file component:
  - `<template>` - the UI tree (components below).
  - `<style>` - a CSS subset; the page root is usually `width:466px; height:466px`.
  - `<script>` - `export default { data, onInit, methods... }`.
- `manifest.json` (required fields): `package` (unique id), `name`, `icon` (required on-device, or install fails with 5005), `versionName`, `versionCode` (integer, raise to upgrade), `router` (entry + pages), `config.designWidth` (466).

### 5.2 Packaging
- `.pkg` is an unsigned flat "CR" archive (see `dt-g1-research/docs/firmware-analysis/debug-command-table.md` and the PKG_FORMAT reference). No signature, so building is just packing. `.js` is compiled to bytecode and `.uo` is compiled UI by the toolchain; `.png` is converted to PGF.
- Build with the official Glyphix CLI (`gx build -d <device> -f`). A device profile `dtg1-c175r.yml` (vendorID `0x0102`, productID `0x0005`, 466x466 circle) is in `dt-g1-apps/whose-watch/`.
- Install: copy the `.pkg` into `/1:/gui/apps/`, add an entry to `/1:/gui/pkgs.db`, and point a menu slot at the app's AppID. The watch menu is a fixed list, so a new app reuses an existing slot (for example the Wooden Fish slot `0xC82C`), and its menu name stays the built-in name of that AppID.

### 5.3 Available system modules (`@system.*`)
Observed and documented; confirm exact signatures against the `ai_context` reference before use.
- `@system.media` - media, including `createAudioPlayer()` with `player.src`, `play()`, `stop()`. This is the basis for a clean audio player app.
- `@system.file` - read/write files in the app sandbox and the shared USB area (`internal://mass/` maps to `/1:/data/mass/`).
- `@system.router` - page navigation.
- `@system.package` - list installed packages (`list('app')`), get info.
- `@system.launch` - launch another package by name.
- `@system.device` - device info.
- Others documented in the reference: network/fetch, sensors, notification, and more.

### 5.4 UI components
`div`, `stack`, `scroll`, `text`/`p`, `image`, `button`, `input`, `slider`, `progress`, `list`, `picker`, `marquee`, `canvas`, `qrcode`, `barcode`. Template directives: `for`, `if`/`elif`, `on:click`, `:prop` bindings, `{{ }}` interpolation. Lifecycle: page `onInit -> onReady -> onShow -> onHide -> onDestroy`; app `onCreate/onShow/onHide/onDestroy/onError`.

### 5.5 Hebrew
Hebrew is a supported system language. The firmware's language list includes Hebrew (`m_he-il.hex`), and the system font is `harmony.ttf` (HarmonyOS Sans), which includes Hebrew glyphs. The watch UI itself shows Hebrew (the About screen does). The three app-level fonts in `/1:/gui/fonts` (AliPuHui, NotoSans, NotoNaskhArabic) have no Hebrew, but that is the app font set, not the system font.

CONFIRMED (2026-11-05): a test app (`dt-g1-apps/hebrew-test`) that prints live Hebrew `<text>` with no bundled font and no image rendered correctly on the watch - crisp Hebrew letters, right-to-left order, no missing glyphs. So the app renderer uses the system font (which has Hebrew). The app builder uses live Hebrew text (correct, selectable, easy to edit); the pre-rendered image path (what the `whose-watch` sample did with Varela Round) stays only as a fallback for custom fonts or effects. Note: the `.ux` `direction: rtl` style is ignored by the compiler, but Hebrew is strong-RTL so the renderer orders it correctly on its own; use `text-align` for alignment.

## 6. Roadmap

- v1: connection wizard, hidden partition browser, menu editor. All read-mostly, proven operations. IMPLEMENTED (2026-11-05): the Tauri app builds and runs with all three panels. watch-core (menu parsing, serial AT channel, USB mode detection, partition listing) has unit tests. The watch-connected flows (load/save menu, browse the partition) are coded but still need verification on the watch; do that when the watch is next connected.
- v2: feature parity with the WearPro app (what the companion app offers), reverse-engineered from the WearPro APK kept in `dt-g1-research/reverse-engineering/wearpro-apk/`. Likely items: editing watch-face backgrounds and dials, transferring images and photos, pushing music and maps. Each is reimplemented over the same USB/serial and partition access the tool already has.
- v2 also: app builder (templates: image-screen app like whose-watch, and an audio player using `@system.media`), invoking the Glyphix CLI, with live Hebrew text once confirmed.
- v3: advanced ADFU dump on Linux.

## 7. Open questions

- ~~Does an app render Hebrew with the system font?~~ RESOLVED: yes. Live Hebrew text renders correctly with no bundled font (tested 2026-11-05).
- Exact `@system.media` playback to Bluetooth headphones vs the on-wrist speaker: confirm before promising a headphone audio player.
- Whether `pkgs.db` and the menu slot approach is stable across firmware updates.
