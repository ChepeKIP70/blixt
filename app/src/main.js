const { listen } = window.__TAURI__.event;

const statusEl = document.getElementById('status');
const resultEl = document.getElementById('result');

function set(text, cls) {
  statusEl.textContent = text;
  statusEl.className = 'status-line' + (cls ? ' ' + cls : '');
}

listen('recording-started', (e) => {
  set('● Aufnahme: ' + e.payload, 'recording');
  resultEl.textContent = '';
});

listen('recording-stopped', () => set('Verarbeite ...'));
listen('status-update', (e) => set(String(e.payload)));

listen('transcription-result', (e) => {
  set('Fertig ✓', 'done');
  resultEl.textContent = String(e.payload);
});

listen('show-error', (e) => set('⚠ ' + String(e.payload), 'error'));
