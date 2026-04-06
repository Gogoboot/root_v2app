// Глобальные переменные для таймеров (чтобы можно было их остановить)
let p2pInterval, msgInterval;

// ==========================================
// ЛОГИРОВАНИЕ
// ==========================================
function log(msg, type = 'info') {
    // Лог в основном окне (P2P экран)
    const div = document.getElementById('log');
    if (div) {
        const entry = document.createElement('div');
        entry.className = `log-entry log-${type}`;
        entry.textContent = `[${new Date().toLocaleTimeString()}] ${msg}`;
        div.appendChild(entry);
        div.scrollTop = div.scrollHeight;
    }

    // Лог на экране входа (init-log)
    const initLog = document.getElementById('init-log');
    if (initLog) {
        const colors = { info: '#00d9ff', success: '#51cf66', error: '#ff6b6b' };
        initLog.style.color = colors[type] || '#eee';
        initLog.textContent = `[${new Date().toLocaleTimeString()}] ${msg}`;
    }
}

window.log = log;

// ==========================================
// НАВИГАЦИЯ
// ==========================================
function showTab(tabName, btnElement) {
    // Скрываем все вкладки
    document.querySelectorAll('.tab').forEach(el => el.style.display = 'none');
    // Показываем нужную
    const targetTab = document.getElementById('tab-' + tabName);
    if (targetTab) targetTab.style.display = 'block';

    // Обновляем активную кнопку навигации
    document.querySelectorAll('.nav-btn').forEach(b => b.classList.remove('active'));
    if (btnElement) btnElement.classList.add('active');
}

// Переход от экрана входа к экрану приложения
window.enterApp = function() {
    document.getElementById('screen-init').classList.remove('active');
    document.getElementById('screen-app').classList.add('active');
    document.getElementById('main-nav').style.display = 'flex';

    // Показываем первую вкладку (Сеть)
    showTab('network', document.querySelector('.nav-btn'));

    // Загружаем начальные данные
    loadPublicKey();
    loadVersion();

    // Запускаем фоновое обновление
    startAutoRefresh();
}

// ==========================================
// ВЫХОД ИЗ АККАУНТА
// ==========================================
window.logout = async function() {
    if (!confirm('Выйти из аккаунта? База данных будет закрыта.')) return;

    // Останавливаем все фоновые таймеры
    clearInterval(p2pInterval);
    clearInterval(msgInterval);

    // Пробуем остановить P2P если запущен (тихо, без ошибки если не запущен)
    try {
        const invoke = window.__TAURI__.core.invoke;
        // Сначала останавливаем P2P если запущен
        await invoke('stop_p2p_node').catch(() => {});
        // Затем сбрасываем состояние Rust — это главное
        await invoke('lock_database');
    } catch (e) {
        console.log('logout error:', e);
    }

    // Возвращаемся на экран входа
    document.getElementById('screen-app').classList.remove('active');
    document.getElementById('screen-init').classList.add('active');
    document.getElementById('main-nav').style.display = 'none';

    // Очищаем поле пароля
    document.getElementById('db-password').value = '';

    // Сбрасываем отображение ключа
    document.getElementById('my-pubkey').textContent = 'Загрузка...';
    document.getElementById('settings-pubkey').textContent = '—';

    // Сбрасываем чат
    document.getElementById('msg-list').innerHTML =
        '<p style="color: #555; text-align: center; padding: 20px 0;">Нет сообщений</p>';
    document.getElementById('to-key').value = '';
    document.getElementById('msg-content').value = '';

    // Сбрасываем P2P статус
    document.getElementById('p2p-status-text').textContent = 'Остановлен';
    document.getElementById('peer-count-text').textContent = '0 пиров';

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
        document.getElementById('version-text').textContent = "Версия: " + v;
    } catch (e) {}
}

window.copyPubkey = function() {
    const key = document.getElementById('my-pubkey').textContent;
    if (key && key !== 'Загрузка...') {
        navigator.clipboard.writeText(key);
        log('Ключ скопирован', 'success');
    }
}

// Копирование ключа со страницы настроек
window.copySettingsPubkey = function() {
    const key = document.getElementById('settings-pubkey').textContent;
    if (key && key !== '—') {
        navigator.clipboard.writeText(key);
        log('Ключ скопирован', 'success');
    }
}

// Вставить свой ключ в поле получателя (удобно для самотестирования)
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
    // Очищаем старые таймеры (защита от дублирования при повторном входе)
    clearInterval(p2pInterval);
    clearInterval(msgInterval);

    p2pInterval = setInterval(window.refreshP2PStatus, 3000);
    msgInterval = setInterval(window.loadMessages, 5000);

    // Показываем индикатор автообновления
    const indicator = document.getElementById('msg-auto-indicator');
    if (indicator) indicator.textContent = 'авто: вкл ✓';
}
