// ============================================================
// diagnostics.js — вкладка диагностики
// ============================================================

// Все записи лога хранятся здесь для фильтрации
let diagLogEntries = [];
let currentLogFilter = 'all';

// ── Инициализация ────────────────────────────────────────────

// Перехватываем window.log чтобы писать и в diag-log тоже
const _originalLog = window.log;

window.log = function(msg, type = 'info') {
    // Оригинальный лог
    _originalLog(msg, type);

    // Добавляем в хранилище диагностики
    const entry = {
        time: new Date().toLocaleTimeString('ru-RU'),
        msg,
        type
    };
    diagLogEntries.push(entry);

    // Ограничиваем буфер — 500 записей
    if (diagLogEntries.length > 500) {
        diagLogEntries.shift();
    }

    // Если вкладка диагностики открыта — добавляем сразу
    renderDiagLogEntry(entry);
}

// Добавить одну запись в diag-log (с учётом фильтра)
function renderDiagLogEntry(entry) {
    const container = document.getElementById('diag-log');
    if (!container) return;

    // Применяем фильтр
    if (currentLogFilter !== 'all' && entry.type !== currentLogFilter) return;

    const el = document.createElement('div');
    el.className = `log-entry log-${entry.type}`;
    el.dataset.type = entry.type;
    el.textContent = `[${entry.time}] ${entry.msg}`;
    container.appendChild(el);
    container.scrollTop = container.scrollHeight;
}

// ── Фильтр уровня логов ──────────────────────────────────────

window.setLogFilter = function(filter, btnEl) {
    currentLogFilter = filter;

    // Обновляем кнопки
    document.querySelectorAll('.log-filter-btn').forEach(b => b.classList.remove('active'));
    if (btnEl) btnEl.classList.add('active');

    // Перерисовываем лог
    const container = document.getElementById('diag-log');
    if (!container) return;
    container.innerHTML = '';

    diagLogEntries.forEach(entry => {
        if (filter === 'all' || entry.type === filter) {
            const el = document.createElement('div');
            el.className = `log-entry log-${entry.type}`;
            el.textContent = `[${entry.time}] ${entry.msg}`;
            container.appendChild(el);
        }
    });

    container.scrollTop = container.scrollHeight;
}

// ── Обновление метрик ────────────────────────────────────────

window.refreshDiagnostics = async function() {
    const invoke = window.__TAURI__?.core?.invoke;
    if (!invoke) return;

    try {
        // P2P статус
        const running = await invoke('get_p2p_status').catch(() => false);
        const p2pEl = document.getElementById('diag-p2p-state');
        if (p2pEl) {
            p2pEl.textContent = running ? 'Активен' : 'Остановлен';
            p2pEl.className = 'diag-metric-value ' + (running ? 'ok' : 'error');
        }

        // Пиры
        const peers = await invoke('get_peers').catch(() => []);
        const peerCountEl = document.getElementById('diag-peer-count');
        if (peerCountEl) {
            peerCountEl.textContent = peers.length;
            peerCountEl.className = 'diag-metric-value ' + (peers.length > 0 ? 'ok' : 'warn');
        }

        // Список пиров (такой же как во вкладке Сеть)
        const peerListEl = document.getElementById('diag-peer-list');
        if (peerListEl) {
            if (peers.length === 0) {
                peerListEl.innerHTML = '<p class="empty-state">Нет активных соединений</p>';
            } else {
                peerListEl.innerHTML = peers.map(peer => {
                    const proto = peer.protocol.toLowerCase();
                    const shortId = peer.peer_id.length > 20
                        ? peer.peer_id.slice(0, 8) + '...' + peer.peer_id.slice(-6)
                        : peer.peer_id;
                    const now  = Math.floor(Date.now() / 1000);
                    const diff = now - peer.connected_at;
                    const time = diff < 60 ? `${diff}с` : diff < 3600 ? `${Math.floor(diff/60)}м` : `${Math.floor(diff/3600)}ч`;
                    return `<div class="peer-item">
                        <span class="peer-protocol ${proto}">${peer.protocol}</span>
                        <span class="peer-id" title="${peer.peer_id}">${shortId}</span>
                        <span class="peer-time">${time}</span>
                    </div>`;
                }).join('');
            }
        }

        // Bootstrap статус — смотрим есть ли хоть один адрес
        const bootstrapList = await invoke('get_bootstrap_list').catch(() => []);
        const bsEl = document.getElementById('diag-bootstrap-status');
        if (bsEl) {
            if (bootstrapList.length === 0) {
                bsEl.textContent = 'Не настроен';
                bsEl.className = 'diag-metric-value warn';
            } else {
                bsEl.textContent = running && peers.length > 0 ? 'Подключён' : 'Настроен';
                bsEl.className = 'diag-metric-value ' + (running && peers.length > 0 ? 'ok' : 'warn');
            }
        }

        // Публичный ключ
        const pubkeyEl = document.getElementById('diag-pubkey');
        if (pubkeyEl) {
            const key = await invoke('get_public_key').catch(() => '—');
            pubkeyEl.textContent = key;
        }

        // Путь к БД
        const dbPathEl = document.getElementById('diag-db-path');
        if (dbPathEl) {
            const path = await invoke('get_db_path').catch(() => '—');
            dbPathEl.textContent = path;
        }

        // Версия
        const versionEl = document.getElementById('diag-version');
        if (versionEl) {
            const v = await invoke('get_version').catch(() => '—');
            versionEl.textContent = v;
        }

    } catch (e) {
        console.log('refreshDiagnostics error:', e);
    }
}

// ── Копировать отчёт ─────────────────────────────────────────

window.copyDiagReport = async function() {
    const invoke = window.__TAURI__?.core?.invoke;
    const lines = [];

    lines.push('=== ROOT v2.0 Diagnostic Report ===');
    lines.push('Время: ' + new Date().toLocaleString('ru-RU'));
    lines.push('');

    try {
        if (invoke) {
            const v    = await invoke('get_version').catch(() => '?');
            const key  = await invoke('get_public_key').catch(() => '?');
            const path = await invoke('get_db_path').catch(() => '?');
            const running = await invoke('get_p2p_status').catch(() => false);
            const peers   = await invoke('get_peers').catch(() => []);
            const bootstrap = await invoke('get_bootstrap_list').catch(() => []);

            lines.push(`Версия: ${v}`);
            lines.push(`P2P: ${running ? 'Активен' : 'Остановлен'}`);
            lines.push(`Пиров: ${peers.length}`);
            lines.push(`Bootstrap узлов: ${bootstrap.length}`);
            lines.push(`Публичный ключ: ${key}`);
            lines.push(`БД: ${path}`);
            lines.push('');
            lines.push('--- Пиры ---');
            peers.forEach(p => {
                lines.push(`  ${p.protocol} | ${p.peer_id}`);
            });
            lines.push('');
            lines.push('--- Bootstrap ---');
            bootstrap.forEach(addr => lines.push(`  ${addr}`));
        }
    } catch {}

    lines.push('');
    lines.push('--- Лог событий ---');
    diagLogEntries.slice(-50).forEach(e => {
        lines.push(`[${e.time}] [${e.type.toUpperCase()}] ${e.msg}`);
    });

    const report = lines.join('\n');
    await navigator.clipboard.writeText(report);
    window.log('Отчёт скопирован в буфер обмена', 'success');
}
