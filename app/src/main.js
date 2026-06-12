const { listen } = window.__TAURI__.event;

const statusEl = document.getElementById('status');
const resultEl = document.getElementById('result');

function set(text, cls) {
  statusEl.textContent = text;
  statusEl.className = 'status-line' + (cls ? ' ' + cls : '');
}

// Hebt die Zeile des gerade laufenden Modus hervor (gelb + fett).
function highlight(hotkey) {
  document.querySelectorAll('.mode-row').forEach((row) => {
    row.classList.toggle('active', row.dataset.hk === hotkey);
  });
}

function clearHighlight() {
  document.querySelectorAll('.mode-row.active').forEach((row) => row.classList.remove('active'));
}

listen('recording-started', (e) => {
  const p = e.payload || {};
  set('● Aufnahme: ' + (p.label || ''), 'recording');
  resultEl.textContent = '';
  highlight(p.hotkey);
});

listen('recording-stopped', () => set('Verarbeite ...'));
listen('status-update', (e) => set(String(e.payload)));

listen('transcription-result', (e) => {
  set('Fertig ✓', 'done');
  resultEl.textContent = String(e.payload);
  clearHighlight();
});

listen('show-error', (e) => {
  set('⚠ ' + String(e.payload), 'error');
  clearHighlight();
});
