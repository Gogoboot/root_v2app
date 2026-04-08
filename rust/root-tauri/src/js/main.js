// ============================================================
// ROOT v2.0 — js/main.js
// Навигация, логирование, фоновое обновление
// ============================================================

let p2pInterval, msgInterval;

// ── Логирование ──────────────────────────────────────────────

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

// ── Навигация ────────────────────────────────────────────────

function showTab(tabName, btnElement) {
    document.querySelectorAll('.tab').forEach(el => el.style.display = 'none');
    const targetTab = document.getElementById('tab-' + tabName);
    if (targetTab) targetTab.style.display = 'flex';

    document.querySelectorAll('.nav-btn').forEach(b => b.classList.remove('active'));
    if (btnElement) btnElement.classList.add('active');

    // Загружаем данные при открытии конкретных вкладок
    if (tabName === 'settings') {
        window.loadBootstrapList && window.loadBootstrapList();
        window.loadDbPath && window.loadDbPath();
    }
    if (tabName === 'network') {
        window.refreshP2PStatus && window.refreshP2PStatus();
    }
}

window.showTab = showTab;

// Переход от экрана входа к экрану приложения
window.enterApp = async function() {
    document.getElementById('screen-init').classList.remove('active');
    document.getElementById('screen-app').classList.add('active');
    document.getElementById('main-nav').style.display = 'flex';

    showTab('network', document.querySelector('.nav-btn'));

    loadPublicKey();
    loadVersion();

    if (typeof window.initMessaging === 'function') {
        await window.initMessaging();
    }

    startAutoRefresh();
}

// ── Выход ────────────────────────────────────────────────────

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

    // Очистка UI
    document.getElementById('db-password').value = '';
    document.getElementById('my-pubkey').textContent = 'Загрузка...';
    document.getElementById('settings-pubkey').textContent = '—';
    document.getElementById('db-path-display').textContent = '—';
    document.getElementById('p2p-status-text').textContent = 'Остановлен';
    document.getElementById('peer-count-text').textContent = '0 пиров';
    document.getElementById('peer-list').innerHTML = '<p class="empty-state">Нет активных соединений</p>';
    document.getElementById('bootstrap-list').innerHTML = '<p class="empty-state">Нет bootstrap узлов</p>';

    document.getElementById('msg-list').innerHTML = `
        <div class="empty-chat-state">
            <span class="empty-chat-icon">⬡</span>
            <p>Выберите контакт слева<br>или начните новый чат</p>
        </div>
    `;
    document.getElementById('contact-list').innerHTML = '<p class="empty-state">Нет переписок</p>';
    document.getElementById('current-chat-name').textContent = 'Выберите чат';
    document.getElementById('current-chat-key').textContent = '';
    document.getElementById('to-key').value = '';
    document.getElementById('msg-content').value = '';

    log('Выход выполнен', 'info');
}

// ── Данные ───────────────────────────────────────────────────

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

window.pasteMyKey = function() {
    const key = document.getElementById('my-pubkey').textContent;
    if (key && key !== 'Загрузка...') {
        document.getElementById('to-key').value = key;
        log('Свой ключ вставлен (тест-режим)', 'info');
    }
}

// ── Фоновое обновление ───────────────────────────────────────

function startAutoRefresh() {
    clearInterval(p2pInterval);
    clearInterval(msgInterval);

    // P2P статус и список пиров — каждые 3 секунды
    p2pInterval = setInterval(window.refreshP2PStatus, 3000);
    // Сообщения — каждые 5 секунд
    msgInterval = setInterval(window.loadMessages, 5000);
}
