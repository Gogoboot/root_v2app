// ============================================================
// chat-actions.js — действия в шапке чата
// ============================================================

window.startVideoCall = function() {
    window.log('Видеозвонок — в разработке', 'info');
}

window.addCurrentToContacts = function() {
    const key = window._currentChatKey;
    if (!key) return;
    window.openNickModal();
}

window.toggleChatMenu = function() {
    const menu = document.getElementById('chat-menu-dropdown');
    if (!menu) return;
    const isVisible = menu.style.display !== 'none';
    menu.style.display = isVisible ? 'none' : 'flex';

    if (!isVisible) {
        setTimeout(() => {
            document.addEventListener('click', function handler(e) {
                if (!menu.contains(e.target)) {
                    menu.style.display = 'none';
                    document.removeEventListener('click', handler);
                }
            });
        }, 0);
    }
}

window.clearChat = function() {
    window.log('Очистить чат — в разработке', 'info');
    document.getElementById('chat-menu-dropdown').style.display = 'none';
}

window.copyContactKey = function() {
    const key = window._currentChatKey;
    if (!key) return;
    navigator.clipboard.writeText(key);
    window.log('Ключ скопирован', 'success');
    document.getElementById('chat-menu-dropdown').style.display = 'none';
}

window.deleteContact = function() {
    window.log('Удалить контакт — в разработке', 'info');
    document.getElementById('chat-menu-dropdown').style.display = 'none';
}
