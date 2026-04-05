//import { invoke } from '@tauri-apps/api/core';

window.sendMessage = async function() {
    const invoke = window.__TAURI__.core.invoke;
    const toKey = document.getElementById('to-key').value.trim();
    const content = document.getElementById('msg-content').value.trim();

    if (!toKey || !content) {
        window.log('Заполните все поля', 'error');
        return;
    }

    try {
        // ВАЖНО: to_key (snake_case)
        const id = await invoke('send_message', { to_key: toKey, content: content });
        window.log('Сообщение отправлено (ID: ' + id + ')', 'success');
        document.getElementById('msg-content').value = '';
        window.loadMessages();
    } catch (e) {
        window.log('Ошибка отправки: ' + e, 'error');
    }
}

window.loadMessages = async function() {
    const invoke = window.__TAURI__.core.invoke;
    try {
        const msgs = await invoke('get_incoming_messages');
        const list = document.getElementById('msg-list');
        
        if (msgs.length === 0) {
            list.innerHTML = '<p>Нет сообщений</p>';
            return;
        }

        list.innerHTML = msgs.map(m => `
            <div class="msg-item">
                <div class="from">От: ${m.from_key.slice(0, 12)}...</div>
                <div>${escapeHtml(m.content)}</div>
                <div style="font-size:10px; color:#555">${new Date(m.timestamp * 1000).toLocaleString()}</div>
            </div>
        `).join('');
        
        // Скролл вниз
        list.scrollTop = list.scrollHeight;
    } catch (e) {
        // Тихая ошибка для фонового опроса
    }
}

function escapeHtml(text) {
    const div = document.createElement('div');
    div.textContent = text;
    return div.innerHTML;
}
