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