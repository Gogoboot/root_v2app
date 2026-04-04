// ============================================================
// ROOT v2.0 — p2p.js
// Управление P2P сетью: запуск, остановка, статус
// ============================================================

// ── Запуск P2P узла ───────────────────────────────────────────────────────────

/**
 * Запускает P2P узел в сети libp2p.
 *
 * После запуска:
 * - Узел начинает искать пиров через mDNS (локальная сеть)
 * - Узел подключается к Bootstrap relay если настроен
 * - Начинается приём входящих сообщений через Gossipsub
 *
 * Возвращает PeerID этого устройства.
 */
async function startP2P() {
    try {
        // Вызываем Rust команду start_p2p_node
        // Rust запустит libp2p в отдельном Tokio потоке
        const peerId = await invoke('start_p2p_node');

        log(`P2P узел запущен. PeerID: ${peerId.slice(0, 20)}...`, 'success');

        // Обновляем статус сразу после запуска
        refreshP2PStatus();

    } catch (e) {
        // Возможные причины: порт занят, P2P уже запущен,
        // identity не инициализирована
        log(`Ошибка запуска P2P: ${e}`, 'error');
    }
}

// ── Остановка P2P узла ────────────────────────────────────────────────────────

/**
 * Останавливает P2P узел.
 *
 * После остановки:
 * - Все P2P соединения закрываются
 * - Входящие сообщения больше не принимаются
 * - БД и identity остаются доступными
 */
async function stopP2P() {
    try {
        // Вызываем Rust команду stop_p2p_node
        // Rust отправит сигнал остановки через oneshot канал
        await invoke('stop_p2p_node');

        log('P2P узел остановлен', 'info');

        // Обновляем статус после остановки
        refreshP2PStatus();

    } catch (e) {
        log(`Ошибка остановки P2P: ${e}`, 'error');
    }
}

// ── Статус P2P ────────────────────────────────────────────────────────────────

/**
 * Обновляет отображение статуса P2P узла.
 *
 * Вызывается:
 * - После запуска/остановки узла
 * - Автоматически каждые 3 секунды через startAutoRefresh()
 */
async function refreshP2PStatus() {
    try {
        // Запрашиваем статус и количество пиров параллельно
        const running = await invoke('get_p2p_status');
        const count   = await invoke('get_peer_count');

        // Обновляем бейдж статуса
        const badge = document.getElementById('p2p-badge');
        const dot   = document.getElementById('p2p-dot');
        const text  = document.getElementById('p2p-status-text');
        const peers = document.getElementById('peer-count-text');

        if (running) {
            // Узел активен — зелёный бейдж
            badge.className  = 'status-badge active';
            dot.className    = 'dot green';
            text.textContent = 'Активен';
        } else {
            // Узел остановлен — красный бейдж
            badge.className  = 'status-badge inactive';
            dot.className    = 'dot red';
            text.textContent = 'Остановлен';
        }

        // Обновляем счётчик пиров с правильным склонением
        peers.textContent = `${count} ${peerWord(count)}`;

    } catch (e) {
        // Ошибку статуса не показываем в лог — она вызывается часто
        // и может мусорить при незапущенном P2P
    }
}

// ── Вспомогательные функции ───────────────────────────────────────────────────

/**
 * Возвращает правильное склонение слова "пир" для русского языка.
 *
 * Примеры:
 *   1  → "пир"
 *   2  → "пира"
 *   5  → "пиров"
 *   11 → "пиров"
 *
 * @param {number} n - количество пиров
 * @returns {string} - правильная форма слова
 */
function peerWord(n) {
    // Особый случай: 11-14 всегда "пиров"
    if (n % 100 >= 11 && n % 100 <= 14) return 'пиров';

    // Остаток от деления на 10 определяет форму
    const rem = n % 10;
    if (rem === 1)              return 'пир';
    if (rem >= 2 && rem <= 4)  return 'пира';
    return 'пиров';
}
