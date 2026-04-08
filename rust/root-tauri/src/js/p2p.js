// ============================================================
// ROOT v2.0 — js/p2p.js
// P2P управление: запуск/стоп, список пиров, bootstrap
// ============================================================

let isP2pBusy = false;

// ── Запуск / остановка ───────────────────────────────────────

window.startP2P = async function() {
    const invoke = window.__TAURI__.core.invoke;
    if (isP2pBusy) return;
    isP2pBusy = true;
    try {
        const result = await invoke('start_p2p_node');
        window.log('P2P запускается...', 'info');
        // Статус обновится через refreshP2PStatus через секунду
        setTimeout(window.refreshP2PStatus, 1200);
    } catch (e) {
        window.log('Ошибка P2P: ' + e, 'error');
    } finally {
        isP2pBusy = false;
    }
}

window.stopP2P = async function() {
    const invoke = window.__TAURI__.core.invoke;
    if (isP2pBusy) return;
    isP2pBusy = true;
    try {
        await invoke('stop_p2p_node');
        window.log('P2P остановлен', 'info');
        window.refreshP2PStatus();
    } catch (e) {
        window.log('Ошибка: ' + e, 'error');
    } finally {
        isP2pBusy = false;
    }
}

// ── Статус и список пиров ────────────────────────────────────

window.refreshP2PStatus = async function() {
    const invoke = window.__TAURI__.core.invoke;
    try {
        const running = await invoke('get_p2p_status');
        const peers   = await invoke('get_peers'); // Vec<PeerInfoDto>

        // Статус бейдж
        const badge = document.getElementById('p2p-badge');
        const dot   = document.getElementById('p2p-dot');
        const text  = document.getElementById('p2p-status-text');

        if (running) {
            badge.className = 'status-badge active';
            dot.className   = 'dot green';
            text.textContent = 'Активен';
        } else {
            badge.className = 'status-badge inactive';
            dot.className   = 'dot red';
            text.textContent = 'Остановлен';
        }

        // Счётчик со склонением
        const count = peers.length;
        let word = 'пиров';
        if (count % 10 === 1 && count % 100 !== 11) word = 'пир';
        else if ([2,3,4].includes(count % 10) && ![12,13,14].includes(count % 100)) word = 'пира';
        document.getElementById('peer-count-text').textContent = `${count} ${word}`;

        // Рендер списка пиров
        renderPeerList(peers);

    } catch (e) {
        // тихо — статус просто не обновится
    }
}

// Рендер плоского списка пиров с группировкой по протоколу
function renderPeerList(peers) {
    const container = document.getElementById('peer-list');
    if (!container) return;

    if (peers.length === 0) {
        container.innerHTML = '<p class="empty-state">Нет активных соединений</p>';
        return;
    }

    // Сортируем по протоколу, потом по времени подключения
    const sorted = [...peers].sort((a, b) => {
        if (a.protocol !== b.protocol) return a.protocol.localeCompare(b.protocol);
        return a.connected_at - b.connected_at;
    });

    // Первые 5 всегда видны, остальные скрыты
    const VISIBLE_LIMIT = 5;
    const visible = sorted.slice(0, VISIBLE_LIMIT);
    const hidden  = sorted.slice(VISIBLE_LIMIT);

    let html = '';
    let lastProtocol = null;

    visible.forEach(peer => {
        // Разделитель протокола
        if (peer.protocol !== lastProtocol) {
            lastProtocol = peer.protocol;
        }
        html += renderPeerItem(peer);
    });

    // Кнопка "показать ещё"
    if (hidden.length > 0) {
        html += `<button class="peer-show-more" onclick="toggleHiddenPeers(this)">
            ▼ Показать ещё ${hidden.length} пиров
        </button>`;
        // Скрытые пиры
        html += `<div class="hidden-peers" style="display:none;">`;
        hidden.forEach(peer => { html += renderPeerItem(peer); });
        html += `</div>`;
    }

    container.innerHTML = html;
}

function renderPeerItem(peer) {
    const proto = peer.protocol.toLowerCase();
    const shortId = peer.peer_id.length > 20
        ? peer.peer_id.slice(0, 8) + '...' + peer.peer_id.slice(-6)
        : peer.peer_id;
    const time = formatPeerTime(peer.connected_at);

    return `<div class="peer-item">
        <span class="peer-protocol ${proto}">${peer.protocol}</span>
        <span class="peer-id" title="${peer.peer_id}">${shortId}</span>
        <span class="peer-time">${time}</span>
    </div>`;
}

function formatPeerTime(unixTs) {
    const now  = Math.floor(Date.now() / 1000);
    const diff = now - unixTs;
    if (diff < 60)  return `${diff}с`;
    if (diff < 3600) return `${Math.floor(diff / 60)}м`;
    return `${Math.floor(diff / 3600)}ч`;
}

window.toggleHiddenPeers = function(btn) {
    const hidden = btn.nextElementSibling;
    if (hidden.style.display === 'none') {
        hidden.style.display = 'flex';
        hidden.style.flexDirection = 'column';
        hidden.style.gap = '6px';
        btn.textContent = '▲ Скрыть';
    } else {
        hidden.style.display = 'none';
        const count = hidden.children.length;
        btn.textContent = `▼ Показать ещё ${count} пиров`;
    }
}

// ── Bootstrap управление ─────────────────────────────────────

// Загружаем bootstrap список при открытии настроек
window.loadBootstrapList = async function() {
    const invoke = window.__TAURI__.core.invoke;
    try {
        const list = await invoke('get_bootstrap_list');
        renderBootstrapList(list);
    } catch (e) {
        console.log('bootstrap load error:', e);
    }
}

function renderBootstrapList(list) {
    const container = document.getElementById('bootstrap-list');
    if (!container) return;

    if (!list || list.length === 0) {
        container.innerHTML = '<p class="empty-state">Нет bootstrap узлов</p>';
        return;
    }

    container.innerHTML = list.map((addr, i) => `
        <div class="bootstrap-item">
            <span class="bootstrap-addr" title="${addr}">${addr}</span>
            <button class="bootstrap-remove" onclick="removeBootstrap(${i})" title="Удалить">✕</button>
        </div>
    `).join('');

    // Сохраняем список в памяти для операций
    window._bootstrapList = list;
}

window.addBootstrap = async function() {
    const invoke = window.__TAURI__.core.invoke;
    const input  = document.getElementById('bootstrap-input');
    const addr   = input.value.trim();

    if (!addr) return;

    // Базовая проверка формата
    if (!addr.startsWith('/')) {
        window.log('Неверный Multiaddr — должен начинаться с /', 'error');
        return;
    }

    try {
        const current = await invoke('get_bootstrap_list');
        if (current.includes(addr)) {
            window.log('Этот адрес уже добавлен', 'info');
            return;
        }
        const updated = [...current, addr];
        await invoke('save_bootstrap_list', { addrs: updated });
        input.value = '';
        renderBootstrapList(updated);
        window.log('Bootstrap узел добавлен', 'success');
    } catch (e) {
        window.log('Ошибка: ' + e, 'error');
    }
}

window.removeBootstrap = async function(index) {
    const invoke = window.__TAURI__.core.invoke;
    try {
        const current = await invoke('get_bootstrap_list');
        const updated = current.filter((_, i) => i !== index);
        await invoke('save_bootstrap_list', { addrs: updated });
        renderBootstrapList(updated);
        window.log('Bootstrap узел удалён', 'info');
    } catch (e) {
        window.log('Ошибка: ' + e, 'error');
    }
}

// Ручной диал всех bootstrap адресов
window.dialBootstrap = async function() {
    const invoke = window.__TAURI__.core.invoke;
    try {
        const running = await invoke('get_p2p_status');
        if (!running) {
            window.log('Сначала запустите P2P узел', 'error');
            return;
        }
        const list = await invoke('get_bootstrap_list');
        if (list.length === 0) {
            window.log('Нет bootstrap адресов', 'info');
            return;
        }
        for (const addr of list) {
            try {
                await invoke('dial_node', { addr });
                window.log('Диал: ' + addr.slice(0, 40) + '...', 'info');
            } catch (e) {
                window.log('Ошибка диала: ' + e, 'error');
            }
        }
    } catch (e) {
        window.log('Ошибка: ' + e, 'error');
    }
}

// ── Путь к БД ────────────────────────────────────────────────

window.loadDbPath = async function() {
    const invoke = window.__TAURI__.core.invoke;
    try {
        const path = await invoke('get_db_path');
        const el = document.getElementById('db-path-display');
        if (el) el.textContent = path;
    } catch (e) {}
}
