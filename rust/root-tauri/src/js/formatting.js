// ============================================================
// formatting.js — форматирование текста в поле ввода
// ============================================================

window.formatText = function(type) {
    const textarea = document.getElementById('msg-content');
    if (!textarea) return;

    const start = textarea.selectionStart;
    const end   = textarea.selectionEnd;
    const selected = textarea.value.substring(start, end);

    const markers = {
        bold:   '**',
        italic: '_',
        strike: '~~',
        code:   '`',
    };

    const marker = markers[type];
    if (!marker) return;

    const before = textarea.value.substring(0, start);
    const after  = textarea.value.substring(end);

    if (selected) {
        textarea.value = before + marker + selected + marker + after;
        textarea.selectionStart = start + marker.length;
        textarea.selectionEnd   = end   + marker.length;
    } else {
        textarea.value = before + marker + marker + after;
        textarea.selectionStart = start + marker.length;
        textarea.selectionEnd   = start + marker.length;
    }

    textarea.focus();
}

window.attachFile = function() {
    window.log('Прикрепить файл — в разработке', 'info');
}
let markdownEnabled = true;

window.toggleMarkdown = function() {
    markdownEnabled = !markdownEnabled;
    const btn = document.getElementById('btn-markdown-toggle');
    if (btn) btn.classList.toggle('active', markdownEnabled);
    window.log('Markdown ' + (markdownEnabled ? 'включён' : 'выключён'), 'info');
}

window.isMarkdownEnabled = function() {
    return markdownEnabled;
}
window.toggleEmojiPicker = function() {
    const container = document.getElementById('emoji-picker-container');
    if (!container) return;
    const isVisible = container.style.display !== 'none';
    container.style.display = isVisible ? 'none' : 'block';

    if (!isVisible) {
        const picker = container.querySelector('emoji-picker');
        if (picker) {
            picker.addEventListener('emoji-click', function(e) {
                const textarea = document.getElementById('msg-content');
                if (!textarea) return;
                const pos = textarea.selectionStart;
                const before = textarea.value.substring(0, pos);
                const after  = textarea.value.substring(pos);
                textarea.value = before + e.detail.unicode + after;
                textarea.selectionStart = pos + e.detail.unicode.length;
                textarea.selectionEnd   = pos + e.detail.unicode.length;
                textarea.focus();
                container.style.display = 'none';
            }, { once: false });
        }

        setTimeout(() => {
            document.addEventListener('click', function handler(e) {
                if (!container.contains(e.target) && e.target.id !== 'btn-emoji') {
                    container.style.display = 'none';
                    document.removeEventListener('click', handler);
                }
            });
        }, 0);
    }
}

