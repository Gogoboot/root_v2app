// ============================================================
// messaging.js — чат, контакты, пузыри сообщений
// ============================================================

let currentChatKey = null;
let myPublicKey = null;
let contactSearchQuery = "";

Object.defineProperty(window, "_currentChatKey", {
  get: () => currentChatKey,
  set: (v) => {
    currentChatKey = v;
  },
});

// ── Инициализация ────────────────────────────────────────────

window.initMessaging = async function () {
  const invoke = window.__TAURI__.core.invoke;
  try {
    myPublicKey = await invoke("get_public_key");
  } catch (e) {
    console.log("initMessaging: не удалось получить ключ", e);
  }
  await window.loadMessages();
};

// ── Загрузка сообщений ───────────────────────────────────────

window.loadMessages = async function () {
  const invoke = window.__TAURI__.core.invoke;
  try {
    const msgs = await invoke("get_incoming_messages");
    renderContactList(msgs);
    if (currentChatKey) {
      renderConversation(msgs, currentChatKey);
    }
  } catch (e) {
    console.log("loadMessages error:", e);
  }
};

// Фильтрация по поиску
window.filterContacts = function (query) {
  contactSearchQuery = query.toLowerCase().trim();
  window.loadMessages();
};

// ── Список контактов с аватарами ─────────────────────────────

function renderContactList(msgs) {
  const list = document.getElementById("contact-list");
  if (!list) return;

  if (msgs.length === 0) {
    list.innerHTML = '<p class="empty-state">Нет переписок</p>';
    return;
  }

  // Собираем уникальных собеседников
  const contactMap = new Map();
  msgs.forEach((m) => {
    const partnerKey = m.from_key === myPublicKey ? m.to_key : m.from_key;
    if (partnerKey === myPublicKey) return;

    if (!contactMap.has(partnerKey)) {
      contactMap.set(partnerKey, {
        key: partnerKey,
        lastMsg: m.content,
        lastTime: m.timestamp,
      });
    } else {
      const ex = contactMap.get(partnerKey);
      if (m.timestamp > ex.lastTime) {
        ex.lastMsg = m.content;
        ex.lastTime = m.timestamp;
      }
    }
  });

  if (contactMap.size === 0) {
    list.innerHTML = '<p class="empty-state">Нет переписок</p>';
    return;
  }

  let contacts = [...contactMap.values()].sort(
    (a, b) => b.lastTime - a.lastTime,
  );

  // Поиск по нику или ключу
  if (contactSearchQuery) {
    contacts = contacts.filter((c) => {
      const nick = window.getDisplayName
        ? window.getDisplayName(c.key).toLowerCase()
        : "";
      return (
        nick.includes(contactSearchQuery) ||
        c.key.toLowerCase().includes(contactSearchQuery)
      );
    });
  }

  if (contacts.length === 0) {
    list.innerHTML = '<p class="empty-state">Ничего не найдено</p>';
    return;
  }

  list.innerHTML = contacts
    .map((c) => {
      const displayName = window.getDisplayName
        ? window.getDisplayName(c.key)
        : c.key.slice(0, 8) + "…" + c.key.slice(-8);
      const avatarColor = window.getAvatarColor
        ? window.getAvatarColor(c.key)
        : "#2196f3";
      const avatarText = window.getAvatarText
        ? window.getAvatarText(c.key)
        : "??";
      const time = formatTime(c.lastTime);
      const isActive = c.key === currentChatKey ? "active" : "";
      const preview =
        escapeHtml(c.lastMsg.slice(0, 45)) + (c.lastMsg.length > 45 ? "…" : "");

      return `
            <div class="contact-item ${isActive}"
                 data-key="${escapeHtml(c.key)}"
                 onclick="selectChat('${escapeHtml(c.key)}')">
                <div class="contact-avatar" style="background:${avatarColor};">${avatarText}</div>
                <div class="contact-info">
                    <span class="contact-nick">${escapeHtml(displayName)}</span>
                    <span class="contact-preview">${preview}</span>
                </div>
                <div class="contact-meta">
                    <span class="contact-time">${time}</span>
                </div>
            </div>
        `;
    })
    .join("");
}

// ── Выбор чата ───────────────────────────────────────────────

window.selectChat = async function (key) {
  currentChatKey = key;

  const displayName = window.getDisplayName
    ? window.getDisplayName(key)
    : key.slice(0, 12) + "…" + key.slice(-12);
  const avatarColor = window.getAvatarColor
    ? window.getAvatarColor(key)
    : "#2196f3";
  const avatarText = window.getAvatarText ? window.getAvatarText(key) : "??";

  // Обновляем шапку
  document.getElementById("current-chat-name").textContent = displayName;
  document.getElementById("current-chat-key").textContent = key;
  document.getElementById("to-key").value = key;

  // Аватар в шапке
  const headerAvatar = document.getElementById("chat-header-avatar");
  if (headerAvatar) {
    headerAvatar.style.background = avatarColor;
    headerAvatar.textContent = avatarText;
  }

  // Кнопка редактирования ника
  const editBtn = document.getElementById("btn-edit-nick");
  if (editBtn) editBtn.style.display = "";

  const btnVideo = document.getElementById("btn-video-call");
  const btnAdd = document.getElementById("btn-add-contact");
  const btnMenu = document.getElementById("btn-chat-menu");
  if (btnVideo) btnVideo.style.display = "";
  if (btnAdd) btnAdd.style.display = "";
  if (btnMenu) btnMenu.style.display = "";

  // Активный контакт в списке
  document.querySelectorAll(".contact-item").forEach((el) => {
    el.classList.toggle("active", el.dataset.key === key);
  });

  const invoke = window.__TAURI__.core.invoke;
  try {
    const msgs = await invoke("get_incoming_messages");
    renderConversation(msgs, key);
  } catch (e) {
    console.log("selectChat error:", e);
  }
};

// ── Рендер переписки ─────────────────────────────────────────

function renderConversation(msgs, partnerKey) {
  const list = document.getElementById("msg-list");
  if (!list) return;

  const conversation = msgs.filter(
    (m) => m.from_key === partnerKey || m.to_key === partnerKey,
  );

  if (conversation.length === 0) {
    list.innerHTML = `
            <div class="empty-chat-state">
                <span class="empty-chat-icon">⬡</span>
                <p>Нет сообщений<br>Напишите первым</p>
            </div>`;
    return;
  }

  const sorted = [...conversation].sort((a, b) => a.timestamp - b.timestamp);

  let lastDate = null;
  const html = sorted
    .map((m) => {
      const isOutgoing = m.from_key === myPublicKey;
      const direction = isOutgoing ? "outgoing" : "incoming";

      const time = new Date(m.timestamp * 1000).toLocaleTimeString("ru-RU", {
        hour: "2-digit",
        minute: "2-digit",
      });

      const msgDate = new Date(m.timestamp * 1000).toLocaleDateString("ru-RU", {
        day: "2-digit",
        month: "long",
      });
      let divider = "";
      if (msgDate !== lastDate) {
        lastDate = msgDate;
        divider = `<div class="date-divider"><span>${msgDate}</span></div>`;
      }

      // Статус сообщения (для исходящих)
      let statusHtml = "";
      if (isOutgoing) {
        const status = m.status || "sent";
        const icons = {
          sent: { icon: "◻", cls: "msg-status-sent", title: "Отправлено" },
          pending: { icon: "⏳", cls: "msg-status-pending", title: "Ожидание" },
          delivered: {
            icon: "✓",
            cls: "msg-status-delivered",
            title: "Доставлено",
          },
          error: { icon: "⚠", cls: "msg-status-error", title: "Ошибка" },
        };
        const s = icons[status] || icons.sent;
        statusHtml = `<span class="msg-status-icon ${s.cls}" title="${s.title}">${s.icon}</span>`;
      }

      // Имя отправителя для входящих
      const senderNick =
        !isOutgoing && window.getDisplayName
          ? window.getDisplayName(m.from_key)
          : "";
      const senderLine = !isOutgoing
        ? `<div class="msg-sender">${escapeHtml(senderNick)}</div>`
        : "";

      return `
            ${divider}
            <div class="msg-wrapper ${direction}">
                ${senderLine}
                <div class="msg-bubble">${renderMarkdown(m.content)}</div>
                <div class="msg-meta">
                    <span class="msg-time">${time}</span>
                    ${statusHtml}
                </div>
            </div>
        `;
    })
    .join("");

  list.innerHTML = html;
  list.scrollTop = list.scrollHeight;
}

// ── Отправка ─────────────────────────────────────────────────

window.sendMessage = async function () {
  const invoke = window.__TAURI__.core.invoke;
  const toKey = document.getElementById("to-key").value.trim();
  const content = document.getElementById("msg-content").value.trim();

  if (!toKey) {
    window.log("Выберите контакт или введите ключ получателя", "error");
    return;
  }
  if (!content) {
    window.log("Введите текст сообщения", "error");
    return;
  }

  try {
    const id = await invoke("send_message", { toKey, content });
    window.log("Отправлено (ID: " + id + ")", "success");

    const textarea = document.getElementById("msg-content");
    textarea.value = "";
    textarea.style.height = "auto";

    if (currentChatKey !== toKey) {
      currentChatKey = toKey;
      document.getElementById("to-key").value = toKey;
    }

    await window.loadMessages();
  } catch (e) {
    window.log("Ошибка отправки: " + e, "error");
  }
};

// ── Новый чат ────────────────────────────────────────────────

window.showNewChat = function () {
  const form = document.getElementById("new-chat-form");
  const isVisible = form.style.display !== "none";
  form.style.display = isVisible ? "none" : "flex";
  if (!isVisible) document.getElementById("to-key").focus();
};

window.startChatWithKey = async function () {
  const key = document.getElementById("to-key").value.trim();
  const nick = document.getElementById("new-nick-input")?.value.trim() || "";

  if (!key) {
    window.log("Введите публичный ключ", "error");
    return;
  }
  if (key === myPublicKey) {
    window.log("Нельзя открыть чат с самим собой", "error");
    return;
  }

  // Сохраняем в localStorage для отображения
  if (nick && window.saveNickOnCreate) window.saveNickOnCreate(key, nick);

  // Сохраняем в базу данных
  if (window.addContactToDb) await window.addContactToDb(key, nick);

  document.getElementById("new-chat-form").style.display = "none";
  document.getElementById("to-key").value = "";
  if (document.getElementById("new-nick-input"))
    document.getElementById("new-nick-input").value = "";

  window.selectChat(key);
};

// ── Утилиты ──────────────────────────────────────────────────

window.autoResizeTextarea = function (el) {
  el.style.height = "auto";
  el.style.height = Math.min(el.scrollHeight, 120) + "px";
};

function formatTime(timestamp) {
  const date = new Date(timestamp * 1000);
  const now = new Date();
  if (date.toDateString() === now.toDateString()) {
    return date.toLocaleTimeString("ru-RU", {
      hour: "2-digit",
      minute: "2-digit",
    });
  }
  return date.toLocaleDateString("ru-RU", { day: "2-digit", month: "2-digit" });
}

function escapeHtml(text) {
  const div = document.createElement("div");
  div.textContent = text;
  return div.innerHTML;
}

function renderMarkdown(text) {
    if (typeof marked === 'undefined' || !window.isMarkdownEnabled || !window.isMarkdownEnabled()) {
        return escapeHtml(text);
    }
    try {
        return marked.parse(text, {
            breaks: true,
            gfm: true,
        });
    } catch {
        return escapeHtml(text);
    }
}
