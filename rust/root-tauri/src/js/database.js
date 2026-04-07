// ==========================================
// ОТКРЫТИЕ БАЗЫ
// ==========================================
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

        const result = await invoke('unlock_database', { password, dbPath });

        // ── Сценарий 1: Мнемоника не подтверждена (повторный показ) ──────
        if (result.status === 'mnemonic_pending') {
            window.log('Требуется подтверждение мнемоники', 'info');
            showMnemonicAgain(result.mnemonic);
            return;
        }

        // ── Сценарий 2: НОВАЯ база (public_key отсутствует) ─────────────
        if (result.status === 'ok' && (!result.public_key || result.public_key === '')) {
            window.log('🆕 Новая база — генерация идентити...', 'info');
            
            try {
                // Вызываем генерацию ключей
                const identity = await invoke('generate_identity', { dbPath });
                
                // Показываем мнемонику пользователю
                showMnemonicAgain(identity.mnemonic);
                
            } catch (genErr) {
                // Если аккаунт уже существует (защита от гонки)
                if (genErr.toString().includes('Аккаунт уже существует')) {
                    window.log('⚠ Аккаунт уже создан, входим...', 'info');
                    window.enterApp();
                } else {
                    window.log('❌ Ошибка генерации: ' + genErr, 'error');
                }
            }
            return; // Важно: не идём дальше, ждём подтверждения мнемоники
        }

        // ── Сценарий 3: Обычный вход (существующий аккаунт) ─────────────
        window.log('✅ База данных открыта', 'success');
        window.enterApp();

    } catch (e) {
        window.log('Ошибка входа: ' + e, 'error');
    }
}

// Показывает мнемонику повторно когда она не была подтверждена
function showMnemonicAgain(mnemonic) {
    if (!mnemonic) {
        window.log('Ошибка: мнемоника недоступна', 'error');
        return;
    }

    const words = mnemonic.split(' ');
    const grid  = document.getElementById('mnemonic-grid');

    grid.innerHTML = words.map((word, i) => `
        <div class="mnemonic-word">
            <span class="num">${i + 1}.</span>
            <span>${word}</span>
        </div>
    `).join('');

    document.getElementById('mnemonic-display').style.display = 'block';
    window.log('Запишите слова — они не были подтверждены', 'info');
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
