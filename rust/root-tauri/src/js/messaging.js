// ============================================================
// messaging.js — чат, контакты, пузыри сообщений
// ============================================================

// Текущий выбранный собеседник (его публичный ключ)
let currentChatKey = null;

// Мой публичный ключ
let myPublicKey = null;

// Последний поисковый запрос
let contactSearchQuery = '';

// ── Экспортируем currentChatKey для contacts.js ──────────────
Object.defineProperty(window, '_currentChatKey', {
    get: () => currentChatKey,
    set: (v) => { currentChatKey = v; }
});

// ── Инициализация ────────────────────────────────────────────

window.initMessaging = async function() {
    const invoke = window.__TAURI__.core.invoke;
    try {
        myPublicKey = await invoke('get_public_key');
    } catch (e) {
        console.log('initMessaging: не удалось получить ключ', e);
    }
    await window.loadMessages();
}

// ── Загрузка и рендер сообщений ──────────────────────────────

window.loadMessages = async function() {
    const invoke = window.__TAURI__.core.invoke;
    try {
        const msgs = await invoke('get_incoming_messages');
        renderContactList(msgs);
        if (currentChatKey) {
            renderConversation(msgs, currentChatKey);
        }
    } catch (e) {
        console.log('loadMessages error:', e);
    }
}

// Фильтрация контактов по поисковому запросу
window.filterContacts = function(query) {
    contactSearchQuery = query.toLowerCase().trim();
    // Перерисовываем список с текущим фильтром
    window.loadMessages();
}

// Строим список уникальных контактов
function renderContactList(msgs) {
    const list = document.getElementById('contact-list');
    if (!list) return;

    if (msgs.length === 0) {
        list.innerHTML = '<p class="empty-state">Нет переписок</p>';
        return;
    }

    // Собираем уникальных собеседников
    const contactMap = new Map();

    msgs.forEach(m => {
        const partnerKey = m.from_key === myPublicKey ? m.to_key : m.from_key;
        if (partnerKey === myPublicKey) return;

        if (!contactMap.has(partnerKey)) {
            contactMap.set(partnerKey, {
                key: partnerKey,
                lastMsg: m.content,
                lastTime: m.timestamp,
            });
        } else {
            const existing = contactMap.get(partnerKey);
            if (m.timestamp > existing.lastTime) {
                existing.lastMsg = m.content;
                existing.lastTime = m.timestamp;
            }
        }
    });

    if (contactMap.size === 0) {
        list.innerHTML = '<p class="empty-state">Нет переписок</p>';
        return;
    }

    let contacts = [...contactMap.values()].sort((a, b) => b.lastTime - a.lastTime);

    // Применяем поиск — по нику или ключу
    if (contactSearchQuery) {
        contacts = contacts.filter(c => {
            const nick = window.getDisplayName ? window.getDisplayName(c.key).toLowerCase() : '';
            return nick.includes(contactSearchQuery) || c.key.toLowerCase().includes(contactSearchQuery);
        });
    }

    if (contacts.length === 0) {
        list.innerHTML = '<p class="empty-state">Ничего не найдено</p>';
        return;
    }

    list.innerHTML = contacts.map(c => {
        // Ник или сокращённый ключ
        const displayName = window.getDisplayName ? window.getDisplayName(c.key) : (c.key.slice(0, 8) + '…' + c.key.slice(-8));
        const hasNick = window.getContactNick && window.getContactNick(c.key)?.nick;

        // Сокращённый ключ — показываем под ником
        const shortKey = c.key.slice(0, 8) + '…' + c.key.slice(-8);

        const time = formatTime(c.lastTime);
        const isActive = c.key === currentChatKey ? 'active' : '';
        const preview = escapeHtml(c.lastMsg.slice(0, 40)) + (c.lastMsg.length > 40 ? '…' : '');

        return `
            <div class="contact-item ${isActive}"
                 data-key="${escapeHtml(c.key)}"
                 onclick="selectChat('${escapeHtml(c.key)}')">
                <div style="display:flex; justify-content:space-between; align-items:flex-start; gap:6px;">
                    <span class="contact-nick">${escapeHtml(displayName)}</span>
                    <span class="contact-time">${time}</span>
                </div>
                ${hasNick ? `<span class="contact-key-short">${shortKey}</span>` : ''}
                <span class="contact-preview">${preview}</span>
            </div>
        `;
    }).join('');
}

// Выбираем чат
window.selectChat = async function(key) {
    currentChatKey = key;

    // Имя: ник или ключ
    const displayName = window.getDisplayName ? window.getDisplayName(key) : (key.slice(0, 12) + '…' + key.slice(-12));

    document.getElementById('current-chat-name').textContent = displayName;
    document.getElementById('current-chat-key').textContent = key;
    document.getElementById('to-key').value = key;

    // Показываем кнопку редактирования ника
    const editBtn = document.getElementById('btn-edit-nick');
    if (editBtn) editBtn.style.display = '';

    // Активный контакт в списке
    document.querySelectorAll('.contact-item').forEach(el => {
        el.classList.toggle('active', el.dataset.key === key);
    });

    const invoke = window.__TAURI__.core.invoke;
    try {
        const msgs = await invoke('get_incoming_messages');
        renderConversation(msgs, key);
    } catch (e) {
        console.log('selectChat error:', e);
    }
}

// Рендерим переписку с пузырями и статусами
function renderConversation(msgs, partnerKey) {
    const list = document.getElementById('msg-list');
    if (!list) return;

    const conversation = msgs.filter(m =>
        m.from_key === partnerKey || m.to_key === partnerKey
    );

    if (conversation.length === 0) {
        list.innerHTML = `
            <div class="empty-chat-state">
                <span class="empty-chat-icon">⬡</span>
                <p>Нет сообщений<br>Напишите первым</p>
            </div>
        `;
        return;
    }

    const sorted = [...conversation].sort((a, b) => a.timestamp - b.timestamp);

    let lastDate = null;
    const html = sorted.map(m => {
        const isOutgoing = m.from_key === myPublicKey;
        const direction  = isOutgoing ? 'outgoing' : 'incoming';

        const time = new Date(m.timestamp * 1000).toLocaleTimeString('ru-RU', {
            hour: '2-digit', minute: '2-digit'
        });

        const msgDate = new Date(m.timestamp * 1000).toLocaleDateString('ru-RU', {
            day: '2-digit', month: 'long'
        });

        let divider = '';
        if (msgDate !== lastDate) {
            lastDate = msgDate;
            divider = `<div class="date-divider"><span>${msgDate}</span></div>`;
        }

        // Статус сообщения
        // Логика: исходящее = ◻ sent, у нас нет реальных ACK пока — показываем sent
        // TODO: когда появится delivery ACK — переключать на delivered/pending/error
        let statusHtml = '';
        if (isOutgoing) {
            const status = m.status || 'sent';
            const icons = {
                sent:      { icon: '◻', cls: 'msg-status-sent',      title: 'Отправлено' },
                pending:   { icon: '⏳', cls: 'msg-status-pending',   title: 'Ожидание доставки' },
                delivered: { icon: '✓', cls: 'msg-status-delivered',  title: 'Доставлено' },
                error:     { icon: '⚠', cls: 'msg-status-error',      title: 'Ошибка доставки' },
            };
            const s = icons[status] || icons.sent;
            statusHtml = `<span class="msg-status-icon ${s.cls}" title="${s.title}">${s.icon}</span>`;
        }

        // Имя отправителя для входящих
        const senderNick = !isOutgoing && window.getDisplayName
            ? window.getDisplayName(m.from_key)
            : '';
        const senderLine = !isOutgoing ? `<div class="msg-sender">${escapeHtml(senderNick)}</div>` : '';

        return `
            ${divider}
            <div class="msg-wrapper ${direction}">
                ${senderLine}
                <div class="msg-bubble">${escapeHtml(m.content)}</div>
                <div class="msg-meta">
                    <span class="msg-time">${time}</span>
                    ${statusHtml}
                </div>
            </div>
        `;
    }).join('');

    list.innerHTML = html;
    list.scrollTop = list.scrollHeight;
}

// ── Отправка сообщения ───────────────────────────────────────

window.sendMessage = async function() {
    const invoke = window.__TAURI__.core.invoke;
    const toKey   = document.getElementById('to-key').value.trim();
    const content = document.getElementById('msg-content').value.trim();

    if (!toKey) {
        window.log('Выберите контакт или введите ключ получателя', 'error');
        return;
    }
    if (!content) {
        window.log('Введите текст сообщения', 'error');
        return;
    }

    try {
        const id = await invoke('send_message', { toKey, content });
        window.log('Отправлено (ID: ' + id + ')', 'success');

        const textarea = document.getElementById('msg-content');
        textarea.value = '';
        textarea.style.height = 'auto';

        if (currentChatKey !== toKey) {
            currentChatKey = toKey;
            document.getElementById('to-key').value = toKey;
        }

        await window.loadMessages();

    } catch (e) {
        window.log('Ошибка отправки: ' + e, 'error');
    }
}

// ── Новый чат ────────────────────────────────────────────────

window.showNewChat = function() {
    const form = document.getElementById('new-chat-form');
    const isVisible = form.style.display !== 'none';
    form.style.display = isVisible ? 'none' : 'flex';
    if (!isVisible) {
        document.getElementById('to-key').focus();
    }
}

window.startChatWithKey = function() {
    const key  = document.getElementById('to-key').value.trim();
    const nick = document.getElementById('new-nick-input')?.value.trim() || '';

    if (!key) {
        window.log('Введите публичный ключ', 'error');
        return;
    }

    if (key === myPublicKey) {
        window.log('Нельзя открыть чат с самим собой', 'error');
        return;
    }

    // Если ник указан — сохраняем сразу
    if (nick && window.saveNickOnCreate) {
        window.saveNickOnCreate(key, nick);
    }

    document.getElementById('new-chat-form').style.display = 'none';
    document.getElementById('to-key').value = '';
    if (document.getElementById('new-nick-input')) {
        document.getElementById('new-nick-input').value = '';
    }

    window.selectChat(key);
}

// ── Утилиты ──────────────────────────────────────────────────

window.autoResizeTextarea = function(el) {
    el.style.height = 'auto';
    el.style.height = Math.min(el.scrollHeight, 120) + 'px';
}

function formatTime(timestamp) {
    const date = new Date(timestamp * 1000);
    const now  = new Date();
    const isToday = date.toDateString() === now.toDateString();
    if (isToday) {
        return date.toLocaleTimeString('ru-RU', { hour: '2-digit', minute: '2-digit' });
    }
    return date.toLocaleDateString('ru-RU', { day: '2-digit', month: '2-digit' });
}

function escapeHtml(text) {
    const div = document.createElement('div');
    div.textContent = text;
    return div.innerHTML;
}
