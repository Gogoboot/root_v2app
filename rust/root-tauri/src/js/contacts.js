// ============================================================
// contacts.js — ники и описания контактов
// Хранение: localStorage (ключ → { nick, desc })
// ============================================================

// Получить ник для публичного ключа (или null)
window.getContactNick = function(pubkey) {
    if (!pubkey) return null;
    try {
        const data = localStorage.getItem('contact:' + pubkey);
        if (!data) return null;
        return JSON.parse(data);
    } catch {
        return null;
    }
}

// Получить отображаемое имя: ник или сокращённый ключ
window.getDisplayName = function(pubkey) {
    const contact = window.getContactNick(pubkey);
    if (contact && contact.nick) return contact.nick;
    if (!pubkey) return '—';
    return pubkey.slice(0, 8) + '…' + pubkey.slice(-8);
}

// Сохранить ник и описание
window.saveContactNick = function(pubkey, nick, desc) {
    if (!pubkey) return;
    try {
        localStorage.setItem('contact:' + pubkey, JSON.stringify({ nick: nick.trim(), desc: (desc || '').trim() }));
    } catch (e) {
        console.log('saveContactNick error:', e);
    }
}

// ── Модалка редактирования ───────────────────────────────────

// Открыть модалку для текущего выбранного чата
window.openNickModal = function() {
    const key = window._currentChatKey;
    if (!key) return;

    const overlay = document.getElementById('nick-modal-overlay');
    const nickInput = document.getElementById('nick-input');
    const descInput = document.getElementById('desc-input');

    const existing = window.getContactNick(key);
    nickInput.value = existing ? existing.nick : '';
    descInput.value = existing ? existing.desc : '';

    overlay.classList.add('visible');
    nickInput.focus();
}

// Закрытие по клику на оверлей (не на само окно)
window.closeNickModal = function(event) {
    if (event.target.id === 'nick-modal-overlay') {
        closeNickModalDirect();
    }
}

window.closeNickModalDirect = function() {
    document.getElementById('nick-modal-overlay').classList.remove('visible');
}

// Сохранить и применить
window.saveNick = function() {
    const key = window._currentChatKey;
    if (!key) return;

    const nick = document.getElementById('nick-input').value.trim();
    const desc = document.getElementById('desc-input').value.trim();

    window.saveContactNick(key, nick, desc);

    // Обновляем шапку чата
    const displayName = nick || window.getDisplayName(key);
    document.getElementById('current-chat-name').textContent = displayName;

    // Перерисовываем список контактов
    window.loadMessages && window.loadMessages();

    closeNickModalDirect();
    window.log('Контакт сохранён: ' + (nick || 'без имени'), 'success');
}

// Если ник задан при добавлении нового чата — сразу сохраняем
window.saveNickOnCreate = function(pubkey, nick) {
    if (!pubkey || !nick) return;
    window.saveContactNick(pubkey, nick, '');
}
