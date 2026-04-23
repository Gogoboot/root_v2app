// ============================================================
// ROOT v2.0 — js/main.js
// Навигация через iconbar, темы, логирование
// ============================================================

let p2pInterval, msgInterval, diagInterval;

// ── Цвета аватаров — детерминированы по ключу ────────────────
const AVATAR_COLORS = [
  "#2196f3",
  "#e91e63",
  "#9c27b0",
  "#00bcd4",
  "#4caf50",
  "#ff9800",
  "#795548",
  "#607d8b",
  "#f44336",
  "#009688",
  "#673ab7",
  "#3f51b5",
];

// Получить цвет аватара по публичному ключу
window.getAvatarColor = function (pubkey) {
  if (!pubkey) return AVATAR_COLORS[0];
  let hash = 0;
  for (let i = 0; i < pubkey.length; i++) {
    hash = (hash * 31 + pubkey.charCodeAt(i)) & 0xffffffff;
  }
  return AVATAR_COLORS[Math.abs(hash) % AVATAR_COLORS.length];
};

// Получить текст аватара: первые 2 + последние 2 символа ключа
window.getAvatarText = function (pubkey) {
  if (!pubkey || pubkey.length < 4) return "??";
  return pubkey.slice(0, 2) + pubkey.slice(-2);
};

// Создать HTML-элемент аватара
window.makeAvatar = function (pubkey, size = 42) {
  const color = window.getAvatarColor(pubkey);
  const text = window.getAvatarText(pubkey);
  return `<div class="contact-avatar" style="width:${size}px;height:${size}px;background:${color};">${text}</div>`;
};

// ── Логирование ──────────────────────────────────────────────

function log(msg, type = "info") {
  const div = document.getElementById("log");
  if (div) {
    const entry = document.createElement("div");
    entry.className = `log-entry log-${type}`;
    entry.textContent = `[${new Date().toLocaleTimeString()}] ${msg}`;
    div.appendChild(entry);
    div.scrollTop = div.scrollHeight;
  }

  const initLog = document.getElementById("init-log");
  if (initLog) {
    const colors = {
      info: "var(--accent-dim)",
      success: "var(--green)",
      error: "var(--red)",
      warn: "var(--orange)",
    };
    initLog.style.color = colors[type] || "var(--text-muted)";
    initLog.textContent = `[${new Date().toLocaleTimeString()}] ${msg}`;
  }
}

window.log = log;

// ── Темы ─────────────────────────────────────────────────────

window.setTheme = function (theme, btnEl) {
  document.documentElement.setAttribute("data-theme", theme);
  localStorage.setItem("root-theme", theme);

  document
    .querySelectorAll(".theme-btn")
    .forEach((b) => b.classList.remove("active"));
  if (btnEl) {
    btnEl.classList.add("active");
  } else {
    const btn = document.getElementById("theme-btn-" + theme);
    if (btn) btn.classList.add("active");
  }
};

function restoreTheme() {
  const saved = localStorage.getItem("root-theme") || "dark";
  window.setTheme(saved, null);
}

// ── Показ/скрытие пароля ─────────────────────────────────────

window.togglePasswordVisibility = function () {
  const input = document.getElementById("db-password");
  const btn = document.querySelector(".password-toggle");
  if (!input) return;
  if (input.type === "password") {
    input.type = "text";
    if (btn) btn.textContent = "🙈";
  } else {
    input.type = "password";
    if (btn) btn.textContent = "👁";
  }
};

// ── Навигация (iconbar) ──────────────────────────────────────

function showTab(tabName, btnElement) {
  // Скрываем все вкладки
  document.querySelectorAll(".tab").forEach((el) => {
    el.classList.remove("active");
  });

  // Показываем нужную
  const target = document.getElementById("tab-" + tabName);
  if (target) target.classList.add("active");

  // Обновляем кнопки iconbar
  document
    .querySelectorAll(".icon-btn")
    .forEach((b) => b.classList.remove("active"));
  if (btnElement) btnElement.classList.add("active");

  // Загружаем данные при открытии
  if (tabName === "settings") {
    window.loadBootstrapList && window.loadBootstrapList();
    window.loadDbPath && window.loadDbPath();
  }
  if (tabName === "network") {
    window.refreshP2PStatus && window.refreshP2PStatus();
  }
  if (tabName === "diag") {
    window.refreshDiagnostics && window.refreshDiagnostics();
  }
}

window.showTab = showTab;

// ── Вход в приложение ────────────────────────────────────────

window.enterApp = async function () {
  // Показываем iconbar
  document.getElementById("iconbar").classList.add("visible");

  // Переключаем экраны
  document.getElementById("screen-init").classList.remove("active");
  document.getElementById("screen-app").classList.add("active");

  // Чат — первая вкладка
  showTab("chat", document.getElementById("nav-chat"));

  loadPublicKey();
  loadVersion();

  if (typeof window.initMessaging === "function") {
    await window.initMessaging();
  }

  // Загружаем контакты из базы в localStorage
  if (typeof window.loadContactsFromDb === "function") {
    await window.loadContactsFromDb();
  }

  startAutoRefresh();
};

// ── Выход ────────────────────────────────────────────────────

window.logout = async function () {
  if (!confirm("Выйти из аккаунта? База данных будет закрыта.")) return;

  clearInterval(p2pInterval);
  clearInterval(msgInterval);
  clearInterval(diagInterval);

  try {
    const invoke = window.__TAURI__.core.invoke;
    await invoke("stop_p2p_node").catch(() => {});
    await invoke("lock_database");
  } catch (e) {
    console.log("logout error:", e);
  }

  // Скрываем iconbar
  document.getElementById("iconbar").classList.remove("visible");

  // Возврат на экран входа
  document.getElementById("screen-app").classList.remove("active");
  document.getElementById("screen-init").classList.add("active");

  // Сброс UI
  const passInput = document.getElementById("db-password");
  if (passInput) {
    passInput.value = "";
    passInput.type = "password";
  }
  const passToggle = document.querySelector(".password-toggle");
  if (passToggle) passToggle.textContent = "👁";

  document.getElementById("my-pubkey").textContent = "Загрузка...";
  document.getElementById("settings-pubkey").textContent = "—";
  document.getElementById("db-path-display").textContent = "—";
  document.getElementById("p2p-status-text").textContent = "Остановлен";
  document.getElementById("peer-count-text").textContent = "0 пиров";
  document.getElementById("peer-list").innerHTML =
    '<p class="empty-state">Нет активных соединений</p>';
  document.getElementById("bootstrap-list").innerHTML =
    '<p class="empty-state">Нет bootstrap узлов</p>';
  document.getElementById("msg-list").innerHTML = `
        <div class="empty-chat-state">
            <span class="empty-chat-icon">⬡</span>
            <p>Выберите контакт слева<br>или начните новый чат</p>
        </div>`;
  document.getElementById("contact-list").innerHTML =
    '<p class="empty-state">Нет переписок</p>';
  document.getElementById("current-chat-name").textContent = "Выберите чат";
  document.getElementById("current-chat-key").textContent = "";
  document.getElementById("chat-header-avatar").innerHTML = "";
  document.getElementById("to-key").value = "";
  document.getElementById("msg-content").value = "";

  const editBtn = document.getElementById("btn-edit-nick");
  if (editBtn) editBtn.style.display = "none";

  log("Выход выполнен", "info");
};

// ── Данные ───────────────────────────────────────────────────

async function loadPublicKey() {
  try {
    const invoke = window.__TAURI__.core.invoke;
    const key = await invoke("get_public_key");
    document.getElementById("my-pubkey").textContent = key;
    document.getElementById("settings-pubkey").textContent = key;
    const diagKey = document.getElementById("diag-pubkey");
    if (diagKey) diagKey.textContent = key;
  } catch (e) {
    console.log("Ошибка загрузки ключа:", e);
  }
}

async function loadVersion() {
  try {
    const invoke = window.__TAURI__.core.invoke;
    const v = await invoke("get_version");
    document.getElementById("version-text").textContent = "Версия: " + v;
    const diagV = document.getElementById("diag-version");
    if (diagV) diagV.textContent = v;
  } catch (e) {}
}

window.copyPubkey = function () {
  const key = document.getElementById("my-pubkey").textContent;
  if (key && key !== "Загрузка...") {
    navigator.clipboard.writeText(key);
    log("Ключ скопирован", "success");
  }
};

window.copySettingsPubkey = function () {
  const key = document.getElementById("settings-pubkey").textContent;
  if (key && key !== "—") {
    navigator.clipboard.writeText(key);
    log("Ключ скопирован", "success");
  }
};

// ── Фоновое обновление ───────────────────────────────────────

function startAutoRefresh() {
  clearInterval(p2pInterval);
  clearInterval(msgInterval);
  clearInterval(diagInterval);

  p2pInterval = setInterval(window.refreshP2PStatus, 3000);
  msgInterval = setInterval(window.loadMessages, 5000);
  diagInterval = setInterval(() => {
    const diagTab = document.getElementById("tab-diag");
    if (diagTab && diagTab.classList.contains("active")) {
      window.refreshDiagnostics && window.refreshDiagnostics();
    }
  }, 10000);
}

// ── Инициализация ────────────────────────────────────────────

document.addEventListener("DOMContentLoaded", () => {
  restoreTheme();
});
