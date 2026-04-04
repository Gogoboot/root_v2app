// ============================================================
// ROOT v2.0 — main.js
// Инициализация приложения, навигация, лог событий
// ============================================================

// Tauri API — функция для вызова Rust команд
const { invoke } = window.__TAURI__.core;

// ── Навигация ─────────────────────────────────────────────────────────────────

/**
 * Переключает активный экран.
 * @param {string} name - имя экрана ('p2p', 'msg', 'settings')
 * @param {HTMLElement} btn - кнопка навигации которую нажали
 */
function showScreen(name, btn) {
    // Скрываем все экраны
    document.querySelectorAll('.screen').forEach(s => s.classList.remove('active'));

    // Показываем нужный экран
    document.getElementById('screen-' + name).classList.add('active');

    // Убираем активный класс со всех кнопок навигации
    document.querySelectorAll('.nav-btn').forEach(b => b.classList.remove('active'));

    // Помечаем нажатую кнопку как активную
    if (btn) btn.classList.add('active');
}

/**
 * Переходим в основной интерфейс после успешного входа.
 * Показывает навигацию и загружает начальные данные.
 */
function enterApp() {
    // Скрываем экран инициализации
    document.getElementById('screen-init').classList.remove('active');

    // Показываем навигацию внизу
    document.getElementById('main-nav').style.display = 'flex';

    // Открываем экран P2P по умолчанию
    showScreen('p2p', document.querySelector('.nav-btn'));

    // Загружаем публичный ключ и версию
    loadPublicKey();
    loadVersion();

    // Запускаем автообновление статуса P2P и сообщений
    startAutoRefresh();
}

// ── Публичный ключ ────────────────────────────────────────────────────────────

/**
 * Загружает публичный ключ из Rust и показывает на экранах P2P и Настройки.
 */
async function loadPublicKey() {
    try {
        const key = await invoke('get_public_key');

        // Показываем ключ в двух местах
        document.getElementById('my-pubkey').textContent      = key;
        document.getElementById('settings-pubkey').textContent = key;
    } catch (e) {
        // Ключ может быть недоступен если identity не создана — это нормально
        log('Публичный ключ недоступен', 'error');
    }
}

/**
 * Копирует публичный ключ в буфер обмена.
 */
function copyPubkey() {
    const key = document.getElementById('my-pubkey').textContent;
    navigator.clipboard.writeText(key);
    log('Ключ скопирован в буфер обмена', 'info');
}

// ── Версия ────────────────────────────────────────────────────────────────────

/**
 * Загружает версию приложения из Rust.
 */
async function loadVersion() {
    try {
        const v = await invoke('get_version');
        document.getElementById('version-text').textContent = v;
    } catch (e) {
        document.getElementById('version-text').textContent = 'Недоступно';
    }
}

// ── Лог событий ──────────────────────────────────────────────────────────────

/**
 * Добавляет запись в лог событий на экране P2P.
 *
 * @param {string} msg  - текст сообщения
 * @param {string} type - тип: 'info' | 'success' | 'error'
 */
function log(msg, type = 'info') {
    const div = document.getElementById('log');
    if (!div) return;

    // Создаём строку лога с временной меткой
    const entry = document.createElement('div');
    entry.className = `log-entry log-${type}`;
    entry.textContent = `[${new Date().toLocaleTimeString()}] ${msg}`;

    // Добавляем в конец лога
    div.appendChild(entry);

    // Прокручиваем вниз чтобы видеть последнее сообщение
    div.scrollTop = div.scrollHeight;
}

// ── Автообновление ────────────────────────────────────────────────────────────

/**
 * Запускает периодическое обновление статуса P2P и входящих сообщений.
 * Вызывается один раз после входа в приложение.
 */
function startAutoRefresh() {
    // Обновляем статус P2P каждые 3 секунды
    setInterval(refreshP2PStatus, 3000);

    // Проверяем новые сообщения каждые 5 секунд
    setInterval(loadMessages, 5000);
}

// ── Инициализация ─────────────────────────────────────────────────────────────

// Выполняется когда HTML страница полностью загружена
window.addEventListener('DOMContentLoaded', () => {
    log('ROOT Desktop готов', 'success');
});
