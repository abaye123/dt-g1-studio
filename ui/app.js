// DT G1 Studio - frontend. Uses the global Tauri API (withGlobalTauri).
const invoke = (cmd, args) => window.__TAURI__.core.invoke(cmd, args);

const I18N = {
  he: {
    "tab.connect": "חיבור",
    "tab.partition": "מחיצה מוסתרת",
    "tab.menu": "עריכת תפריט",
    "connect.title": "חיבור השעון",
    "connect.intro": "חבר את שעון ה-DT G1 למחשב עם כבל הטעינה, ולחץ על זיהוי. הכלי ידריך אותך צעד-צעד.",
    "connect.detect": "זיהוי השעון",
    "connect.test": "בדיקת חיבור",
    "connect.unknown": "מצב לא ידוע",
    "status.charging": "מחובר במצב Charging - מוכן לעבודה",
    "status.mass": "מחובר במצב אחסון (Mass Storage)",
    "status.adfu": "במצב הורדה (ADFU)",
    "status.absent": "לא מזוהה שעון",
    "guide.charging": "השעון מחובר בערוץ הנכון. אפשר להמשיך, או ללחוץ בדיקת חיבור.",
    "guide.mass": "בשעון: הגדרות -> מצב USB -> בחר Charging mode, ואז נתק וחבר מחדש.",
    "guide.adfu": "השעון במצב הורדה. נתק וחבר מחדש כדי לחזור למצב רגיל.",
    "guide.absent": "חבר את השעון לעריסת הטעינה ולמחשב. אם כבר מחובר, בדוק שהעריסה יושבת טוב.",
    "ping.ok": "החיבור תקין. השעון השיב.",
    "ping.fail": "החיבור נכשל",
    "soon": "בקרוב.",
    "footer.safe": "קריאה בלבד ומעברי-מצב בלבד. אין כתיבה או מחיקה לקושחה.",
  },
  en: {
    "tab.connect": "Connect",
    "tab.partition": "Hidden partition",
    "tab.menu": "Menu editor",
    "connect.title": "Connect the watch",
    "connect.intro": "Connect the DT G1 watch to the PC with its charging cable, then press Detect. The tool guides you step by step.",
    "connect.detect": "Detect watch",
    "connect.test": "Test connection",
    "connect.unknown": "Unknown state",
    "status.charging": "Connected in Charging mode - ready",
    "status.mass": "Connected in Mass Storage mode",
    "status.adfu": "In download mode (ADFU)",
    "status.absent": "No watch detected",
    "guide.charging": "The watch is on the right channel. Continue, or press Test connection.",
    "guide.mass": "On the watch: Settings -> USB mode -> choose Charging mode, then reconnect.",
    "guide.adfu": "The watch is in download mode. Reconnect it to return to normal.",
    "guide.absent": "Connect the watch to its charging dock and the PC. If it is already connected, check the dock contact.",
    "ping.ok": "Connection works. The watch replied.",
    "ping.fail": "Connection test failed",
    "soon": "Coming soon.",
    "footer.safe": "Read-only and mode-change only. No firmware writes or erases.",
  },
};

let lang = "he";

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
  document.querySelectorAll(".tab").forEach((el) => {
    el.textContent = t("tab." + el.getAttribute("data-tab"));
  });
}

function log(line) {
  const el = document.getElementById("log");
  const now = new Date().toLocaleTimeString();
  el.textContent += `[${now}] ${line}\n`;
  el.scrollTop = el.scrollHeight;
}

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
}

async function detect() {
  try {
    const conn = await invoke("detect_connection");
    setStatus(conn.mode);
    log("detect: " + conn.mode + (conn.port ? " (" + conn.port + ")" : ""));
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
  document.querySelectorAll(".tab").forEach((b) => b.classList.toggle("active", b.dataset.tab === name));
  document.querySelectorAll(".panel").forEach((p) => p.classList.toggle("active", p.id === "tab-" + name));
}

window.addEventListener("DOMContentLoaded", async () => {
  applyI18n();
  document.getElementById("langToggle").addEventListener("click", () => {
    lang = lang === "he" ? "en" : "he";
    applyI18n();
  });
  document.getElementById("detectBtn").addEventListener("click", detect);
  document.getElementById("pingBtn").addEventListener("click", ping);
  document.querySelectorAll(".tab").forEach((b) => {
    b.addEventListener("click", () => { if (!b.disabled) switchTab(b.dataset.tab); });
  });
  try {
    document.getElementById("version").textContent = "v" + (await invoke("app_version"));
  } catch (e) { /* ignore */ }
  detect();
});
