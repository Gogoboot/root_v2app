// ==========================================
// ОТПРАВКА СООБЩЕНИЯ
// ==========================================
window.sendMessage = async function() {
    const invoke = window.__TAURI__.core.invoke;
    const toKey = document.getElementById('to-key').value.trim();
    const content = document.getElementById('msg-content').value.trim();

    if (!toKey || !content) {
        window.log('Заполните все поля', 'error');
        return;
    }

    try {
        // ВАЖНО: Tauri конвертирует to_key (Rust snake_case) → toKey (JS camelCase)
        const id = await invoke('send_message', { toKey, content });
        window.log('Сообщение отправлено (ID: ' + id + ')', 'success');
        document.getElementById('msg-content').value = '';
        // Сразу обновляем входящие (вдруг отправили себе — для теста)
        window.loadMessages();
    } catch (e) {
        window.log('Ошибка отправки: ' + e, 'error');
    }
}

// ==========================================
// ЗАГРУЗКА ВХОДЯЩИХ СООБЩЕНИЙ
// ==========================================
window.loadMessages = async function() {
    const invoke = window.__TAURI__.core.invoke;
    try {
        const msgs = await invoke('get_incoming_messages');
        const list = document.getElementById('msg-list');
        const counter = document.getElementById('msg-count');

        if (msgs.length === 0) {
            list.innerHTML = '<p style="color: #555; text-align: center; padding: 20px 0;">Нет сообщений</p>';
            if (counter) counter.textContent = '';
            return;
        }

        // Обновляем счётчик
        if (counter) counter.textContent = `${msgs.length} сообщений`;

        // Сортируем от старых к новым
        const sorted = [...msgs].sort((a, b) => a.timestamp - b.timestamp);

        list.innerHTML = sorted.map(m => {
            const time = new Date(m.timestamp * 1000).toLocaleString('ru-RU', {
                hour: '2-digit',
                minute: '2-digit',
                day: '2-digit',
                month: '2-digit'
            });
            // Сокращаем ключ отправителя для читаемости
            const shortKey = m.from_key.length > 16
                ? m.from_key.slice(0, 8) + '…' + m.from_key.slice(-8)
                : m.from_key;

            return `
                <div class="msg-item">
                    <div class="msg-header">
                        <span class="msg-from" title="${escapeHtml(m.from_key)}">
                            От: ${escapeHtml(shortKey)}
                        </span>
                        <span class="msg-time">${time}</span>
                    </div>
                    <div class="msg-body">${escapeHtml(m.content)}</div>
                </div>
            `;
        }).join('');

        // Автоскролл вниз (к последнему сообщению)
        list.scrollTop = list.scrollHeight;

    } catch (e) {
        // Тихая ошибка — фоновый опрос не должен шуметь в логе
        console.log('loadMessages error:', e);
    }
}

// ==========================================
// УТИЛИТЫ
// ==========================================

// Защита от XSS — экранирует HTML-символы
function escapeHtml(text) {
    const div = document.createElement('div');
    div.textContent = text;
    return div.innerHTML;
}
