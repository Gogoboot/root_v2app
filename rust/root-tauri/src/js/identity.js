// ==========================================
// СОЗДАНИЕ АККАУНТА
// ==========================================
window.generateIdentity = async function() {
    const invoke = window.__TAURI__.core.invoke;
    const password = document.getElementById('db-password').value;

    if (!password) {
        window.log('Введите пароль для шифрования', 'error');
        return;
    }

    try {
        // 1. Получаем путь к БД
        const dbPath = await invoke('get_db_path');

        // 2. Открываем или создаём БД
        await invoke('unlock_database', { password, dbPath });

        // 3. Генерируем ключи
        const info = await invoke('generate_identity');

        // 4. Показываем мнемонику — только один раз
        if (info.mnemonic) {
            document.getElementById('mnemonic-text').textContent = info.mnemonic;
            document.getElementById('mnemonic-display').style.display = 'block';
            window.log('Аккаунт создан. Сохраните слова!', 'success');
        }
    } catch (e) {
        window.log('Ошибка создания: ' + e, 'error');
    }
}

// Вызывается когда пользователь нажал "Я сохранил"
window.closeMnemonicAndEnter = async function() {
    const invoke = window.__TAURI__.core.invoke;
    document.getElementById('mnemonic-display').style.display = 'none';
    window.enterApp();
    // Потом загружаем ключ — элементы уже существуют в DOM
    try {
        const key = await invoke('get_public_key');
        document.getElementById('my-pubkey').textContent = key;
        document.getElementById('settings-pubkey').textContent = key;
    } catch (e) {
        console.log('Ключ будет загружен после входа');
    }
}
