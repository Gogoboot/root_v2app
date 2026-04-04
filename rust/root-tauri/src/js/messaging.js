// ============================================================
// ROOT v2.0 — messaging.js
// Отправка и получение сообщений
// ============================================================

// ── Отправка сообщения ────────────────────────────────────────────────────────

/**
 * Отправляет сообщение указанному получателю.
 *
 * Сообщение сохраняется в локальной БД и отправляется
 * через P2P сеть если узел запущен.
 *
 * @requires P2P узел может быть не запущен — сообщение
 * сохранится локально но не доставится до запуска P2P.
 */
async function sendMessage() {
    // Читаем публичный ключ получателя и текст сообщения
    const toKey   = document.getElementById('to-key').value.trim();
    const content = document.getElementById('msg-content').value.trim();

    // Проверяем что поля заполнены
    if (!toKey) {
        log('Укажи публичный ключ получателя', 'error');
        return;
    }
    if (!content) {
        log('Напиши текст сообщения', 'error');
        return;
    }

    try {
        // Вызываем Rust команду send_message
        // Rust сохранит сообщение в БД и отправит через P2P если активен
        // Возвращает внутренний id сообщения в SQLite
        const id = await invoke('send_message', { toKey, content });

        log(`Сообщение #${id} отправлено`, 'success');

        // Очищаем поле текста после отправки
        // Поле получателя оставляем — удобно отправлять несколько сообщений
        document.getElementById('msg-content').value = '';

        // Обновляем список сообщений чтобы увидеть отправленное
        loadMessages();

    } catch (e) {
        // Возможные причины: БД не открыта, identity не инициализирована,
        // неверный формат публичного ключа
        log(`Ошибка отправки: ${e}`, 'error');
    }
}

// ── Получение сообщений ───────────────────────────────────────────────────────

/**
 * Загружает входящие P2P сообщения из очереди в памяти Rust.
 *
 * Rust хранит входящие сообщения в Vec внутри APP_STATE.
 * Каждый вызов get_incoming_messages() забирает все накопленные
 * сообщения и очищает очередь (drain).
 *
 * Вызывается:
 * - Вручную кнопкой "Обновить"
 * - Автоматически каждые 5 секунд через startAutoRefresh()
 */
async function loadMessages() {
    try {
        // Вызываем Rust команду get_incoming_messages
        // Возвращает Vec<MessageInfo> и очищает очередь
        const msgs = await invoke('get_incoming_messages');

        const list = document.getElementById('msg-list');

        // Если сообщений нет — показываем заглушку
        if (msgs.length === 0) {
            list.innerHTML = '<p style="color:#555;font-size:13px;">Нет новых сообщений</p>';
            return;
        }

        // Строим HTML для каждого сообщения
        list.innerHTML = msgs.map(m => `
            <div class="msg-item">
                <!-- Сокращённый публичный ключ отправителя -->
                <div class="from">
                    От: ${m.from_key.slice(0, 24)}...
                </div>

                <!-- Текст сообщения -->
                <div class="content">${escapeHtml(m.content)}</div>

                <!-- Время в локальном формате -->
                <div class="time">
                    ${formatTime(m.timestamp)}
                </div>
            </div>
        `).join('');

    } catch (e) {
        // Ошибку не показываем в лог — loadMessages вызывается часто
        // автоматически и может мусорить при незапущенном P2P
        console.error('loadMessages:', e);
    }
}

// ── Вспомогательные функции ───────────────────────────────────────────────────

/**
 * Форматирует Unix timestamp в читаемое время.
 *
 * @param {number} timestamp - Unix время в секундах
 * @returns {string} - строка вида "15:30:45" или "вчера 15:30"
 */
function formatTime(timestamp) {
    // timestamp от Rust приходит в секундах — умножаем на 1000 для JS
    const date = new Date(timestamp * 1000);
    return date.toLocaleString('ru-RU', {
        hour:   '2-digit',
        minute: '2-digit',
        second: '2-digit',
        day:    '2-digit',
        month:  '2-digit',
    });
}

/**
 * Экранирует HTML спецсимволы в тексте сообщения.
 *
 * Защищает от XSS атак — если злоумышленник отправит
 * сообщение содержащее HTML теги, они не выполнятся.
 *
 * Пример:
 *   "<script>alert(1)</script>" → "&lt;script&gt;alert(1)&lt;/script&gt;"
 *
 * @param {string} text - исходный текст
 * @returns {string} - безопасный текст для вставки в HTML
 */
function escapeHtml(text) {
    const div = document.createElement('div');
    div.textContent = text; // textContent автоматически экранирует HTML
    return div.innerHTML;
}
