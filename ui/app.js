// DT G1 Studio - frontend. Uses the global Tauri API (withGlobalTauri).
const invoke = (cmd, args) => window.__TAURI__.core.invoke(cmd, args);

const I18N = {
  he: {
    "tab.connect": "חיבור",
    "tab.partition": "מחיצה מוסתרת",
    "tab.menu": "עריכת תפריט",
    "nav.hint": "קריאה בלבד כברירת מחדל. כל כתיבה מגובה ומאושרת.",
    "connect.title": "חיבור השעון",
    "connect.intro": "חבר את שעון ה-DT G1 למחשב עם כבל הטעינה. הכלי יזהה את מצב החיבור וידריך אותך צעד-צעד.",
    "connect.detect": "זיהוי מחדש",
    "connect.test": "בדיקת חיבור",
    "connect.unknown": "בודק מצב...",
    "status.charging": "מחובר במצב Charging - מוכן לעבודה",
    "status.mass": "מחובר במצב אחסון (Mass Storage)",
    "status.adfu": "במצב הורדה (ADFU)",
    "status.absent": "לא מזוהה שעון",
    "guide.charging": "השעון מחובר בערוץ הנכון. אפשר להמשיך, או ללחוץ בדיקת חיבור.",
    "guide.mass": "בשעון: הגדרות ‹ מצב USB ‹ בחר Charging mode, ואז נתק וחבר מחדש.",
    "guide.adfu": "השעון במצב הורדה. נתק וחבר מחדש כדי לחזור למצב רגיל.",
    "guide.absent": "חבר את השעון לעריסת הטעינה ולמחשב. אם כבר מחובר, בדוק שהעריסה יושבת טוב.",
    "status.demo": "מצב הדגמה - נתוני דוגמה",
    "guide.demo": "אתה עובד מול נתוני דוגמה, בלי שעון. אפשר לעיין בדפדפן המחיצה ובעורך התפריט כדי לראות איך הם נראים ועובדים.",
    "demo.banner": "מצב הדגמה פעיל - כל הנתונים הם דוגמה, שום שינוי לא נכתב לשעון.",
    "demo.off": "כבה הדגמה",
    "ping.ok": "החיבור תקין. השעון השיב.",
    "ping.fail": "בדיקת החיבור נכשלה",
    "partition.title": "המחיצה המוסתרת",
    "partition.intro": "המחיצה המוסתרת (/1:/) מכילה את קובצי המערכת של השעון. כאן אפשר לדפדף בה בקריאה בלבד, כתחליף פתוח לכלי המפעל הסגור.",
    "partition.open": "פתח את המחיצה",
    "partition.explorer": "פתח בסייר הקבצים",
    "partition.drive_note": "פתיחת המחיצה מרכיבה אותה גם ככונן חדש ב-Windows (למשל E:), שאפשר לפתוח ישירות בסייר הקבצים. הכלי מציג כאן תצוגה בטוחה בקריאה בלבד.",
    "partition.refresh": "רענן",
    "partition.exposing": "חושף את המחיצה... נתק אל תנתק את השעון.",
    "partition.need_charging": "כדי לפתוח את המחיצה, חבר את השעון במצב Charging mode.",
    "partition.not_found": "המחיצה לא נמצאה. נסה לנתק ולחבר מחדש את השעון.",
    "partition.readonly": "קריאה בלבד",
    "partition.empty": "התיקייה ריקה.",
    "partition.root": "שורש",
    "menu.title": "עריכת התפריט",
    "menu.intro": "טען את תפריט האפליקציות מהשעון, הסתר או סדר פריטים מחדש, ושמור בחזרה. הכלי שומר על מבנה הקובץ ומגבה אותו לפני כל כתיבה.",
    "menu.load": "טען מהשעון",
    "menu.save": "שמור לשעון",
    "menu.loading": "טוען את התפריט מהשעון...",
    "menu.need_charging": "כדי לטעון את התפריט, חבר את השעון במצב Charging mode.",
    "menu.not_found": "קובץ התפריט לא נמצא. נסה שוב.",
    "menu.saving": "שומר...",
    "menu.saved": "נשמר בהצלחה. גיבוי נוצר.",
    "menu.restart": "נתק והפעל מחדש את השעון כדי להחיל את השינויים.",
    "menu.visible": "מוצג",
    "menu.hidden": "מוסתר",
    "tab.background": "רקע שעון",
    "bg.title": "רקע מותאם",
    "bg.intro": "בחר תמונה, גרור למיקום המדויק, הגדל בזום, והוסף גוון או טקסט. התצוגה המקדימה העגולה היא בדיוק מה שייכתב לשעון (res/user_clock_bg.bin).",
    "bg.pick": "בחר תמונה",
    "bg.pick_hint": "לא נבחרה תמונה",
    "bg.drag_hint": "גרור להזזה, גלגל העכבר לזום",
    "bg.zoom": "זום",
    "bg.tint": "גוון צבע",
    "bg.text": "טקסט",
    "bg.reset": "איפוס",
    "bg.apply": "החל לשעון",
    "bg.applying": "כותב את הרקע...",
    "bg.done": "הרקע נכתב. בשעון, בחר את חזית התמונה המותאמת, והפעל מחדש אם צריך.",
    "bg.need": "כדי להחיל, חבר את השעון במצב מתאים (או הפעל מצב הדגמה).",
    "tab.maps": "מפות",
    "maps.title": "מפות אופליין",
    "maps.intro": "המפות המותקנות בשעון (בכונן המשתמש, תחת MAP/map). אפשר להסיר מפות, ולהוסיף חדשות דרך סייר הקבצים.",
    "maps.open": "פתח את תיקיית המפות",
    "maps.refresh": "רענן",
    "maps.osm_note": "ליצירת מפות OSM מפורטות משלך: הכלי dtg1-map-tools מקמפל קובץ map.osm (מיצוא של OpenStreetMap) לפורמט השעון, כולל מסלולי GPX. יש גם מפות מוכנות להורדה. בקרוב נשלב את הקימפול כאן.",
    "maps.empty": "אין מפות מותקנות. פתח את תיקיית המפות כדי להוסיף.",
    "maps.need": "חבר את השעון (או הפעל מצב הדגמה) כדי לראות מפות.",
    "maps.confirm_del": "להסיר את המפה",
    "maps.compile_title": "צור מפת OSM חדשה",
    "maps.compile_intro": "חפש אזור, מקם אותו על המפה, והתוכנה תוריד את נתוני OpenStreetMap ותקמפל אותם לפורמט השעון.",
    "maps.setup_note": "קימפול דורש הורדה חד-פעמית של כלי הקימפול הרשמי (כ-13MB).",
    "maps.setup": "הורד את כלי הקימפול",
    "maps.setting_up": "מוריד את כלי הקימפול (כ-13MB)...",
    "maps.search": "חפש",
    "maps.area_hint": "האזור המוצג על המפה הוא מה שיורד.",
    "maps.poi_landuse": "נקודות עניין: מוטמעות (מומלץ)",
    "maps.poi_native": "נקודות עניין: שכבה נפרדת",
    "maps.poi_none": "בלי נקודות עניין",
    "maps.compile": "הורד וקמפל",
    "maps.install": "התקן בשעון",
    "maps.downloading": "מוריד נתוני OSM...",
    "maps.compiling": "מקמפל את המפה...",
    "maps.installed": "המפה הותקנה בשעון.",
    "maps.area_big": "האזור גדול - אם הקימפול נכשל, הקטן את התצוגה (זום פנימה).",
    "maps.need_name": "הזן שם למפה.",
    "tab.apps": "אפליקציות",
    "apps.title": "בונה אפליקציות",
    "apps.intro": "בנה אפליקציות עברית פשוטות לשעון מתוך תבניות - ללא קוד. מלא טקסט, ראה תצוגה מקדימה חיה, והתקן.",
    "apps.gx_missing": "כלי הבנייה (Glyphix gx) לא נמצא במחשב. בנייה והתקנה לא יעבדו עד שיותקן.",
    "apps.launcher_title": "משגר (מומלץ)",
    "apps.launcher_intro": "התקנת המשגר PurrLaunch פעם אחת (מחליף את 'דגי עץ'). אחריו כל אפליקציה חדשה מופיעה בתוכו אוטומטית, בלי לוותר על עוד פריט בתפריט.",
    "apps.launcher_install": "התקן משגר",
    "launcher.installed": "המשגר מותקן",
    "launcher.not_installed": "המשגר לא מותקן",
    "launcher.installing": "מוריד ומתקין את המשגר...",
    "launcher.done": "המשגר הותקן. הפעל מחדש את השעון.",
    "apps.pick_template": "בחר תבנית",
    "apps.display_name": "שם האפליקציה (נראה במשגר)",
    "apps.preview_hint": "תצוגה מקדימה חיה - כך זה ייראה על השעון",
    "apps.install_title": "התקנה",
    "apps.target": "יעד ההתקנה",
    "apps.target_launcher": "אפליקציה חופשית במשגר (מומלץ)",
    "apps.target_launcher_hint": "האפליקציה תופיע בתוך המשגר. צריך שהמשגר יהיה מותקן.",
    "apps.target_slot_hint": "האפליקציה תחליף אפליקציית מקור ותתפוס את מקומה בתפריט.",
    "apps.build_install": "בנה והתקן",
    "apps.building": "בונה את האפליקציה (gx)...",
    "apps.installing": "מתקין בשעון...",
    "apps.need_title": "הזן כותרת לאפליקציה.",
    "apps.need_launcher": "התקן קודם את המשגר, או בחר יעד של החלפת אפליקציה.",
    "apps.restart": "הותקן. הפעל מחדש את השעון כדי לראות את האפליקציה.",
    "apps.need": "חבר את השעון (או הפעל מצב דמו) כדי להתקין.",
    "apps.built_demo": "נבנה בהצלחה (מצב דמו - לא הותקן בשעון).",
    "soon": "בבנייה. יתווסף בקרוב.",
    "footer.safe": "קריאה בלבד ומעברי-מצב בלבד. אין כתיבה או מחיקה לקושחה.",
  },
  en: {
    "tab.connect": "Connect",
    "tab.partition": "Hidden partition",
    "tab.menu": "Menu editor",
    "nav.hint": "Read-only by default. Any write is backed up and confirmed.",
    "connect.title": "Connect the watch",
    "connect.intro": "Connect the DT G1 watch to the PC with its charging cable. The tool detects the connection state and guides you step by step.",
    "connect.detect": "Detect again",
    "connect.test": "Test connection",
    "connect.unknown": "Checking...",
    "status.charging": "Connected in Charging mode - ready",
    "status.mass": "Connected in Mass Storage mode",
    "status.adfu": "In download mode (ADFU)",
    "status.absent": "No watch detected",
    "guide.charging": "The watch is on the right channel. Continue, or press Test connection.",
    "guide.mass": "On the watch: Settings › USB mode › choose Charging mode, then reconnect.",
    "guide.adfu": "The watch is in download mode. Reconnect it to return to normal.",
    "guide.absent": "Connect the watch to its charging dock and the PC. If it is already connected, check the dock contact.",
    "status.demo": "Demo mode - sample data",
    "guide.demo": "You are working with sample data, no watch needed. Browse the partition and the menu editor to see how they look and work.",
    "demo.banner": "Demo mode is on - all data is sample data, nothing is written to a watch.",
    "demo.off": "Turn off demo",
    "ping.ok": "Connection works. The watch replied.",
    "ping.fail": "Connection test failed",
    "partition.title": "Hidden partition",
    "partition.intro": "The hidden partition (/1:/) holds the watch system files. Browse it here read-only, an open replacement for the vendor's closed tool.",
    "partition.open": "Open the partition",
    "partition.explorer": "Open in File Explorer",
    "partition.drive_note": "Opening the partition also mounts it as a new drive in Windows (for example E:), which you can open directly in File Explorer. The tool shows a safe read-only view here.",
    "partition.refresh": "Refresh",
    "partition.exposing": "Exposing the partition... do not disconnect the watch.",
    "partition.need_charging": "To open the partition, connect the watch in Charging mode.",
    "partition.not_found": "Partition not found. Try reconnecting the watch.",
    "partition.readonly": "Read-only",
    "partition.empty": "The folder is empty.",
    "partition.root": "root",
    "menu.title": "Menu editor",
    "menu.intro": "Load the app menu from the watch, hide or reorder items, and save it back. The tool keeps the file structure intact and backs it up before every write.",
    "menu.load": "Load from watch",
    "menu.save": "Save to watch",
    "menu.loading": "Loading the menu from the watch...",
    "menu.need_charging": "To load the menu, connect the watch in Charging mode.",
    "menu.not_found": "Menu file not found. Try again.",
    "menu.saving": "Saving...",
    "menu.saved": "Saved. A backup was created.",
    "menu.restart": "Disconnect and restart the watch to apply the changes.",
    "menu.visible": "Shown",
    "menu.hidden": "Hidden",
    "tab.background": "Watch face",
    "bg.title": "Custom background",
    "bg.intro": "Pick an image, drag to position, zoom in, and add a tint or text. The round preview is exactly what gets written to the watch (res/user_clock_bg.bin).",
    "bg.pick": "Choose image",
    "bg.pick_hint": "No image selected",
    "bg.drag_hint": "Drag to move, scroll to zoom",
    "bg.zoom": "Zoom",
    "bg.tint": "Color tint",
    "bg.text": "Text",
    "bg.reset": "Reset",
    "bg.apply": "Apply to watch",
    "bg.applying": "Writing the background...",
    "bg.done": "Background written. On the watch, select the custom photo face, and restart if needed.",
    "bg.need": "To apply, connect the watch (or turn on demo mode).",
    "tab.maps": "Maps",
    "maps.title": "Offline maps",
    "maps.intro": "The maps installed on the watch (on the user drive, under MAP/map). Remove maps, and add new ones through File Explorer.",
    "maps.open": "Open the maps folder",
    "maps.refresh": "Refresh",
    "maps.osm_note": "To make your own detailed OSM maps: the dtg1-map-tools compiler turns a map.osm file (exported from OpenStreetMap) into the watch format, including GPX routes. Pre-compiled maps are also available. Compiling will be built in here soon.",
    "maps.empty": "No maps installed. Open the maps folder to add some.",
    "maps.need": "Connect the watch (or turn on demo mode) to see maps.",
    "maps.confirm_del": "Remove the map",
    "maps.compile_title": "Create a new OSM map",
    "maps.compile_intro": "Search for an area, position it on the map, and the tool downloads OpenStreetMap data and compiles it to the watch format.",
    "maps.setup_note": "Compiling needs a one-time download of the official compiler (about 13 MB).",
    "maps.setup": "Download the compiler",
    "maps.setting_up": "Downloading the compiler (about 13 MB)...",
    "maps.search": "Search",
    "maps.area_hint": "The area shown on the map is what gets downloaded.",
    "maps.poi_landuse": "Points of interest: baked in (recommended)",
    "maps.poi_native": "Points of interest: separate layer",
    "maps.poi_none": "No points of interest",
    "maps.compile": "Download and compile",
    "maps.install": "Install to watch",
    "maps.downloading": "Downloading OSM data...",
    "maps.compiling": "Compiling the map...",
    "maps.installed": "Map installed on the watch.",
    "maps.area_big": "Large area - if compiling fails, zoom in to a smaller area.",
    "maps.need_name": "Enter a map name.",
    "tab.apps": "Apps",
    "apps.title": "App builder",
    "apps.intro": "Build simple Hebrew watch apps from templates - no code. Fill in text, see a live preview, and install.",
    "apps.gx_missing": "The build toolchain (Glyphix gx) was not found. Building and installing will not work until it is installed.",
    "apps.launcher_title": "Launcher (recommended)",
    "apps.launcher_intro": "Install the PurrLaunch launcher once (it takes over Wooden Fish). After that, every new app appears inside it automatically, without giving up another menu slot.",
    "apps.launcher_install": "Install launcher",
    "launcher.installed": "Launcher installed",
    "launcher.not_installed": "Launcher not installed",
    "launcher.installing": "Downloading and installing the launcher...",
    "launcher.done": "Launcher installed. Restart the watch.",
    "apps.pick_template": "Choose a template",
    "apps.display_name": "App name (shown in the launcher)",
    "apps.preview_hint": "Live preview - this is how it looks on the watch",
    "apps.install_title": "Install",
    "apps.target": "Install target",
    "apps.target_launcher": "Free app in the launcher (recommended)",
    "apps.target_launcher_hint": "The app appears inside the launcher. Requires the launcher to be installed.",
    "apps.target_slot_hint": "The app replaces a stock app and takes its place in the menu.",
    "apps.build_install": "Build and install",
    "apps.building": "Building the app (gx)...",
    "apps.installing": "Installing to the watch...",
    "apps.need_title": "Enter an app title.",
    "apps.need_launcher": "Install the launcher first, or pick a replace-app target.",
    "apps.restart": "Installed. Restart the watch to see the app.",
    "apps.need": "Connect the watch (or turn on demo mode) to install.",
    "apps.built_demo": "Built successfully (demo mode - not installed to a watch).",
    "soon": "Under construction. Coming soon.",
    "footer.safe": "Read-only and mode-change only. No firmware writes or erases.",
  },
};

let lang = localStorage.getItem("lang") || "he";

function t(key) {
  return (I18N[lang] && I18N[lang][key]) || key;
}

function applyI18n() {
  document.documentElement.lang = lang;
  document.documentElement.dir = lang === "he" ? "rtl" : "ltr";
  document.getElementById("langToggle").textContent = lang === "he" ? "EN" : "עב";
  document.querySelectorAll("[data-i18n]").forEach((el) => {
    el.textContent = t(el.getAttribute("data-i18n"));
  });
}

/* ---------- Theme ---------- */
function effectiveTheme() {
  const forced = document.documentElement.dataset.theme;
  if (forced) return forced;
  return window.matchMedia("(prefers-color-scheme: dark)").matches ? "dark" : "light";
}
function updateThemeIcon() {
  const dark = effectiveTheme() === "dark";
  document.querySelector(".icon-sun").style.display = dark ? "block" : "none";
  document.querySelector(".icon-moon").style.display = dark ? "none" : "block";
}
function applyStoredTheme() {
  const stored = localStorage.getItem("theme");
  if (stored === "light" || stored === "dark") {
    document.documentElement.dataset.theme = stored;
  } else {
    delete document.documentElement.dataset.theme;
  }
  updateThemeIcon();
}
function toggleTheme() {
  const next = effectiveTheme() === "dark" ? "light" : "dark";
  document.documentElement.dataset.theme = next;
  localStorage.setItem("theme", next);
  updateThemeIcon();
}

/* ---------- Log ---------- */
function log(line) {
  const el = document.getElementById("log");
  const now = new Date().toLocaleTimeString();
  el.textContent += `[${now}] ${line}\n`;
  el.scrollTop = el.scrollHeight;
}

/* ---------- Connection ---------- */
let lastMode = null;

function setStatus(mode) {
  const dot = document.getElementById("statusDot");
  const text = document.getElementById("statusText");
  const guide = document.getElementById("guideText");
  const pingBtn = document.getElementById("pingBtn");
  dot.className = "dot";
  if (mode === "charging") dot.classList.add("ok");
  else if (mode === "mass" || mode === "adfu") dot.classList.add("warn");
  else dot.classList.add("err");
  text.textContent = t("status." + mode);
  guide.textContent = t("guide." + mode);
  pingBtn.disabled = mode !== "charging";

  // Enable feature tabs: partition works in charging/mass/demo; menu in charging/demo.
  const partTab = document.querySelector('.nav-item[data-tab="partition"]');
  const menuTab = document.querySelector('.nav-item[data-tab="menu"]');
  if (partTab) partTab.disabled = !(mode === "charging" || mode === "mass" || mode === "demo");
  if (menuTab) menuTab.disabled = !(mode === "charging" || mode === "demo");
  const bgTab = document.querySelector('.nav-item[data-tab="background"]');
  if (bgTab) bgTab.disabled = !(mode === "charging" || mode === "mass" || mode === "demo");
  const mapsTab = document.querySelector('.nav-item[data-tab="maps"]');
  if (mapsTab) mapsTab.disabled = !(mode === "charging" || mode === "mass" || mode === "demo");
  const appsTab = document.querySelector('.nav-item[data-tab="apps"]');
  if (appsTab) appsTab.disabled = !(mode === "charging" || mode === "mass" || mode === "demo");
}

/* ---------- Maps ---------- */
async function loadMaps() {
  const msg = document.getElementById("mapsMsg");
  const list = document.getElementById("mapsList");
  try {
    const maps = await invoke("list_maps");
    list.innerHTML = "";
    msg.textContent = "";
    if (maps.length === 0) {
      const e = document.createElement("div");
      e.className = "map-empty";
      e.textContent = t("maps.empty");
      list.appendChild(e);
      return;
    }
    for (const m of maps) {
      const row = document.createElement("div");
      row.className = "map-row";
      row.innerHTML =
        '<svg class="map-icon" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M9 3L3 6v15l6-3 6 3 6-3V3l-6 3-6-3zM9 3v15M15 6v15"/></svg>' +
        '<div class="map-info"><div class="map-name"></div><div class="map-sub"></div></div>' +
        '<button class="map-del"><svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M3 6h18M8 6V4a2 2 0 0 1 2-2h4a2 2 0 0 1 2 2v2M19 6l-1 14a2 2 0 0 1-2 2H8a2 2 0 0 1-2-2L5 6"/></svg></button>';
      row.querySelector(".map-name").textContent = m.display;
      row.querySelector(".map-sub").textContent = m.name + "  ·  " + formatSize(m.size);
      row.querySelector(".map-del").addEventListener("click", () => deleteMap(m));
      list.appendChild(row);
    }
  } catch (e) {
    list.innerHTML = "";
    msg.textContent = String(e).includes("not_mounted") ? t("maps.need") : String(e);
  }
}

async function deleteMap(m) {
  if (!window.confirm(t("maps.confirm_del") + ' "' + m.display + '"?')) return;
  try {
    await invoke("delete_map", { name: m.name });
    await loadMaps();
  } catch (e) {
    document.getElementById("mapsMsg").textContent = String(e);
  }
}

async function revealMaps() {
  try {
    await invoke("reveal_maps");
  } catch (e) {
    document.getElementById("mapsMsg").textContent =
      String(e).includes("not_mounted") ? t("maps.need") : String(e);
  }
}

/* ---------- OSM map compiler ---------- */
let leaflet = null;
let compiledName = null;

function fmtSize(n) {
  if (n < 1024) return n + " B";
  if (n < 1048576) return (n / 1024).toFixed(0) + " KB";
  return (n / 1048576).toFixed(1) + " MB";
}

function initMap() {
  if (leaflet || typeof L === "undefined") return;
  leaflet = L.map("leafMap", { zoomControl: true }).setView([32.08, 34.78], 13);
  L.tileLayer("https://tile.openstreetmap.org/{z}/{x}/{y}.png", {
    maxZoom: 19,
    attribution: "© OpenStreetMap",
  }).addTo(leaflet);
  leaflet.on("moveend", updateArea);
  updateArea();
}

function updateArea() {
  if (!leaflet) return;
  const b = leaflet.getBounds();
  const latMid = (b.getNorth() + b.getSouth()) / 2;
  const w = (b.getEast() - b.getWest()) * 111.32 * Math.cos((latMid * Math.PI) / 180);
  const h = (b.getNorth() - b.getSouth()) * 111.32;
  const km2 = Math.abs(w * h);
  const el = document.getElementById("areaSize");
  el.textContent = "~" + (km2 < 10 ? km2.toFixed(1) : Math.round(km2)) + " km²";
  document.getElementById("compileMsg").textContent = km2 > 400 ? t("maps.area_big") : "";
}

async function geocode() {
  const q = document.getElementById("geoInput").value.trim();
  if (!q || !leaflet) return;
  try {
    const url = "https://nominatim.openstreetmap.org/search?format=jsonv2&limit=1&q=" + encodeURIComponent(q);
    const res = await fetch(url, { headers: { "Accept-Language": lang } });
    const arr = await res.json();
    if (arr.length === 0) return;
    const r = arr[0];
    const bb = r.boundingbox.map(Number); // [south, north, west, east]
    leaflet.fitBounds([[bb[0], bb[2]], [bb[1], bb[3]]]);
    const nm = (r.display_name || q).split(",")[0].trim();
    if (!document.getElementById("mapName").value) document.getElementById("mapName").value = nm;
  } catch (e) {
    document.getElementById("compileMsg").textContent = String(e);
  }
}

async function checkCompiler() {
  try {
    const ready = await invoke("map_compiler_ready");
    document.getElementById("mapsSetup").style.display = ready ? "none" : "flex";
    document.getElementById("mapsCompiler").style.display = ready ? "block" : "none";
    if (ready && leaflet) setTimeout(() => leaflet.invalidateSize(), 50);
  } catch (e) { /* ignore */ }
}

async function setupCompiler() {
  const msg = document.getElementById("mapsMsg");
  msg.textContent = t("maps.setting_up");
  document.getElementById("mapsSetupBtn").disabled = true;
  try {
    await invoke("map_compiler_setup");
    msg.textContent = "";
    await checkCompiler();
    initMap();
    setTimeout(() => leaflet && leaflet.invalidateSize(), 60);
  } catch (e) {
    msg.textContent = String(e);
  } finally {
    document.getElementById("mapsSetupBtn").disabled = false;
  }
}

async function compileMap() {
  const name = document.getElementById("mapName").value.trim();
  const msg = document.getElementById("compileMsg");
  if (!name) { msg.textContent = t("maps.need_name"); return; }
  if (!leaflet) return;
  const b = leaflet.getBounds();
  const poi = document.getElementById("poiMode").value;
  document.getElementById("compileResult").style.display = "none";
  document.getElementById("compileBtn").disabled = true;
  msg.textContent = t("maps.downloading");
  try {
    const r = await invoke("compile_region", {
      west: b.getWest(), south: b.getSouth(), east: b.getEast(), north: b.getNorth(),
      name, poiMode: poi,
    });
    compiledName = name;
    msg.textContent = "";
    document.getElementById("compileResultText").textContent =
      "OSM: " + fmtSize(r.osm_bytes) + "  ·  " + t("maps.title") + ": " + fmtSize(r.total);
    document.getElementById("compileResult").style.display = "flex";
  } catch (e) {
    const s = String(e);
    msg.textContent = s.includes("area_too_large") ? t("maps.area_big")
      : s.includes("compiler_not_ready") ? t("maps.setup_note")
      : s.includes("osm_download_failed") ? "OSM download failed - try again or a smaller area."
      : s;
  } finally {
    document.getElementById("compileBtn").disabled = false;
  }
}

async function installCompiled() {
  if (!compiledName) return;
  const msg = document.getElementById("compileMsg");
  try {
    await invoke("install_compiled_map", { name: compiledName });
    msg.textContent = t("maps.installed");
    document.getElementById("compileResult").style.display = "none";
    await loadMaps();
  } catch (e) {
    msg.textContent = String(e).includes("not_mounted") ? t("maps.need") : String(e);
  }
}

/* ---------- Background editor (interactive canvas) ---------- */
const SIDE = 466;
let bgImage = null;
let bgView = { scale: 1, x: 0, y: 0 };
let bgEnrich = { tintColor: "#000000", tintAmt: 0, text: "", textColor: "#ffffff", textSize: 46 };

function bgCanvas() { return document.getElementById("bgCanvas"); }

function bgDraw() {
  const ctx = bgCanvas().getContext("2d");
  ctx.fillStyle = "#000";
  ctx.fillRect(0, 0, SIDE, SIDE);
  if (bgImage) {
    ctx.drawImage(bgImage, bgView.x, bgView.y, bgImage.width * bgView.scale, bgImage.height * bgView.scale);
  }
  if (bgEnrich.tintAmt > 0) {
    ctx.save();
    ctx.globalAlpha = bgEnrich.tintAmt / 100;
    ctx.fillStyle = bgEnrich.tintColor;
    ctx.fillRect(0, 0, SIDE, SIDE);
    ctx.restore();
  }
  if (bgEnrich.text) {
    ctx.save();
    ctx.font = `700 ${bgEnrich.textSize}px Rubik, sans-serif`;
    ctx.fillStyle = bgEnrich.textColor;
    ctx.textAlign = "center";
    ctx.shadowColor = "rgba(0,0,0,.55)";
    ctx.shadowBlur = 8;
    ctx.shadowOffsetY = 2;
    ctx.fillText(bgEnrich.text, SIDE / 2, SIDE - 66, SIDE - 56);
    ctx.restore();
  }
}

function bgClamp() {
  if (!bgImage) return;
  const w = bgImage.width * bgView.scale, h = bgImage.height * bgView.scale;
  if (w >= SIDE) bgView.x = Math.min(0, Math.max(SIDE - w, bgView.x));
  if (h >= SIDE) bgView.y = Math.min(0, Math.max(SIDE - h, bgView.y));
}

function bgFitCover() {
  if (!bgImage) return;
  const s = Math.max(SIDE / bgImage.width, SIDE / bgImage.height);
  bgView.scale = s;
  bgView.x = (SIDE - bgImage.width * s) / 2;
  bgView.y = (SIDE - bgImage.height * s) / 2;
  document.getElementById("bgZoom").value = 100;
  bgDraw();
}

function bgZoomTo(percent) {
  if (!bgImage) return;
  const base = Math.max(SIDE / bgImage.width, SIDE / bgImage.height);
  const s1 = base * (percent / 100);
  const s0 = bgView.scale;
  const c = SIDE / 2;
  bgView.x = c - ((c - bgView.x) / s0) * s1;
  bgView.y = c - ((c - bgView.y) / s0) * s1;
  bgView.scale = s1;
  bgClamp();
  bgDraw();
}

function bgLoadFile(file) {
  const url = URL.createObjectURL(file);
  const im = new Image();
  im.onload = () => {
    bgImage = im;
    document.getElementById("bgFrame").classList.add("has-img");
    document.getElementById("bgApply").disabled = false;
    document.getElementById("bgDone").style.display = "none";
    bgFitCover();
    URL.revokeObjectURL(url);
  };
  im.src = url;
}

function bgSetupPan() {
  const c = bgCanvas();
  let dragging = false, lastX = 0, lastY = 0;
  c.addEventListener("pointerdown", (e) => {
    if (!bgImage) return;
    dragging = true; lastX = e.clientX; lastY = e.clientY;
    c.setPointerCapture(e.pointerId);
  });
  c.addEventListener("pointermove", (e) => {
    if (!dragging) return;
    const r = c.getBoundingClientRect();
    bgView.x += (e.clientX - lastX) * (SIDE / r.width);
    bgView.y += (e.clientY - lastY) * (SIDE / r.height);
    lastX = e.clientX; lastY = e.clientY;
    bgClamp(); bgDraw();
  });
  c.addEventListener("pointerup", () => { dragging = false; });
  c.addEventListener("wheel", (e) => {
    if (!bgImage) return;
    e.preventDefault();
    const z = document.getElementById("bgZoom");
    z.value = Math.max(100, Math.min(400, Number(z.value) + (e.deltaY < 0 ? 12 : -12)));
    bgZoomTo(Number(z.value));
  }, { passive: false });
}

async function bgApply() {
  if (!bgImage) return;
  const msg = document.getElementById("bgMsg");
  msg.textContent = t("bg.applying");
  try {
    const blob = await new Promise((res) => bgCanvas().toBlob(res, "image/jpeg", 0.9));
    const b64 = await new Promise((res, rej) => {
      const r = new FileReader();
      r.onload = () => res(String(r.result).split(",")[1]);
      r.onerror = rej;
      r.readAsDataURL(blob);
    });
    await invoke("apply_background_jpeg", { jpegB64: b64 });
    msg.textContent = "";
    document.getElementById("bgDone").style.display = "flex";
  } catch (e) {
    msg.textContent = String(e).includes("not_mounted") ? t("bg.need") : String(e);
  }
}

function bgReset() {
  bgEnrich = { tintColor: "#000000", tintAmt: 0, text: "", textColor: "#ffffff", textSize: 46 };
  document.getElementById("bgTintColor").value = "#000000";
  document.getElementById("bgTintAmt").value = 0;
  document.getElementById("bgText").value = "";
  document.getElementById("bgTextColor").value = "#ffffff";
  document.getElementById("bgTextSize").value = 46;
  bgFitCover();
}

/* ---------- Demo mode ---------- */
let demoOn = false;
async function toggleDemo(force) {
  const want = force !== undefined ? force : !demoOn;
  try {
    if (want) await invoke("enable_demo", { path: null });
    else await invoke("disable_demo");
    demoOn = want;
    document.getElementById("demoToggle").classList.toggle("on", demoOn);
    document.getElementById("demoBanner").style.display = demoOn ? "flex" : "none";
    document.getElementById("menuRestart").style.display = "none";
    await detect(true);
  } catch (e) {
    log("demo error: " + e);
  }
}

/* ---------- Partition browser ---------- */
function formatSize(n) {
  if (n < 1024) return n + " B";
  if (n < 1024 * 1024) return (n / 1024).toFixed(1) + " KB";
  return (n / 1048576).toFixed(1) + " MB";
}

const folderSvg = '<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M3 7a2 2 0 0 1 2-2h4l2 2h8a2 2 0 0 1 2 2v8a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2z"/></svg>';
const fileSvg = '<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M14 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V8z"/><path d="M14 2v6h6"/></svg>';

let partPath = "";

function renderCrumb() {
  const crumb = document.getElementById("partCrumb");
  crumb.innerHTML = "";
  const parts = partPath.split("/").filter(Boolean);
  const mk = (label, rel, current) => {
    const b = document.createElement("button");
    b.className = "crumb" + (current ? " current" : "");
    b.textContent = label;
    if (!current) b.addEventListener("click", () => partBrowse(rel));
    return b;
  };
  crumb.appendChild(mk(t("partition.root"), "", parts.length === 0));
  let acc = "";
  parts.forEach((p, i) => {
    acc = acc ? acc + "/" + p : p;
    const sep = document.createElement("span");
    sep.className = "crumb-sep";
    sep.textContent = "›";
    crumb.appendChild(sep);
    crumb.appendChild(mk(p, acc, i === parts.length - 1));
  });
}

async function partBrowse(rel) {
  try {
    const entries = await invoke("partition_list", { rel });
    partPath = rel;
    renderCrumb();
    const list = document.getElementById("partList");
    list.innerHTML = "";
    if (entries.length === 0) {
      const e = document.createElement("div");
      e.className = "file-empty";
      e.textContent = t("partition.empty");
      list.appendChild(e);
      return;
    }
    for (const ent of entries) {
      const row = document.createElement("div");
      row.className = "file-row" + (ent.is_dir ? " dir" : "");
      row.innerHTML =
        (ent.is_dir ? folderSvg : fileSvg) +
        '<span class="file-name"></span>' +
        '<span class="file-size"></span>';
      row.querySelector(".file-name").textContent = ent.name;
      row.querySelector(".file-size").textContent = ent.is_dir ? "" : formatSize(ent.size);
      if (ent.is_dir) {
        row.addEventListener("click", () => partBrowse(rel ? rel + "/" + ent.name : ent.name));
      }
      list.appendChild(row);
    }
  } catch (e) {
    document.getElementById("partMsg").textContent = String(e);
  }
}

async function exposePartition() {
  const msg = document.getElementById("partMsg");
  const browser = document.getElementById("partBrowser");
  const refresh = document.getElementById("partRefresh");
  msg.textContent = t("partition.exposing");
  try {
    await invoke("expose_partition");
    msg.textContent = "";
    browser.style.display = "block";
    refresh.style.display = "inline-flex";
    document.getElementById("partExplorer").style.display = "inline-flex";
    await partBrowse("");
  } catch (e) {
    const key = String(e).includes("charging") ? "partition.need_charging"
      : String(e).includes("not_found") ? "partition.not_found" : null;
    msg.textContent = key ? t(key) : String(e);
  }
}

async function detect(fromUser) {
  try {
    const conn = await invoke("detect_connection");
    setStatus(conn.mode);
    if (fromUser || conn.mode !== lastMode) {
      log("detect: " + conn.mode + (conn.port ? " (" + conn.port + ")" : ""));
    }
    lastMode = conn.mode;
  } catch (e) {
    log("detect error: " + e);
  }
}

async function ping() {
  log("ping...");
  try {
    const reply = await invoke("ping_watch");
    const clean = (reply || "").replace(/\r/g, "").trim();
    log(t("ping.ok") + " [" + clean + "]");
  } catch (e) {
    log(t("ping.fail") + ": " + e);
  }
}

function switchTab(name) {
  document.querySelectorAll(".nav-item").forEach((b) => b.classList.toggle("active", b.dataset.tab === name));
  document.querySelectorAll(".panel").forEach((p) => p.classList.toggle("active", p.id === "tab-" + name));
  if (name === "maps") {
    loadMaps();
    initMap();
    checkCompiler();
    setTimeout(() => leaflet && leaflet.invalidateSize(), 80);
  }
  if (name === "apps") {
    initApps();
  }
}

/* ---------- Menu editor ---------- */
const upSvg = '<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.4" stroke-linecap="round" stroke-linejoin="round"><path d="M18 15l-6-6-6 6"/></svg>';
const downSvg = '<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.4" stroke-linecap="round" stroke-linejoin="round"><path d="M6 9l6 6 6-6"/></svg>';

let menuModel = [];

function mkReorder(svg, disabled, fn) {
  const b = document.createElement("button");
  b.className = "reorder-btn";
  b.type = "button";
  b.innerHTML = svg;
  b.disabled = disabled;
  if (!disabled) b.addEventListener("click", fn);
  return b;
}

function moveMenu(from, to) {
  const x = menuModel.splice(from, 1)[0];
  menuModel.splice(to, 0, x);
  renderMenu();
}

function renderMenu() {
  const list = document.getElementById("menuList");
  list.style.display = "block";
  list.innerHTML = "";
  menuModel.forEach((s, i) => {
    const row = document.createElement("div");
    row.className = "menu-row" + (s.visible ? "" : " hidden");

    const sw = document.createElement("label");
    sw.className = "switch";
    const cb = document.createElement("input");
    cb.type = "checkbox";
    cb.checked = s.visible;
    cb.addEventListener("change", () => {
      s.visible = cb.checked;
      row.classList.toggle("hidden", !s.visible);
    });
    const slider = document.createElement("span");
    slider.className = "slider";
    sw.append(cb, slider);

    const label = document.createElement("div");
    label.className = "menu-label";
    label.innerHTML = '<span class="menu-name"></span><span class="menu-hex"></span>';
    label.querySelector(".menu-name").textContent =
      s.name && s.name !== "?" ? s.name : "AppID " + s.hex;
    label.querySelector(".menu-hex").textContent = s.hex;

    const re = document.createElement("div");
    re.className = "menu-reorder";
    re.append(
      mkReorder(upSvg, i === 0, () => moveMenu(i, i - 1)),
      mkReorder(downSvg, i === menuModel.length - 1, () => moveMenu(i, i + 1))
    );

    row.append(sw, label, re);
    list.appendChild(row);
  });
}

async function loadMenu() {
  const msg = document.getElementById("menuMsg");
  msg.textContent = t("menu.loading");
  try {
    menuModel = await invoke("menu_load");
    msg.textContent = "";
    renderMenu();
    document.getElementById("menuSave").disabled = false;
  } catch (e) {
    const s = String(e);
    msg.textContent = s.includes("charging")
      ? t("menu.need_charging")
      : s.includes("not_found") || s.includes("menu")
      ? t("menu.not_found")
      : s;
  }
}

async function saveMenu() {
  const msg = document.getElementById("menuMsg");
  msg.textContent = t("menu.saving");
  try {
    const slots = menuModel.map((s) => ({
      app_id: s.app_id,
      icon_id: s.icon_id,
      name_id: s.name_id,
      visible: s.visible,
    }));
    await invoke("menu_save", { slots });
    msg.textContent = t("menu.saved");
    document.getElementById("menuRestart").style.display = "flex";
  } catch (e) {
    msg.textContent = String(e);
  }
}

/* ---------- App builder ---------- */
let appTemplates = null;
let appSlots = [];
let appSel = null; // selected template object
let appFields = {}; // key -> value
let launcherInstalled = false;

function esc(s) {
  return String(s == null ? "" : s).replace(/[&<>"]/g, (c) =>
    ({ "&": "&amp;", "<": "&lt;", ">": "&gt;", '"': "&quot;" }[c])
  );
}

// A small stable string hash -> 4 hex chars, for unique-but-stable package ids.
function hash4(s) {
  let h = 0x811c9dc5;
  for (let i = 0; i < s.length; i++) {
    h ^= s.charCodeAt(i);
    h = (h * 0x01000193) >>> 0;
  }
  return ("0000" + h.toString(16)).slice(-4);
}

function slugify(s) {
  const a = String(s || "").toLowerCase().replace(/[^a-z0-9]/g, "");
  return a || "app";
}

async function initApps() {
  const gxBox = document.getElementById("gxMissing");
  try {
    const gx = await invoke("gx_status");
    gxBox.style.display = gx.ready ? "none" : "flex";
  } catch (e) {
    gxBox.style.display = "none";
  }
  if (!appTemplates) {
    try {
      appTemplates = await invoke("app_templates");
      appSlots = await invoke("app_slots");
    } catch (e) {
      appTemplates = [];
      appSlots = [];
    }
    renderGallery();
  }
  await refreshLauncher();
}

async function refreshLauncher() {
  const pill = document.getElementById("launcherState");
  try {
    const st = await invoke("launcher_status");
    launcherInstalled = !!st.installed;
    pill.textContent = st.installed ? t("launcher.installed") : t("launcher.not_installed");
    pill.className = "pill " + (st.installed ? "ok" : "");
  } catch (e) {
    launcherInstalled = false;
    pill.textContent = t("launcher.not_installed");
    pill.className = "pill";
  }
}

function renderGallery() {
  const g = document.getElementById("tplGallery");
  g.innerHTML = "";
  for (const tpl of appTemplates || []) {
    const card = document.createElement("button");
    card.type = "button";
    card.className = "tpl-card";
    const name = lang === "he" ? tpl.he : tpl.en;
    const desc = lang === "he" ? tpl.desc_he : tpl.desc_en;
    card.innerHTML = '<div class="tpl-name"></div><div class="tpl-desc"></div>';
    card.querySelector(".tpl-name").textContent = name;
    card.querySelector(".tpl-desc").textContent = desc;
    card.addEventListener("click", () => selectTemplate(tpl, card));
    g.appendChild(card);
  }
}

const APP_DEFAULT_COLORS = { bg: "#101322", accent: "#ffd65a" };

function selectTemplate(tpl, card) {
  appSel = tpl;
  appFields = {};
  document.querySelectorAll(".tpl-card").forEach((c) => c.classList.toggle("active", c === card));
  // Seed defaults.
  for (const f of tpl.fields) {
    if (f.kind === "color") appFields[f.key] = APP_DEFAULT_COLORS[f.key] || "#101322";
    else appFields[f.key] = "";
  }
  renderFields();
  renderTargets();
  document.getElementById("builderTitle").textContent = lang === "he" ? tpl.he : tpl.en;
  document.getElementById("appBuilder").style.display = "grid";
  document.getElementById("appInstallBox").style.display = "block";
  document.getElementById("appDone").style.display = "none";
  document.getElementById("appMsg").textContent = "";
  renderPreview();
}

function renderFields() {
  const box = document.getElementById("appFields");
  box.innerHTML = "";
  for (const f of appSel.fields) {
    const ctrl = document.createElement("div");
    ctrl.className = "ctrl";
    const label = document.createElement("label");
    label.textContent = lang === "he" ? f.he : f.en;
    ctrl.appendChild(label);
    let input;
    if (f.kind === "multiline" || f.kind === "list") {
      input = document.createElement("textarea");
      input.rows = f.kind === "list" ? 5 : 7;
    } else if (f.kind === "color") {
      input = document.createElement("input");
      input.type = "color";
      input.value = appFields[f.key] || "#101322";
    } else {
      input = document.createElement("input");
      input.type = "text";
      if (f.kind === "phone") input.inputMode = "tel";
    }
    if (f.kind !== "color") input.value = appFields[f.key] || "";
    input.addEventListener("input", () => {
      appFields[f.key] = input.value;
      renderPreview();
    });
    ctrl.appendChild(input);
    box.appendChild(ctrl);
  }
}

function renderTargets() {
  const sel = document.getElementById("appTarget");
  sel.innerHTML = "";
  const opt = document.createElement("option");
  opt.value = "launcher";
  opt.textContent = t("apps.target_launcher");
  sel.appendChild(opt);
  for (const s of appSlots) {
    const o = document.createElement("option");
    o.value = "slot:" + s.app_id;
    o.textContent = (lang === "he" ? "החלף: " : "Replace: ") + s.label + " (" + s.hex + ")";
    sel.appendChild(o);
  }
  sel.onchange = updateTargetHint;
  updateTargetHint();
}

function updateTargetHint() {
  const sel = document.getElementById("appTarget");
  const hint = document.getElementById("appTargetHint");
  hint.textContent = sel.value === "launcher" ? t("apps.target_launcher_hint") : t("apps.target_slot_hint");
}

// Build the package id and optional target AppID for the chosen target.
function targetPlan() {
  const sel = document.getElementById("appTarget");
  const title = appFields.title || document.getElementById("appName").value || "";
  if (sel.value === "launcher") {
    const pid = "com.abaye." + slugify(title) + hash4(appSel.id + "|" + title);
    return { package_id: pid, target_app_id: null, launcher: true };
  }
  const appId = Number(sel.value.split(":")[1]);
  const slot = appSlots.find((s) => s.app_id === appId);
  return { package_id: slot.package_id, target_app_id: appId, launcher: false };
}

/* ----- live preview ----- */
function previewHtml(tpl, f) {
  const bg = /^#[0-9a-fA-F]{6}$/.test(f.bg || "") ? f.bg : APP_DEFAULT_COLORS.bg;
  const ac = /^#[0-9a-fA-F]{6}$/.test(f.accent || "") ? f.accent : APP_DEFAULT_COLORS.accent;
  const nn = (s) => (s || "").split(/\r?\n/).map((x) => x.trim()).filter(Boolean);
  if (tpl.id === "text_reader") {
    const paras = nn(f.body).map((p) => `<div class="pv-para">${esc(p)}</div>`).join("");
    const title = f.title ? `<div class="pv-title" style="color:${ac}">${esc(f.title)}</div>` : "";
    return `<div class="pv-scroll" style="background:${bg}">${title}${paras}</div>`;
  }
  if (tpl.id === "list") {
    const rows = nn(f.items).map((it) => `<div class="pv-row"><span>${esc(it)}</span></div>`).join("");
    const title = f.title ? `<div class="pv-title" style="color:${ac}">${esc(f.title)}</div>` : "";
    return `<div class="pv-scroll" style="background:${bg}">${title}${rows}</div>`;
  }
  // info_card
  const lines = [f.line1, f.line2].filter((x) => x && x.trim())
    .map((l) => `<div class="pv-line">${esc(l)}</div>`).join("");
  const title = f.title ? `<div class="pv-title" style="color:${ac}">${esc(f.title)}</div>` : "";
  const badge = f.name ? `<div class="pv-badge" style="background:${ac};color:${bg}">${esc(f.name)}</div>` : "";
  const phone = f.phone ? `<div class="pv-phone">${esc(f.phone)}</div>` : "";
  return `<div class="pv-center" style="background:${bg}">${title}${lines}${badge}${phone}</div>`;
}

function renderPreview() {
  if (!appSel) return;
  document.getElementById("appScreen").innerHTML = previewHtml(appSel, appFields);
  drawAppIcon();
}

// Draw a 192x192 icon: accent-ring tile with the first glyph of the title/name.
function drawAppIcon() {
  const c = document.getElementById("appIcon");
  const g = c.getContext("2d");
  const bg = /^#[0-9a-fA-F]{6}$/.test(appFields.bg || "") ? appFields.bg : APP_DEFAULT_COLORS.bg;
  const ac = /^#[0-9a-fA-F]{6}$/.test(appFields.accent || "") ? appFields.accent : APP_DEFAULT_COLORS.accent;
  g.clearRect(0, 0, 192, 192);
  g.fillStyle = bg;
  g.beginPath();
  g.arc(96, 96, 92, 0, Math.PI * 2);
  g.fill();
  g.lineWidth = 8;
  g.strokeStyle = ac;
  g.beginPath();
  g.arc(96, 96, 86, 0, Math.PI * 2);
  g.stroke();
  const txt = (appFields.name || appFields.title || "A").trim();
  const ch = Array.from(txt)[0] || "A";
  g.fillStyle = ac;
  g.font = "bold 110px Rubik, sans-serif";
  g.textAlign = "center";
  g.textBaseline = "middle";
  g.fillText(ch, 96, 104);
}

async function buildInstall() {
  const msg = document.getElementById("appMsg");
  const done = document.getElementById("appDone");
  done.style.display = "none";
  const title = (appFields.title || document.getElementById("appName").value || "").trim();
  if (!title) {
    msg.textContent = t("apps.need_title");
    return;
  }
  const plan = targetPlan();
  if (plan.launcher && !launcherInstalled && lastMode !== "demo") {
    msg.textContent = t("apps.need_launcher");
    return;
  }
  const name = (document.getElementById("appName").value || title).trim();
  const icon = document.getElementById("appIcon").toDataURL("image/png");
  try {
    msg.textContent = t("apps.building");
    await invoke("app_build", {
      template: appSel.id,
      packageId: plan.package_id,
      name,
      fields: appFields,
      iconB64: icon,
    });
    msg.textContent = t("apps.installing");
    await invoke("app_install", {
      packageId: plan.package_id,
      targetAppId: plan.target_app_id,
    });
    if (lastMode === "demo") {
      msg.textContent = t("apps.built_demo");
    } else {
      msg.textContent = "";
      done.style.display = "flex";
    }
    await refreshLauncher();
  } catch (e) {
    const s = String(e);
    msg.textContent = s.includes("not_charging")
      ? t("apps.need")
      : s.includes("gx_not_ready")
      ? t("apps.gx_missing")
      : s;
  }
}

async function launcherInstall() {
  const msg = document.getElementById("launcherMsg");
  msg.textContent = t("launcher.installing");
  try {
    await invoke("launcher_install");
    msg.textContent = t("launcher.done");
    await refreshLauncher();
  } catch (e) {
    const s = String(e);
    msg.textContent = s.includes("not_charging") ? t("apps.need") : s;
  }
}

window.addEventListener("DOMContentLoaded", async () => {
  applyStoredTheme();
  applyI18n();

  document.getElementById("themeToggle").addEventListener("click", toggleTheme);
  window.matchMedia("(prefers-color-scheme: dark)").addEventListener("change", updateThemeIcon);
  document.getElementById("demoToggle").addEventListener("click", () => toggleDemo());
  document.getElementById("demoOff").addEventListener("click", () => toggleDemo(false));

  document.getElementById("langToggle").addEventListener("click", () => {
    lang = lang === "he" ? "en" : "he";
    localStorage.setItem("lang", lang);
    applyI18n();
    if (lastMode) setStatus(lastMode);
  });

  document.getElementById("detectBtn").addEventListener("click", () => detect(true));
  document.getElementById("pingBtn").addEventListener("click", ping);
  document.getElementById("exposeBtn").addEventListener("click", exposePartition);
  document.getElementById("partRefresh").addEventListener("click", () => partBrowse(partPath));
  document.getElementById("partExplorer").addEventListener("click", async () => {
    try { await invoke("reveal_in_explorer", { rel: partPath }); }
    catch (e) { document.getElementById("partMsg").textContent = String(e); }
  });
  document.getElementById("menuLoad").addEventListener("click", loadMenu);
  document.getElementById("menuSave").addEventListener("click", saveMenu);
  document.getElementById("bgInput").addEventListener("change", (e) => {
    const file = e.target.files && e.target.files[0];
    if (file) bgLoadFile(file);
  });
  document.getElementById("bgZoom").addEventListener("input", (e) => bgZoomTo(Number(e.target.value)));
  document.getElementById("bgTintColor").addEventListener("input", (e) => { bgEnrich.tintColor = e.target.value; bgDraw(); });
  document.getElementById("bgTintAmt").addEventListener("input", (e) => { bgEnrich.tintAmt = Number(e.target.value); bgDraw(); });
  document.getElementById("bgText").addEventListener("input", (e) => { bgEnrich.text = e.target.value; bgDraw(); });
  document.getElementById("bgTextColor").addEventListener("input", (e) => { bgEnrich.textColor = e.target.value; bgDraw(); });
  document.getElementById("bgTextSize").addEventListener("input", (e) => { bgEnrich.textSize = Number(e.target.value); bgDraw(); });
  document.getElementById("bgApply").addEventListener("click", bgApply);
  document.getElementById("bgReset").addEventListener("click", bgReset);
  bgSetupPan();
  document.getElementById("mapsOpen").addEventListener("click", revealMaps);
  document.getElementById("mapsRefresh").addEventListener("click", loadMaps);
  document.getElementById("mapsSetupBtn").addEventListener("click", setupCompiler);
  document.getElementById("geoBtn").addEventListener("click", geocode);
  document.getElementById("geoInput").addEventListener("keydown", (e) => { if (e.key === "Enter") geocode(); });
  document.getElementById("compileBtn").addEventListener("click", compileMap);
  document.getElementById("installBtn").addEventListener("click", installCompiled);
  document.getElementById("launcherInstall").addEventListener("click", launcherInstall);
  document.getElementById("appBuild").addEventListener("click", buildInstall);
  document.querySelectorAll(".nav-item").forEach((b) => {
    b.addEventListener("click", () => { if (!b.disabled) switchTab(b.dataset.tab); });
  });

  await detect(true);
  // live refresh so the status updates when the user changes USB mode
  setInterval(() => detect(false), 3000);
});
