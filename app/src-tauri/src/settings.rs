use serde::{Deserialize, Serialize};
use tauri::AppHandle;
use tauri_plugin_store::StoreExt;

const KEYRING_SERVICE: &str = "app.blixt";
const STORE_PATH: &str = "settings.json";

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Settings {
    pub provider: String,            // Transkriptions-Anbieter: "groq" | "openai" | "local"
    pub transcription_model: String, // z.B. whisper-large-v3-turbo (Groq) / whisper-1 (OpenAI)
    pub chat_model: String,          // Cloud-Textmodell: llama-3.3-70b-versatile (Groq) / gpt-4o-mini (OpenAI)
    pub language: String,
    pub hotkey_mode: String, // "toggle" | "hold" (hold ist Ausbaustufe)
    #[serde(default = "default_true")]
    pub auto_paste_enabled: bool,
    // Anbieter fuer das Textmodell (Transform-Modi), getrennt vom Transkriptions-Anbieter.
    // "local" = Ollama (offline, kein Schluessel).
    #[serde(default = "default_groq")]
    pub chat_provider: String,
    #[serde(default = "default_local_chat_model")]
    pub local_chat_model: String,
    // Lokaler Whisper-Server (whisper.cpp, OpenAI-kompatibel) für offline-Transkription.
    #[serde(default = "default_local_stt_url")]
    pub local_stt_url: String,
}

fn default_true() -> bool {
    true
}

fn default_groq() -> String {
    "groq".to_string()
}

fn default_local_chat_model() -> String {
    "qwen2.5:7b".to_string()
}

fn default_local_stt_url() -> String {
    "http://127.0.0.1:8765/v1".to_string()
}

impl Default for Settings {
    fn default() -> Self {
        Settings {
            provider: "groq".to_string(),
            transcription_model: "whisper-large-v3-turbo".to_string(),
            chat_model: "llama-3.3-70b-versatile".to_string(),
            language: "de".to_string(),
            hotkey_mode: "toggle".to_string(),
            auto_paste_enabled: true,
            chat_provider: "groq".to_string(),
            local_chat_model: "qwen2.5:7b".to_string(),
            local_stt_url: "http://127.0.0.1:8765/v1".to_string(),
        }
    }
}

pub fn load(app: &AppHandle) -> Settings {
    app.store(STORE_PATH)
        .ok()
        .and_then(|store| store.get("settings"))
        .and_then(|v| serde_json::from_value(v).ok())
        .unwrap_or_default()
}

pub fn save(app: &AppHandle, settings: &Settings) -> Result<(), String> {
    let store = app.store(STORE_PATH).map_err(|e| e.to_string())?;
    store.set(
        "settings",
        serde_json::to_value(settings).map_err(|e| e.to_string())?,
    );
    store.save().map_err(|e| e.to_string())
}

// ── API-Schluessel je Anbieter (Windows Credential Manager) ──────────────────
// Konto = Anbieter-ID, damit Groq- und OpenAI-Schluessel getrennt liegen.

pub fn get_api_key(provider: &str) -> Result<String, String> {
    keyring::Entry::new(KEYRING_SERVICE, provider)
        .map_err(|e| e.to_string())?
        .get_password()
        .map_err(|_| "Kein API-Schluessel gespeichert.".to_string())
}

pub fn save_api_key(provider: &str, key: &str) -> Result<(), String> {
    keyring::Entry::new(KEYRING_SERVICE, provider)
        .map_err(|e| e.to_string())?
        .set_password(key)
        .map_err(|e| e.to_string())
}

pub fn delete_api_key(provider: &str) -> Result<(), String> {
    keyring::Entry::new(KEYRING_SERVICE, provider)
        .map_err(|e| e.to_string())?
        .delete_credential()
        .map_err(|e| e.to_string())
}

pub fn has_api_key(provider: &str) -> bool {
    get_api_key(provider).is_ok()
}

pub fn api_key_display(provider: &str) -> String {
    match get_api_key(provider) {
        Ok(k) if k.len() > 8 => format!("{} ••••••••", &k[..4]),
        Ok(_) => "••••••••".to_string(),
        Err(_) => String::new(),
    }
}
