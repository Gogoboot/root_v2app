// Импорт Tauri API (Стандарт для v2)
//import { invoke } from '@tauri-apps/api/core';

// Глобальные переменные для таймеров (чтобы можно было их остановить)
let p2pInterval, msgInterval;

// ==========================================
// ЛОГИРОВАНИЕ
// ==========================================
function log(msg, type = 'info') {
    // 1. Лог в основном окне (P2P экран)
    const div = document.getElementById('log');
    if (div) {
        const entry = document.createElement('div');
        entry.className = `log-entry log-${type}`;
        entry.textContent = `[${new Date().toLocaleTimeString()}] ${msg}`;
        div.appendChild(entry);
        div.scrollTop = div.scrollHeight; // Автоскролл вниз
    }

    // 2. Лог на экране входа (init-log)
    const initLog = document.getElementById('init-log');
    if (initLog) {
        const colors = { info: '#00d9ff', success: '#51cf66', error: '#ff6b6b' };
        initLog.style.color = colors[type] || '#eee';
        initLog.textContent = `[${new Date().toLocaleTimeString()}] ${msg}`;
    }
}

// Делаем log доступным глобально для других файлов
window.log = log;

// ==========================================
// НАВИГАЦИЯ
// ==========================================
// Переключает вкладки внутри приложения (Сеть / Чат / Настройки)
function showTab(tabName, btnElement) {
    // Скрываем все вкладки
    document.querySelectorAll('.tab').forEach(el => el.style.display = 'none');
    // Показываем нужную
    const targetTab = document.getElementById('tab-' + tabName);
    if (targetTab) targetTab.style.display = 'block';

    // Обновляем кнопки
    document.querySelectorAll('.nav-btn').forEach(b => b.classList.remove('active'));
    if (btnElement) btnElement.classList.add('active');
}

// Переход от Экрана Входа к Экрану Приложения
window.enterApp = function() {
    document.getElementById('screen-init').classList.remove('active');
    document.getElementById('screen-app').classList.add('active');
    document.getElementById('main-nav').style.display = 'flex';
    
    // Загружаем начальные данные
    loadPublicKey();
    loadVersion();
    
    // Запускаем фоновое обновление
    startAutoRefresh();
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
        // Если ключа нет (аккаунт не создан), это нормально
        console.log("Ключ еще не создан");
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
    if(key && key !== 'Загрузка...') {
        navigator.clipboard.writeText(key);
        log('Ключ скопирован', 'success');
    }
}

// ==========================================
// ФОНОВОЕ ОБНОВЛЕНИЕ
// ==========================================
function startAutoRefresh() {
    // Очищаем старые таймеры, если были (защита от дублирования)
    clearInterval(p2pInterval);
    clearInterval(msgInterval);

    // Запускаем новые
    p2pInterval = setInterval(window.refreshP2PStatus, 3000);
    msgInterval = setInterval(window.loadMessages, 5000);
}
