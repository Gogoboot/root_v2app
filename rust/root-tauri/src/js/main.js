// ============================================================
// ROOT v2.0 — js/main.js
// Навигация, темы, логирование, фоновое обновление
// ============================================================

let p2pInterval, msgInterval, diagInterval;

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
        const colors = { info: 'var(--accent-dim)', success: 'var(--green)', error: 'var(--red)', warn: 'var(--orange)' };
        initLog.style.color = colors[type] || 'var(--text-muted)';
        initLog.textContent = `[${new Date().toLocaleTimeString()}] ${msg}`;
    }
}

window.log = log;

// ── Темы ─────────────────────────────────────────────────────

// Применить тему
window.setTheme = function(theme, btnEl) {
    document.documentElement.setAttribute('data-theme', theme);
    localStorage.setItem('root-theme', theme);

    // Обновляем кнопки
    document.querySelectorAll('.theme-btn').forEach(b => b.classList.remove('active'));
    if (btnEl) {
        btnEl.classList.add('active');
    } else {
        // При загрузке без кнопки — найдём по id
        const btn = document.getElementById('theme-btn-' + theme);
        if (btn) btn.classList.add('active');
    }
}

// Восстановить тему из localStorage
function restoreTheme() {
    const saved = localStorage.getItem('root-theme') || 'dark';
    window.setTheme(saved, null);
}

// ── Показ/скрытие пароля ─────────────────────────────────────

window.togglePasswordVisibility = function() {
    const input = document.getElementById('db-password');
    const btn   = document.querySelector('.password-toggle');
    if (!input) return;

    if (input.type === 'password') {
        input.type = 'text';
        if (btn) btn.textContent = '🙈';
    } else {
        input.type = 'password';
        if (btn) btn.textContent = '👁';
    }
}

// ── Навигация ────────────────────────────────────────────────

function showTab(tabName, btnElement) {
    document.querySelectorAll('.tab').forEach(el => el.style.display = 'none');
    const targetTab = document.getElementById('tab-' + tabName);
    if (targetTab) {
        // chat-tab — flex, остальные — flex тоже но column
        targetTab.style.display = 'flex';
    }

    document.querySelectorAll('.nav-btn').forEach(b => b.classList.remove('active'));
    if (btnElement) btnElement.classList.add('active');

    // Загружаем данные при открытии вкладок
    if (tabName === 'settings') {
        window.loadBootstrapList && window.loadBootstrapList();
        window.loadDbPath && window.loadDbPath();
    }
    if (tabName === 'network') {
        window.refreshP2PStatus && window.refreshP2PStatus();
    }
    if (tabName === 'diag') {
        window.refreshDiagnostics && window.refreshDiagnostics();
    }
}

window.showTab = showTab;

// Переход от экрана входа к экрану приложения
window.enterApp = async function() {
    document.getElementById('screen-init').classList.remove('active');
    document.getElementById('screen-app').classList.add('active');
    document.getElementById('main-nav').style.display = 'flex';

    // Чат — первая вкладка по умолчанию
    showTab('chat', document.querySelector('.nav-btn'));

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
    clearInterval(diagInterval);

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

    // Сброс пароля и иконки
    const passInput = document.getElementById('db-password');
    if (passInput) { passInput.value = ''; passInput.type = 'password'; }
    const passToggle = document.querySelector('.password-toggle');
    if (passToggle) passToggle.textContent = '👁';

    // Очистка UI
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

    const editBtn = document.getElementById('btn-edit-nick');
    if (editBtn) editBtn.style.display = 'none';

    log('Выход выполнен', 'info');
}

// ── Данные ───────────────────────────────────────────────────

async function loadPublicKey() {
    const invoke = window.__TAURI__.core.invoke;
    try {
        const key = await invoke('get_public_key');
        document.getElementById('my-pubkey').textContent = key;
        document.getElementById('settings-pubkey').textContent = key;

        // Также в диагностику
        const diagKey = document.getElementById('diag-pubkey');
        if (diagKey) diagKey.textContent = key;
    } catch (e) {
        console.log('Ошибка загрузки ключа:', e);
    }
}

async function loadVersion() {
    const invoke = window.__TAURI__.core.invoke;
    try {
        const v = await invoke('get_version');
        document.getElementById('version-text').textContent = 'Версия: ' + v;
        const diagV = document.getElementById('diag-version');
        if (diagV) diagV.textContent = v;
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

// ── Фоновое обновление ───────────────────────────────────────

function startAutoRefresh() {
    clearInterval(p2pInterval);
    clearInterval(msgInterval);
    clearInterval(diagInterval);

    // P2P статус — каждые 3 секунды
    p2pInterval = setInterval(window.refreshP2PStatus, 3000);
    // Сообщения — каждые 5 секунд
    msgInterval = setInterval(window.loadMessages, 5000);
    // Диагностика — каждые 10 секунд (только метрики, лог пишется через window.log)
    diagInterval = setInterval(() => {
        // Обновляем только если вкладка диагностики активна
        const diagTab = document.getElementById('tab-diag');
        if (diagTab && diagTab.style.display !== 'none') {
            window.refreshDiagnostics && window.refreshDiagnostics();
        }
    }, 10000);
}

// ── Инициализация при загрузке ───────────────────────────────

document.addEventListener('DOMContentLoaded', () => {
    restoreTheme();
});
