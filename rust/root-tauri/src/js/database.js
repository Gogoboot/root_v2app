// ==========================================
// ОТКРЫТИЕ БАЗЫ
// ==========================================
window.unlockDatabase = async function() {
    const invoke = window.__TAURI__.core.invoke;
    const password = document.getElementById('db-password').value;
    
    if (!password) {
        window.log('Введите пароль', 'error');
        return;
    }

    try {
        const dbPath = await invoke('get_db_path');
        await invoke('unlock_database', { password, dbPath });
        window.log('База данных открыта', 'success');
        window.enterApp();
    } catch (e) {
        window.log('Ошибка входа: ' + e, 'error');
    }
}

// ==========================================
// PANIC BUTTON
// ==========================================
window.panicButton = async function() {
    const invoke = window.__TAURI__.core.invoke;
    if (!confirm('Вы уверены? Все данные будут удалены!')) return;

    try {
        await invoke('panic_button');
        window.log('Данные уничтожены.', 'error');
        document.getElementById('screen-app').classList.remove('active');
        document.getElementById('screen-init').classList.add('active');
        document.getElementById('main-nav').style.display = 'none';
        document.getElementById('db-password').value = '';
    } catch (e) {
        window.log('Ошибка: ' + e, 'error');
    }
}
