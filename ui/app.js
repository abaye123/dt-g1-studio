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
  document.querySelectorAll(".nav-item").forEach((b) => {
    b.addEventListener("click", () => { if (!b.disabled) switchTab(b.dataset.tab); });
  });

  await detect(true);
  // live refresh so the status updates when the user changes USB mode
  setInterval(() => detect(false), 3000);
});
