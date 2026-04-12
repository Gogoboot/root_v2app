// ==========================================
// СОЗДАНИЕ АККАУНТА
// ==========================================
// window.generateIdentity = async function() {
//     const invoke = window.__TAURI__.core.invoke;
//     const password = document.getElementById('db-password').value;

//     if (!password) {
//         window.log('Введите пароль для шифрования', 'error');
//         return;
//     }

//     try {
//         // 1. Получаем путь к БД
//         const dbPath = await invoke('get_db_path');

//         // 2. Открываем или создаём БД
//         await invoke('unlock_database', { password, dbPath });

//         // 3. Генерируем ключи
//         // Бэкенд автоматически ставит mnemonic_confirmed = false
//         const info = await invoke('generate_identity');

//         // 4. Показываем мнемонику — только один раз
//         if (info.mnemonic) {
//             const words = info.mnemonic.split(' ');
//             const grid  = document.getElementById('mnemonic-grid');

//             grid.innerHTML = words.map((word, i) => `
//                 <div class="mnemonic-word">
//                     <span class="num">${i + 1}.</span>
//                     <span>${word}</span>
//                 </div>
//             `).join('');

//             document.getElementById('mnemonic-display').style.display = 'block';
//             window.log('Аккаунт создан. Запишите слова!', 'success');
//         }
//     } catch (e) {
//         window.log('Ошибка создания: ' + e, 'error');
//     }
// }

// Вызывается когда пользователь нажал "Я сохранил"
window.closeMnemonicAndEnter = async function() {
    const invoke = window.__TAURI__.core.invoke;

    try {
        // Подтверждаем что мнемоника записана.
        // Теперь при следующем входе экран мнемоники не появится.
        await invoke('confirm_mnemonic');
        window.log('Мнемоника подтверждена', 'success');
    } catch (e) {
        // Не блокируем вход если что-то пошло не так —
        // при следующем запуске мнемоника будет показана снова
        window.log('Предупреждение: не удалось сохранить подтверждение: ' + e, 'info');
    }

    document.getElementById('mnemonic-display').style.display = 'none';
    window.enterApp();

    // Загружаем ключ — элементы уже существуют в DOM
    try {
        const key = await invoke('get_public_key');
        document.getElementById('my-pubkey').textContent = key;
        document.getElementById('settings-pubkey').textContent = key;
    } catch (e) {
        console.log('Ключ будет загружен после входа');
    }
}

// Показать форму восстановления из мнемоники
window.showRestoreForm = function() {
    document.getElementById('restore-form').style.display = 'block';
    document.getElementById('restore-mnemonic').focus();
}

// Скрыть форму восстановления
window.hideRestoreForm = function() {
    document.getElementById('restore-form').style.display = 'none';
    document.getElementById('restore-mnemonic').value = '';
}

// Восстановить аккаунт из мнемоники
window.restoreFromMnemonic = async function() {
    const invoke = window.__TAURI__.core.invoke;
    const password = document.getElementById('db-password').value;
    const mnemonic = document.getElementById('restore-mnemonic').value.trim();

    if (!password) {
        window.log('Введите пароль для новой базы данных', 'error');
        return;
    }

    if (!mnemonic) {
        window.log('Введите мнемонику', 'error');
        return;
    }

    // Проверяем что 24 слова
    if (mnemonic.split(' ').length !== 24) {
        window.log('Мнемоника должна содержать 24 слова', 'error');
        return;
    }

    try {
        // 1. Открываем БД с новым паролем
        const dbPath = await invoke('get_db_path');
        await invoke('unlock_database', { password, dbPath });

        // 2. Восстанавливаем identity из мнемоники
        const info = await invoke('restore_identity', { mnemonic });

        window.log('✅ Аккаунт восстановлен: ' + info.public_key.slice(0, 16) + '...', 'success');

        // 3. Входим в приложение
        window.enterApp();

        // 4. Загружаем ключ в UI
        try {
            const key = await invoke('get_public_key');
            document.getElementById('my-pubkey').textContent = key;
            document.getElementById('settings-pubkey').textContent = key;
        } catch (e) {
            console.log('Ключ будет загружен после входа');
        }

    } catch (e) {
        window.log('❌ Ошибка восстановления: ' + e, 'error');
    }
}