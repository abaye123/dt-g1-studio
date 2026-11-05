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
    "ping.ok": "החיבור תקין. השעון השיב.",
    "ping.fail": "בדיקת החיבור נכשלה",
    "partition.title": "המחיצה המוסתרת",
    "partition.intro": "המחיצה המוסתרת (/1:/) מכילה את קובצי המערכת של השעון. כאן אפשר לדפדף בה בקריאה בלבד, כתחליף פתוח לכלי המפעל הסגור.",
    "partition.open": "פתח את המחיצה",
    "partition.refresh": "רענן",
    "partition.exposing": "חושף את המחיצה... נתק אל תנתק את השעון.",
    "partition.need_charging": "כדי לפתוח את המחיצה, חבר את השעון במצב Charging mode.",
    "partition.not_found": "המחיצה לא נמצאה. נסה לנתק ולחבר מחדש את השעון.",
    "partition.readonly": "קריאה בלבד",
    "partition.empty": "התיקייה ריקה.",
    "partition.root": "שורש",
    "menu.title": "עריכת התפריט",
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
    "ping.ok": "Connection works. The watch replied.",
    "ping.fail": "Connection test failed",
    "partition.title": "Hidden partition",
    "partition.intro": "The hidden partition (/1:/) holds the watch system files. Browse it here read-only, an open replacement for the vendor's closed tool.",
    "partition.open": "Open the partition",
    "partition.refresh": "Refresh",
    "partition.exposing": "Exposing the partition... do not disconnect the watch.",
    "partition.need_charging": "To open the partition, connect the watch in Charging mode.",
    "partition.not_found": "Partition not found. Try reconnecting the watch.",
    "partition.readonly": "Read-only",
    "partition.empty": "The folder is empty.",
    "partition.root": "root",
    "menu.title": "Menu editor",
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

  // Enable feature tabs by mode: partition works in charging or mass; menu needs charging.
  const partTab = document.querySelector('.nav-item[data-tab="partition"]');
  const menuTab = document.querySelector('.nav-item[data-tab="menu"]');
  if (partTab) partTab.disabled = !(mode === "charging" || mode === "mass");
  if (menuTab) menuTab.disabled = mode !== "charging";
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

window.addEventListener("DOMContentLoaded", async () => {
  applyStoredTheme();
  applyI18n();

  document.getElementById("themeToggle").addEventListener("click", toggleTheme);
  window.matchMedia("(prefers-color-scheme: dark)").addEventListener("change", updateThemeIcon);

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
  document.querySelectorAll(".nav-item").forEach((b) => {
    b.addEventListener("click", () => { if (!b.disabled) switchTab(b.dataset.tab); });
  });

  await detect(true);
  // live refresh so the status updates when the user changes USB mode
  setInterval(() => detect(false), 3000);
});
