//import { invoke } from '@tauri-apps/api/core';


let isP2pBusy = false;

window.startP2P = async function() {
    const invoke = window.__TAURI__.core.invoke;
    if (isP2pBusy) return;
    isP2pBusy = true;
    try {
        const peerId = await invoke('start_p2p_node');
        window.log('P2P запущен: ' + peerId.slice(0, 10) + '...', 'success');
        window.refreshP2PStatus();
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

window.refreshP2PStatus = async function() {
    const invoke = window.__TAURI__.core.invoke;
    try {
        const running = await invoke('get_p2p_status');
        const count = await invoke('get_peer_count');

        const badge = document.getElementById('p2p-badge');
        const dot = document.getElementById('p2p-dot');
        const text = document.getElementById('p2p-status-text');
        const peers = document.getElementById('peer-count-text');

        if (running) {
            badge.className = 'status-badge active';
            dot.className = 'dot green';
            text.textContent = 'Активен';
        } else {
            badge.className = 'status-badge inactive';
            dot.className = 'dot red';
            text.textContent = 'Остановлен';
        }
        
        // Склонение слова "пир"
        let word = 'пиров';
        if (count === 1) word = 'пир';
        if (count >= 2 && count <= 4) word = 'пира';
        if (count >= 11 && count <= 14) word = 'пиров';

        peers.textContent = `${count} ${word}`;
    } catch (e) {}
}
