// Глобальные переменные для таймеров
let p2pInterval, msgInterval;

// ==========================================
// ЛОГИРОВАНИЕ
// ==========================================
function log(msg, type = 'info') {
    const div = document.getElementById('log');
    if (div) {
        const entry = document.createElement('div');
        entry.className = `log-entry log-${type}`;
        entry.textContent = `[${new Date().toLocaleTimeString()}] ${msg}`;
        div.appendChild(entry);
        div.scrollTop = div.scrollHeight;
    }

    const initLog = document.getElementById('init-log');
    if (initLog) {
        const colors = { info: '#00a8c8', success: '#51cf66', error: '#ff6b6b' };
        initLog.style.color = colors[type] || '#eee';
        initLog.textContent = `[${new Date().toLocaleTimeString()}] ${msg}`;
    }
}

window.log = log;

// ==========================================
// НАВИГАЦИЯ
// ==========================================
function showTab(tabName, btnElement) {
    document.querySelectorAll('.tab').forEach(el => el.style.display = 'none');
    const targetTab = document.getElementById('tab-' + tabName);
    if (targetTab) targetTab.style.display = tabName === 'chat' ? 'flex' : 'flex';

    document.querySelectorAll('.nav-btn').forEach(b => b.classList.remove('active'));
    if (btnElement) btnElement.classList.add('active');
}

// Переход от экрана входа к экрану приложения
window.enterApp = async function() {
    document.getElementById('screen-init').classList.remove('active');
    document.getElementById('screen-app').classList.add('active');
    document.getElementById('main-nav').style.display = 'flex';

    // Показываем первую вкладку (Сеть)
    showTab('network', document.querySelector('.nav-btn'));

    // Загружаем начальные данные
    loadPublicKey();
    loadVersion();

    // Инициализируем чат — получаем свой ключ и загружаем сообщения
    // ВАЖНО: initMessaging должен быть после loadPublicKey
    if (typeof window.initMessaging === 'function') {
        await window.initMessaging();
    }

    // Запускаем фоновое обновление
    startAutoRefresh();
}

// ==========================================
// ВЫХОД ИЗ АККАУНТА
// ==========================================
window.logout = async function() {
    if (!confirm('Выйти из аккаунта? База данных будет закрыта.')) return;

    clearInterval(p2pInterval);
    clearInterval(msgInterval);

    try {
        const invoke = window.__TAURI__.core.invoke;
        await invoke('stop_p2p_node').catch(() => {});
        await invoke('lock_database');
    } catch (e) {
        console.log('logout error:', e);
    }

    // Возврат на экран входа
    document.getElementById('screen-app').classList.remove('active');
    document.getElementById('screen-init').classList.add('active');
    document.getElementById('main-nav').style.display = 'none';

    // Очищаем UI
    document.getElementById('db-password').value = '';
    document.getElementById('my-pubkey').textContent = 'Загрузка...';
    document.getElementById('settings-pubkey').textContent = '—';
    document.getElementById('p2p-status-text').textContent = 'Остановлен';
    document.getElementById('peer-count-text').textContent = '0 пиров';

    // Сбрасываем чат
    document.getElementById('msg-list').innerHTML = `
        <div class="empty-chat-state">
            <span class="empty-chat-icon">⬡</span>
            <p>Выберите контакт слева<br>или начните новый чат</p>
        </div>
    `;
    document.getElementById('contact-list').innerHTML =
        '<p class="empty-state">Нет переписок</p>';
    document.getElementById('current-chat-name').textContent = 'Выберите чат';
    document.getElementById('current-chat-key').textContent = '';
    document.getElementById('to-key').value = '';
    document.getElementById('msg-content').value = '';

    log('Выход выполнен', 'info');
}

// ==========================================
// ЗАГРУЗКА ДАННЫХ
// ==========================================
async function loadPublicKey() {
    const invoke = window.__TAURI__.core.invoke;
    try {
        const key = await invoke('get_public_key');
        document.getElementById('my-pubkey').textContent = key;
        document.getElementById('settings-pubkey').textContent = key;
    } catch (e) {
        console.log('Ошибка загрузки ключа:', e);
    }
}

async function loadVersion() {
    const invoke = window.__TAURI__.core.invoke;
    try {
        const v = await invoke('get_version');
        document.getElementById('version-text').textContent = 'Версия: ' + v;
    } catch (e) {}
}

window.copyPubkey = function() {
    const key = document.getElementById('my-pubkey').textContent;
    if (key && key !== 'Загрузка...') {
        navigator.clipboard.writeText(key);
        log('Ключ скопирован', 'success');
    }
}

window.copySettingsPubkey = function() {
    const key = document.getElementById('settings-pubkey').textContent;
    if (key && key !== '—') {
        navigator.clipboard.writeText(key);
        log('Ключ скопирован', 'success');
    }
}

// Вставить свой ключ в поле получателя
window.pasteMyKey = function() {
    const key = document.getElementById('my-pubkey').textContent;
    if (key && key !== 'Загрузка...') {
        document.getElementById('to-key').value = key;
        log('Свой ключ вставлен (тест-режим)', 'info');
    }
}

// ==========================================
// ФОНОВОЕ ОБНОВЛЕНИЕ
// ==========================================
function startAutoRefresh() {
    clearInterval(p2pInterval);
    clearInterval(msgInterval);

    p2pInterval = setInterval(window.refreshP2PStatus, 3000);
    // Каждые 5 секунд обновляем сообщения
    msgInterval = setInterval(window.loadMessages, 5000);
}
