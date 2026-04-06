// ============================================================
// messaging.js — чат, контакты, пузыри сообщений
// ============================================================

// Текущий выбранный собеседник (его публичный ключ)
let currentChatKey = null;

// Мой публичный ключ — нужен чтобы определить входящее/исходящее
let myPublicKey = null;

// ==========================================
// ИНИЦИАЛИЗАЦИЯ
// ==========================================

// Вызывается из main.js после входа
window.initMessaging = async function() {
    const invoke = window.__TAURI__.core.invoke;
    try {
        myPublicKey = await invoke('get_public_key');
    } catch (e) {
        console.log('initMessaging: не удалось получить ключ', e);
    }
    await window.loadMessages();
}

// ==========================================
// ЗАГРУЗКА И РЕНДЕР СООБЩЕНИЙ
// ==========================================

window.loadMessages = async function() {
    const invoke = window.__TAURI__.core.invoke;
    try {
        const msgs = await invoke('get_incoming_messages');

        // Обновляем список контактов в сайдбаре
        renderContactList(msgs);

        // Если есть выбранный чат — обновляем переписку
        if (currentChatKey) {
            renderConversation(msgs, currentChatKey);
        }
    } catch (e) {
        console.log('loadMessages error:', e);
    }
}

// Строим список уникальных контактов из всех сообщений
function renderContactList(msgs) {
    const list = document.getElementById('contact-list');
    if (!list) return;

    if (msgs.length === 0) {
        list.innerHTML = '<p class="empty-state">Нет переписок</p>';
        return;
    }

    // Собираем уникальных собеседников
    // Аналогия: из всей стопки писем достаём уникальные адреса
    const contactMap = new Map();

    msgs.forEach(m => {
        // Определяем кто собеседник — не я
        const partnerKey = m.from_key === myPublicKey ? m.to_key : m.from_key;

        // Пропускаем себя — чат с самим собой не показываем
        if (partnerKey === myPublicKey) return;

        if (!contactMap.has(partnerKey)) {
            contactMap.set(partnerKey, {
                key: partnerKey,
                lastMsg: m.content,
                lastTime: m.timestamp,
            });
        } else {
            // Обновляем если это более новое сообщение
            const existing = contactMap.get(partnerKey);
            if (m.timestamp > existing.lastTime) {
                existing.lastMsg = m.content;
                existing.lastTime = m.timestamp;
            }
        }
    });

    // Если после фильтрации контактов нет — показываем пустой экран
    if (contactMap.size === 0) {
        list.innerHTML = '<p class="empty-state">Нет переписок</p>';
        return;
    }

    // Сортируем по времени последнего сообщения (новые сверху)
    const contacts = [...contactMap.values()]
        .sort((a, b) => b.lastTime - a.lastTime);

    list.innerHTML = contacts.map(c => {
        // Сокращаем ключ для отображения: abcd1234…efgh5678
        const shortKey = c.key.length > 16
            ? c.key.slice(0, 8) + '…' + c.key.slice(-8)
            : c.key;

        const time = formatTime(c.lastTime);
        const isActive = c.key === currentChatKey ? 'active' : '';
        const preview = escapeHtml(c.lastMsg.slice(0, 40)) +
                        (c.lastMsg.length > 40 ? '…' : '');

        // data-key хранит полный ключ — используем при клике
        return `
            <div class="contact-item ${isActive}"
                 data-key="${escapeHtml(c.key)}"
                 onclick="selectChat('${escapeHtml(c.key)}')">
                <div style="display:flex; justify-content:space-between; align-items:center;">
                    <span class="contact-name">${shortKey}</span>
                    <span class="contact-time">${time}</span>
                </div>
                <span class="contact-preview">${preview}</span>
            </div>
        `;
    }).join('');
}

// Выбираем чат — показываем переписку с этим контактом
window.selectChat = async function(key) {
    currentChatKey = key;

    // Обновляем шапку чата
    const shortKey = key.slice(0, 12) + '…' + key.slice(-12);
    document.getElementById('current-chat-name').textContent = shortKey;
    document.getElementById('current-chat-key').textContent = key;

    // Вставляем ключ в скрытое поле to-key (для отправки)
    document.getElementById('to-key').value = key;

    // Обновляем активный контакт в списке
    document.querySelectorAll('.contact-item').forEach(el => {
        el.classList.toggle('active', el.dataset.key === key);
    });

    // Загружаем переписку
    const invoke = window.__TAURI__.core.invoke;
    try {
        const msgs = await invoke('get_incoming_messages');
        renderConversation(msgs, key);
    } catch (e) {
        console.log('selectChat error:', e);
    }
}

// Рендерим переписку с конкретным контактом в виде пузырей
function renderConversation(msgs, partnerKey) {
    const list = document.getElementById('msg-list');
    if (!list) return;

    // Фильтруем только сообщения с этим контактом
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

    // Сортируем от старых к новым (старые сверху)
    const sorted = [...conversation].sort((a, b) => a.timestamp - b.timestamp);

    // Группируем по датам — вставляем разделители
    let lastDate = null;
    const html = sorted.map(m => {
        const isOutgoing = m.from_key === myPublicKey;
        const direction  = isOutgoing ? 'outgoing' : 'incoming';

        const time = new Date(m.timestamp * 1000).toLocaleTimeString('ru-RU', {
            hour: '2-digit', minute: '2-digit'
        });

        // Дата для разделителя
        const msgDate = new Date(m.timestamp * 1000).toLocaleDateString('ru-RU', {
            day: '2-digit', month: 'long'
        });

        let divider = '';
        if (msgDate !== lastDate) {
            lastDate = msgDate;
            divider = `
                <div class="date-divider">
                    <span>${msgDate}</span>
                </div>
            `;
        }

        // Статус — пока просто "отправлено"
        // TODO: в v2.1 добавить реальные статусы доставки
        const statusIcon = isOutgoing ? '✓' : '';
        const statusClass = isOutgoing ? 'sent' : '';

        // Имя отправителя показываем только для входящих
        const senderLine = !isOutgoing ? `
            <div class="msg-sender">
                ${m.from_key.slice(0, 8)}…${m.from_key.slice(-8)}
            </div>
        ` : '';

        return `
            ${divider}
            <div class="msg-wrapper ${direction}">
                ${senderLine}
                <div class="msg-bubble">
                    ${escapeHtml(m.content)}
                </div>
                <div class="msg-meta">
                    <span class="msg-time">${time}</span>
                    ${statusIcon ? `<span class="msg-status ${statusClass}">${statusIcon}</span>` : ''}
                </div>
            </div>
        `;
    }).join('');

    list.innerHTML = html;

    // Автоскролл вниз — к последнему сообщению
    list.scrollTop = list.scrollHeight;
}

// ==========================================
// ОТПРАВКА СООБЩЕНИЯ
// ==========================================

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
        // Tauri конвертирует to_key (Rust snake_case) → toKey (JS camelCase)
        const id = await invoke('send_message', { toKey, content });
        window.log('Отправлено (ID: ' + id + ')', 'success');

        // Очищаем поле ввода
        const textarea = document.getElementById('msg-content');
        textarea.value = '';
        textarea.style.height = 'auto'; // сбрасываем высоту после авторесайза

        // Если чат с этим контактом не открыт — открываем
        if (currentChatKey !== toKey) {
            currentChatKey = toKey;
            document.getElementById('to-key').value = toKey;
        }

        // Сразу обновляем переписку
        await window.loadMessages();

    } catch (e) {
        window.log('Ошибка отправки: ' + e, 'error');
    }
}

// ==========================================
// НОВЫЙ ЧАТ
// ==========================================

// Показывает/скрывает форму ввода нового ключа
window.showNewChat = function() {
    const form = document.getElementById('new-chat-form');
    const isVisible = form.style.display !== 'none';
    form.style.display = isVisible ? 'none' : 'flex';
    if (!isVisible) {
        document.getElementById('to-key').focus();
    }
}

// Открывает чат по ключу из поля ввода
window.startChatWithKey = function() {
    const key = document.getElementById('to-key').value.trim();

    if (!key) {
        window.log('Введите публичный ключ', 'error');
        return;
    }

    // Защита от чата с самим собой
    if (key === myPublicKey) {
        window.log('Нельзя открыть чат с самим собой', 'error');
        return;
    }

    // Скрываем форму
    document.getElementById('new-chat-form').style.display = 'none';

    // Очищаем поле — ключ теперь хранится в currentChatKey
    document.getElementById('to-key').value = '';

    // Открываем чат
    window.selectChat(key);
}

// ==========================================
// УТИЛИТЫ
// ==========================================

// Автоматически увеличивает textarea при вводе
window.autoResizeTextarea = function(el) {
    el.style.height = 'auto';
    el.style.height = Math.min(el.scrollHeight, 120) + 'px';
}

// Форматирует timestamp в читаемое время
function formatTime(timestamp) {
    const date = new Date(timestamp * 1000);
    const now  = new Date();
    const isToday = date.toDateString() === now.toDateString();

    if (isToday) {
        return date.toLocaleTimeString('ru-RU', { hour: '2-digit', minute: '2-digit' });
    }
    return date.toLocaleDateString('ru-RU', { day: '2-digit', month: '2-digit' });
}

// Защита от XSS — экранирует HTML-символы
function escapeHtml(text) {
    const div = document.createElement('div');
    div.textContent = text;
    return div.innerHTML;
}
