const { invoke } = window.__TAURI__.core;

const el = (id) => document.getElementById(id);

// Cloud-Anbieter fuer den Text-Schritt ist fest Groq (das Routing nutzt nur Groq).
const CHAT_CLOUD_PROVIDER = 'groq';

const keyDisplayArea    = el('keyDisplayArea');
const keyDisplayValue   = el('keyDisplayValue');
const keyInputArea      = el('keyInputArea');
const apiKeyInput       = el('apiKeyInput');
const saveKeyBtn        = el('saveKeyBtn');
const pasteKeyBtn       = el('pasteKeyBtn');
const changeKeyBtn      = el('changeKeyBtn');
const deleteKeyBtn      = el('deleteKeyBtn');
const testBtn           = el('testBtn');
const keyMsg            = el('keyMsg');

const chatModel          = el('chatModel');
const localChatModel     = el('localChatModel');
const localSttUrl        = el('localSttUrl');
const language           = el('language');
const autoPasteEnabled   = el('autoPasteEnabled');
const saveSettingsBtn    = el('saveSettingsBtn');
const settingsMsg        = el('settingsMsg');

// Felder, die die UI nicht mehr anzeigt, aber die Settings-Struktur weiter erwartet.
// Beim Speichern unveraendert zuruecksenden (Transkription ist ohnehin immer lokal).
let keptTranscriptionModel = 'whisper-large-v3-turbo';

function flash(node, text, type) {
  node.textContent = text;
  node.className = `msg ${type}`;
  node.style.display = 'block';
  setTimeout(() => { node.style.display = 'none'; }, 4000);
}

async function refreshKeyArea() {
  const hasKey = await invoke('has_api_key', { provider: CHAT_CLOUD_PROVIDER });
  if (hasKey) {
    keyDisplayValue.textContent = await invoke('api_key_display', { provider: CHAT_CLOUD_PROVIDER });
    keyDisplayArea.style.display = 'block';
    keyInputArea.style.display = 'none';
  } else {
    keyDisplayArea.style.display = 'none';
    keyInputArea.style.display = 'block';
    apiKeyInput.value = '';
  }
}

async function loadSettings() {
  const s = await invoke('get_settings');
  keptTranscriptionModel   = s.transcriptionModel || 'whisper-large-v3-turbo';
  chatModel.value          = s.chatModel || '';
  localChatModel.value     = s.localChatModel || 'qwen2.5:7b';
  localSttUrl.value        = s.localSttUrl || 'http://127.0.0.1:8765/v1';
  language.value           = s.language ?? 'de';
  autoPasteEnabled.checked = s.autoPasteEnabled !== false;
  await refreshKeyArea();
}

(async () => { await loadSettings(); })();

saveKeyBtn.addEventListener('click', async () => {
  const key = apiKeyInput.value.trim();
  if (!key) { flash(keyMsg, 'Bitte einen Schlüssel eingeben.', 'error'); return; }
  try {
    await invoke('save_api_key', { provider: CHAT_CLOUD_PROVIDER, key });
    apiKeyInput.value = '';
    await refreshKeyArea();
    flash(keyMsg, 'Schlüssel gespeichert.', 'success');
  } catch (e) { flash(keyMsg, String(e), 'error'); }
});

pasteKeyBtn.addEventListener('click', async () => {
  try {
    const text = await navigator.clipboard.readText();
    apiKeyInput.value = (text || '').split('\n')[0].trim();
    flash(keyMsg, 'Eingefügt. Bitte speichern.', 'success');
  } catch { flash(keyMsg, 'Zwischenablage konnte nicht gelesen werden.', 'error'); }
});

changeKeyBtn.addEventListener('click', () => {
  keyDisplayArea.style.display = 'none';
  keyInputArea.style.display = 'block';
  apiKeyInput.focus();
});

deleteKeyBtn.addEventListener('click', async () => {
  if (!confirm('Schlüssel wirklich löschen?')) return;
  try {
    await invoke('delete_api_key', { provider: CHAT_CLOUD_PROVIDER });
    await refreshKeyArea();
    flash(keyMsg, 'Schlüssel gelöscht.', 'success');
  } catch (e) { flash(keyMsg, String(e), 'error'); }
});

testBtn.addEventListener('click', async () => {
  testBtn.disabled = true;
  testBtn.textContent = 'Teste ...';
  try {
    await invoke('test_connection');
    flash(keyMsg, 'Verbindung erfolgreich.', 'success');
  } catch (e) { flash(keyMsg, String(e), 'error'); }
  finally { testBtn.disabled = false; testBtn.textContent = 'Verbindung testen'; }
});

saveSettingsBtn.addEventListener('click', async () => {
  try {
    await invoke('save_settings', {
      s: {
        // Transkription ist immer lokal -> provider fest "local".
        provider: 'local',
        transcriptionModel: keptTranscriptionModel,
        chatModel: chatModel.value.trim() || 'llama-3.3-70b-versatile',
        // Routing entscheidet Groq vs. lokal; chatProvider bleibt aus Kompatibilitaet "groq".
        chatProvider: 'groq',
        localChatModel: localChatModel.value.trim() || 'qwen2.5:7b',
        localSttUrl: localSttUrl.value.trim() || 'http://127.0.0.1:8765/v1',
        language: language.value,
        hotkeyMode: 'toggle',
        autoPasteEnabled: autoPasteEnabled.checked,
      },
    });
    flash(settingsMsg, 'Einstellungen gespeichert.', 'success');
  } catch (e) { flash(settingsMsg, String(e), 'error'); }
});
