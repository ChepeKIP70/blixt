// Anbieter-Schicht (provider layer): kapselt Transkription + Textmodell hinter EINER Schnittstelle.
// Groq und OpenAI sprechen dieselbe OpenAI-kompatible API (gleiche /audio/transcriptions- und
// /chat/completions-Form) -> nur Basis-URL + Modellnamen unterscheiden sich. "Local" = Ollama
// (Textmodell, 127.0.0.1:11434) bzw. der whisper.cpp-Server (Transkription, via transcribe_local()
// mit der Server-URL aus den Einstellungen) -- beide offline, kein Schluessel.
//
// Bewusst als Enum statt `dyn Trait`: vermeidet async-trait/Dyn-Probleme, reicht fuer 3 Anbieter.

use std::path::Path;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Provider {
    Groq,
    OpenAi,
    Local,
}

impl Provider {
    pub fn from_id(s: &str) -> Provider {
        match s {
            "openai" => Provider::OpenAi,
            "local" => Provider::Local,
            _ => Provider::Groq,
        }
    }

    fn base_url(&self) -> &'static str {
        match self {
            Provider::Groq => "https://api.groq.com/openai/v1",
            Provider::OpenAi => "https://api.openai.com/v1",
            // Ollama, OpenAI-kompatibel. Feste IPv4-Loopback (kein "localhost"!) -> funktioniert
            // auch offline; "localhost" loest sonst zu ::1 auf und scheitert ohne Netz/mit VPN.
            Provider::Local => "http://127.0.0.1:11434/v1",
        }
    }

    pub async fn transcribe(
        &self,
        audio: &Path,
        api_key: &str,
        model: &str,
        language: Option<&str>,
    ) -> Result<String, String> {
        if *self == Provider::Local {
            // Lokale Transkription laeuft NICHT ueber diese Methode -- sie kennt die Whisper-Server-URL
            // aus den Einstellungen nicht. main.rs ruft dafuer direkt transcribe_local() auf.
            return Err(
                "Interner Fehler: lokale Transkription muss ueber transcribe_local() laufen."
                    .to_string(),
            );
        }
        transcribe_openai_compatible(self.base_url(), audio, api_key, model, language).await
    }

    pub async fn chat(
        &self,
        system_prompt: &str,
        user_text: &str,
        api_key: &str,
        model: &str,
        temperature: f64,
    ) -> Result<String, String> {
        // Local laeuft ueber Ollama (OpenAI-kompatibel, kein Schluessel noetig).
        chat_openai_compatible(self.base_url(), system_prompt, user_text, api_key, model, temperature).await
    }

    pub async fn test_connection(&self, api_key: &str, chat_model: &str) -> Result<(), String> {
        if *self == Provider::Local {
            return Ok(());
        }
        chat_openai_compatible(self.base_url(), "Antworte mit: OK.", "hi", api_key, chat_model, 0.0)
            .await
            .map(|_| ())
    }
}

// HTTP-Client mit Timeouts + OHNE System-Proxy.
// no_proxy(): sonst leitet reqwest selbst 127.0.0.1-Anfragen ueber einen evtl. gesetzten
// System-Proxy (z.B. via VPN), der offline tot ist -> lokale Server waeren nicht erreichbar.
fn http_client() -> Result<reqwest::Client, String> {
    reqwest::Client::builder()
        .no_proxy()
        .connect_timeout(std::time::Duration::from_secs(10))
        .timeout(std::time::Duration::from_secs(30))
        .build()
        .map_err(|e| format!("HTTP-Client-Fehler: {}", e))
}

/// Transkription über einen lokalen, OpenAI-kompatiblen STT-Server (whisper.cpp).
/// Kein Schlüssel nötig; Server-URL kommt aus den Einstellungen (z.B. http://localhost:8765/v1).
pub async fn transcribe_local(
    base_url: &str,
    audio: &std::path::Path,
    model: &str,
    language: Option<&str>,
) -> Result<String, String> {
    transcribe_openai_compatible(base_url, audio, "", model, language).await
}

fn api_error(label: &str, status: u16, body: &str) -> String {
    match status {
        401 => format!("{}: API-Schluessel ungueltig. Bitte pruefen.", label),
        413 => "Audiodatei zu gross.".to_string(),
        429 => format!("{}: Rate Limit erreicht. Kurz warten und erneut versuchen.", label),
        503 => format!("{} ist gerade nicht erreichbar. Bitte erneut versuchen.", label),
        _ => {
            if let Ok(v) = serde_json::from_str::<serde_json::Value>(body) {
                if let Some(msg) = v["error"]["message"].as_str() {
                    return format!("{}-Fehler: {}", label, msg);
                }
            }
            format!("{}-Fehler (HTTP {})", label, status)
        }
    }
}

async fn transcribe_openai_compatible(
    base_url: &str,
    audio: &Path,
    api_key: &str,
    model: &str,
    language: Option<&str>,
) -> Result<String, String> {
    let url = format!("{}/audio/transcriptions", base_url);
    let bytes = tokio::fs::read(audio)
        .await
        .map_err(|e| format!("Audiodatei konnte nicht gelesen werden: {}", e))?;

    let file_part = reqwest::multipart::Part::bytes(bytes)
        .file_name("audio.wav")
        .mime_str("audio/wav")
        .map_err(|e| e.to_string())?;

    let mut form = reqwest::multipart::Form::new()
        .part("file", file_part)
        .text("model", model.to_string())
        .text("response_format", "text");

    if let Some(lang) = language {
        let lang = lang.trim();
        if !lang.is_empty() {
            form = form.text("language", lang.to_string());
        }
    }

    let client = http_client()?;
    let response = client
        .post(&url)
        .header("Authorization", format!("Bearer {}", api_key))
        .multipart(form)
        .send()
        .await
        .map_err(|e| format!("Netzwerkfehler: {}", e))?;

    let status = response.status().as_u16();
    let body = response.text().await.unwrap_or_default();
    if status != 200 {
        return Err(api_error("Transkription", status, &body));
    }
    let text = body.trim().to_string();
    if text.is_empty() {
        return Err("Transkription fehlgeschlagen - leere Antwort.".to_string());
    }
    Ok(text)
}

async fn chat_openai_compatible(
    base_url: &str,
    system_prompt: &str,
    user_text: &str,
    api_key: &str,
    model: &str,
    temperature: f64,
) -> Result<String, String> {
    let url = format!("{}/chat/completions", base_url);
    let body = serde_json::json!({
        "model": model,
        "messages": [
            {"role": "system", "content": system_prompt},
            {"role": "user",   "content": user_text}
        ],
        "temperature": temperature
    });

    let client = http_client()?;
    let response = client
        .post(&url)
        .header("Authorization", format!("Bearer {}", api_key))
        .header("Content-Type", "application/json")
        .json(&body)
        .send()
        .await
        .map_err(|e| format!("Netzwerkfehler: {}", e))?;

    let status = response.status().as_u16();
    let body_text = response.text().await.unwrap_or_default();
    if status != 200 {
        return Err(api_error("Textmodell", status, &body_text));
    }

    let json: serde_json::Value =
        serde_json::from_str(&body_text).map_err(|e| format!("Antwort-Parsing: {}", e))?;
    let content = json["choices"][0]["message"]["content"]
        .as_str()
        .unwrap_or("")
        .trim()
        .to_string();
    if content.is_empty() {
        return Err("Keine Antwort erhalten.".to_string());
    }
    Ok(content)
}
