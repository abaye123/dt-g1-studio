# DT G1 Studio

A bilingual (Hebrew/English) desktop tool that guides anyone through connecting a DT G1 smartwatch (Actions ATS3085S, Glyphix OS) to a PC and customizing it safely: browse the hidden system partition, edit the app menu, and (later) build and install small apps.

כלי שולחני דו-לשוני (עברית/אנגלית) שמדריך כל אחד, צעד-צעד, לחבר שעון DT G1 (שבב Actions ATS3085S, מערכת Glyphix) למחשב ולהתאים אותו בבטחה: לדפדף במחיצת המערכת המוסתרת, לערוך את תפריט האפליקציות, ובהמשך גם לבנות ולהתקין אפליקציות קטנות.

Built with Tauri (Rust backend + web UI). The knowledge this is based on lives in the sibling `dt-g1-research` repo.

## Status

Planning / scaffolding. See `DESIGN.md` for the architecture and roadmap.

## Planned features (v1)

- Guided connection: detect the watch, walk the user through setting USB mode to Charging mode, and talk to it over the serial channel.
- Hidden partition browser: expose and read partition 2 (`/1:/`), a safe open replacement for the vendor's closed utility.
- Menu editor: a visual editor for `menu_list.cfg` (hide, reorder, show items), with backups and clear warnings.

## Later (v2+)

- App builder: templates and a form over the Glyphix CLI to build and install small apps (for example a clean audio player).
- Advanced: ADFU read-only firmware dump (Linux only; see the `dt-g1-research` repo notes).

## Safety

Read-only or clearly-marked mode-change operations by default. Any action that writes to the watch is explicit, backed up first, and never touches flash erase or partition commands.

## License

TBD. The font and any third-party assets keep their own licenses.
