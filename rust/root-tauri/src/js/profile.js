// ============================================================
// ROOT v2.0 — js/profile.js
// Вкладка профиля: публичный ключ, аватар, QR-код
// ============================================================

// ── QR-код через qrcode.js ───────────────────────────────────
// Библиотека подключается через CDN в index.html:
// <script src="https://cdnjs.cloudflare.com/ajax/libs/qrcodejs/1.0.0/qrcode.min.js"></script>

let _profileQRInstance = null;

function renderProfileQR(text) {
  const wrap = document.getElementById('profile-qr');
  if (!wrap) return;

  wrap.innerHTML = '';
  _profileQRInstance = null;

  if (!text || text === '—' || typeof QRCode === 'undefined') {
    wrap.innerHTML = '<p class="empty-state" style="padding:20px 0;">Ключ не загружен</p>';
    return;
  }

  try {
    _profileQRInstance = new QRCode(wrap, {
      text: text,
      width: 200,
      height: 200,
      colorDark: getComputedStyle(document.documentElement)
        .getPropertyValue('--text-primary').trim() || '#e8eaf0',
      colorLight: getComputedStyle(document.documentElement)
        .getPropertyValue('--bg-surface').trim() || '#16192e',
      correctLevel: QRCode.CorrectLevel.M,
    });
  } catch (e) {
    wrap.innerHTML = '<p class="empty-state" style="color:var(--red);">Ошибка QR</p>';
    console.error('QR render error:', e);
  }
}

// ── Загрузить профиль ────────────────────────────────────────

window.loadProfile = async function () {
  try {
    const invoke = window.__TAURI__.core.invoke;
    const key = await invoke('get_public_key');

    // Полный ключ
    const fullEl = document.getElementById('profile-pubkey-full');
    if (fullEl) fullEl.textContent = key || '—';

    // Короткий ключ (первые 8 + ... + последние 8)
    const shortEl = document.getElementById('profile-pubkey-short');
    if (shortEl && key && key.length > 20) {
      shortEl.textContent = key.slice(0, 8) + '…' + key.slice(-8);
    }

    // Аватар
    const avatarEl = document.getElementById('profile-avatar');
    if (avatarEl && key) {
      avatarEl.style.background = window.getAvatarColor(key);
      avatarEl.textContent = window.getAvatarText(key);
    }

    // Ник — берём из localStorage если есть, иначе пусто
    const nickEl = document.getElementById('profile-nick');
    if (nickEl) {
      const savedNick = localStorage.getItem('root-my-nick') || '';
      nickEl.textContent = savedNick || 'Без имени';
    }

    // QR
    renderProfileQR(key);

  } catch (e) {
    console.error('loadProfile error:', e);
    const fullEl = document.getElementById('profile-pubkey-full');
    if (fullEl) fullEl.textContent = 'Ошибка загрузки';
  }
};

// ── Обновить QR (пересоздать с текущей темой) ────────────────

window.refreshProfileQR = function () {
  const key = document.getElementById('profile-pubkey-full')?.textContent;
  if (key && key !== '—') renderProfileQR(key);
};

// ── Копировать ключ ──────────────────────────────────────────

window.copyProfileKey = function () {
  const key = document.getElementById('profile-pubkey-full')?.textContent;
  if (key && key !== '—') {
    navigator.clipboard.writeText(key);
    window.log && window.log('Ключ скопирован', 'success');

    // Визуальная обратная связь
    const btn = document.querySelector('.profile-copy-btn');
    if (btn) {
      const orig = btn.textContent;
      btn.textContent = '✓ Скопировано';
      btn.style.color = 'var(--green)';
      setTimeout(() => {
        btn.textContent = orig;
        btn.style.color = '';
      }, 1500);
    }
  }
};

// ── Сохранить ник профиля ────────────────────────────────────

window.saveProfileNick = function () {
  const input = document.getElementById('profile-nick-input');
  if (!input) return;
  const nick = input.value.trim();
  localStorage.setItem('root-my-nick', nick);

  const nickEl = document.getElementById('profile-nick');
  if (nickEl) nickEl.textContent = nick || 'Без имени';

  input.blur();
  window.log && window.log('Имя профиля сохранено', 'success');
};
