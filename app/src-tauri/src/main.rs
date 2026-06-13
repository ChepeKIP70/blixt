// Verhindert ein Konsolenfenster im Release-Build unter Windows.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::sync::Mutex;
use tauri::{
    menu::{Menu, MenuItem},
    tray::{MouseButton, TrayIconBuilder, TrayIconEvent},
    AppHandle, Emitter, Manager,
};
use tauri_plugin_clipboard_manager::ClipboardExt;
use tauri_plugin_global_shortcut::{GlobalShortcutExt, Shortcut, ShortcutState};

mod audio;
mod modes;
mod paste;
mod provider;
mod settings;

use modes::Mode;
use provider::Provider;

// ── Zustand ───────────────────────────────────────────────────────────────────

#[derive(Default)]
struct RecordingInner {
    is_recording: bool,
    recorder: Option<audio::Recorder>,
    target_hwnd: Option<isize>,
    active_mode: Option<Mode>,
}

struct AppState(Mutex<RecordingInner>);

// ── Befehle (vom Frontend aufrufbar) ──────────────────────────────────────────

#[tauri::command]
fn get_settings(app: AppHandle) -> settings::Settings {
    settings::load(&app)
}

#[tauri::command]
fn save_settings(app: AppHandle, s: settings::Settings) -> Result<(), String> {
    settings::save(&app, &s)
}

#[tauri::command]
fn save_api_key(provider: String, key: String) -> Result<(), String> {
    let key = key.trim().to_string();
    if key.is_empty() {
        return Err("Bitte einen API-Schluessel eingeben.".to_string());
    }
    settings::save_api_key(&provider, &key)
}

#[tauri::command]
fn delete_api_key(provider: String) -> Result<(), String> {
    settings::delete_api_key(&provider)
}

#[tauri::command]
fn has_api_key(provider: String) -> bool {
    settings::has_api_key(&provider)
}

#[tauri::command]
fn api_key_display(provider: String) -> String {
    settings::api_key_display(&provider)
}

#[tauri::command]
async fn test_connection(app: AppHandle) -> Result<(), String> {
    let s = settings::load(&app);
    let api_key = settings::get_api_key(&s.provider)?;
    Provider::from_id(&s.provider)
        .test_connection(&api_key, &s.chat_model)
        .await
}

// ── Kernlogik ─────────────────────────────────────────────────────────────────

async fn do_toggle(app: &AppHandle, mode: Mode) -> Result<(), String> {
    let recording = {
        let state = app.state::<AppState>();
        let inner = state.0.lock().map_err(|e| e.to_string())?;
        inner.is_recording
    };
    if recording {
        let result = do_stop_and_process(app).await;
        // HUD nach kurzer Ergebnis-Anzeige wieder ausblenden (egal ob Erfolg oder Fehler).
        schedule_hide_hud(app);
        result
    } else {
        do_start(app, mode)
    }
}

fn do_start(app: &AppHandle, mode: Mode) -> Result<(), String> {
    let s = settings::load(app);
    if s.provider != "local" && !settings::has_api_key(&s.provider) {
        app.emit(
            "show-error",
            format!(
                "Kein API-Schluessel fuer {}. Bitte in den Einstellungen eintragen.",
                s.provider
            ),
        )
        .ok();
        open_settings_window(app);
        return Ok(());
    }

    let target_hwnd = paste::capture_foreground_window();

    let recorder = audio::Recorder::new()?;
    recorder.start()?;

    let state = app.state::<AppState>();
    let mut inner = state.0.lock().map_err(|e| e.to_string())?;
    inner.recorder = Some(recorder);
    inner.target_hwnd = target_hwnd;
    inner.active_mode = Some(mode);
    inner.is_recording = true;
    drop(inner);

    // HUD einblenden, damit man die Aufnahme im Bild sieht (Fenster startet sonst versteckt).
    // Bewusst OHNE Fokus: das Zielfenster wurde oben gesichert, das Einfuegen stellt es wieder her.
    show_hud(app);
    app.emit(
        "recording-started",
        serde_json::json!({ "label": mode.label(), "hotkey": mode.default_hotkey() }),
    )
    .ok();
    Ok(())
}

async fn do_stop_and_process(app: &AppHandle) -> Result<(), String> {
    let (recorder, target_hwnd, mode) = {
        let state = app.state::<AppState>();
        let mut inner = state.0.lock().map_err(|e| e.to_string())?;
        inner.is_recording = false;
        (
            inner.recorder.take(),
            inner.target_hwnd.take(),
            inner.active_mode.take(),
        )
    };

    let Some(recorder) = recorder else {
        return Ok(());
    };
    let mode = mode.unwrap_or(Mode::Dictate);

    let duration = recorder.recording_duration_seconds();
    app.emit("recording-stopped", duration).ok();
    if duration < 0.4 {
        app.emit("show-error", "Keine Aufnahme erkannt.").ok();
        return Ok(());
    }

    let wav_path = recorder.stop_and_save()?;
    let s = settings::load(app);
    let provider = Provider::from_id(&s.provider);

    app.emit("status-update", "Wird transkribiert ...").ok();
    // Transkriptions-Sprache: der EN->DE-Modus erwartet ENGLISCHE Sprache -> sonst verhoert sich
    // Whisper mit der (deutschen) Standardsprache. Alle anderen Modi nutzen die Einstellung.
    let stt_lang = if mode == Mode::TranslateEnDe {
        "en"
    } else {
        s.language.as_str()
    };
    // Lokale Transkription läuft über den whisper.cpp-Server (kein Schlüssel);
    // Cloud-Anbieter brauchen einen Schlüssel.
    let transcribe_result = if provider == Provider::Local {
        // "localhost" -> "127.0.0.1": feste IPv4-Loopback, funktioniert auch offline/mit VPN.
        let stt_url = s.local_stt_url.replace("localhost", "127.0.0.1");
        provider::transcribe_local(&stt_url, &wav_path, &s.transcription_model, Some(stt_lang)).await
    } else {
        match settings::get_api_key(&s.provider) {
            Ok(api_key) => {
                provider
                    .transcribe(&wav_path, &api_key, &s.transcription_model, Some(stt_lang))
                    .await
            }
            Err(e) => Err(e),
        }
    };
    let text = match transcribe_result {
        Ok(t) => {
            let _ = std::fs::remove_file(&wav_path);
            t
        }
        Err(e) => {
            let _ = std::fs::remove_file(&wav_path);
            app.emit("show-error", &e).ok();
            return Err(e);
        }
    };

    if text.trim().is_empty() {
        app.emit("show-error", "Keine Aufnahme erkannt.").ok();
        return Ok(());
    }

    // Modi ausser Diktat: zweite Phase durch das Textmodell. Schlaegt sie fehl,
    // liefern wir den reinen Transkript-Text als Rueckfall.
    let final_text = if mode.needs_chat() {
        // Textmodell-Anbieter getrennt von der Transkription. "local" = Ollama (kein Schluessel).
        let chat_provider = Provider::from_id(&s.chat_provider);
        let chat_model = if chat_provider == Provider::Local {
            s.local_chat_model.as_str()
        } else {
            s.chat_model.as_str()
        };
        let chat_key = if chat_provider == Provider::Local {
            String::new()
        } else {
            match settings::get_api_key(&s.chat_provider) {
                Ok(k) => k,
                Err(e) => {
                    app.emit("show-error", &e).ok();
                    String::new()
                }
            }
        };
        app.emit("status-update", mode.processing_label()).ok();
        let first = match chat_provider
            .chat(mode.system_prompt(), &text, &chat_key, chat_model, mode.temperature())
            .await
        {
            Ok(t) => t,
            Err(e) => {
                app.emit("show-error", &e).ok();
                text
            }
        };
        // Entschaerfen: zweiter Pass glaettet die Sprache. Kleine lokale Modelle erzeugen beim
        // Umschreiben Grammatikfehler/Fremdwort-Einsprengsel; ein Lektor-Pass buegelt das aus.
        // Schlaegt er fehl, behalten wir die erste (entschaerfte) Fassung.
        if mode == Mode::Vent {
            app.emit("status-update", "Wird sprachlich geglättet ...").ok();
            match chat_provider
                .chat(modes::VENT_POLISH_PROMPT, &first, &chat_key, chat_model, 0.2)
                .await
            {
                Ok(t) => t,
                Err(_) => first,
            }
        } else {
            first
        }
    } else {
        text
    };

    app.clipboard()
        .write_text(&final_text)
        .map_err(|e| format!("Clipboard-Fehler: {}", e))?;
    app.emit("transcription-result", &final_text).ok();

    if s.auto_paste_enabled {
        app.emit("status-update", "Wird eingefuegt ...").ok();
        let paste_result =
            tauri::async_runtime::spawn_blocking(move || paste::paste_at_cursor(target_hwnd))
                .await
                .map_err(|e| e.to_string())?;
        if let Err(e) = paste_result {
            app.emit(
                "show-error",
                format!("Text ist in der Zwischenablage - bitte Strg+V druecken. ({})", e),
            )
            .ok();
        }
    }

    Ok(())
}

// ── Fenster-Helfer ────────────────────────────────────────────────────────────

/// Zeigt das rahmenlose Statusfenster als HUD (Head-up-Display) waehrend Aufnahme/Verarbeitung.
fn show_hud(app: &AppHandle) {
    if let Some(win) = app.get_webview_window("main") {
        win.show().ok();
    }
}

/// Blendet das HUD nach kurzer Anzeige (4 s) wieder aus -- aber nur, wenn nicht inzwischen
/// eine neue Aufnahme laeuft (sonst wuerde das gerade gezeigte HUD weggerissen).
fn schedule_hide_hud(app: &AppHandle) {
    let handle = app.clone();
    std::thread::spawn(move || {
        std::thread::sleep(std::time::Duration::from_secs(4));
        let still_recording = handle
            .state::<AppState>()
            .0
            .lock()
            .map(|inner| inner.is_recording)
            .unwrap_or(false);
        if still_recording {
            return;
        }
        if let Some(win) = handle.get_webview_window("main") {
            win.hide().ok();
        }
    });
}

/// Platziert das Statusfenster initial rechts-mittig am Bildschirmrand.
/// Danach ist es per Kopfzeile (data-tauri-drag-region) frei verschiebbar.
fn position_right_center(win: &tauri::WebviewWindow) {
    if let (Ok(Some(monitor)), Ok(wsize)) = (win.primary_monitor(), win.outer_size()) {
        let msize = monitor.size();
        let mpos = monitor.position();
        let margin = (24.0 * monitor.scale_factor()) as i32;
        let x = mpos.x + msize.width as i32 - wsize.width as i32 - margin;
        let y = mpos.y + (msize.height as i32 - wsize.height as i32) / 2;
        win.set_position(tauri::PhysicalPosition::new(x, y)).ok();
    }
}

fn toggle_main_window(app: &AppHandle) {
    if let Some(win) = app.get_webview_window("main") {
        if win.is_visible().unwrap_or(false) {
            win.hide().ok();
        } else {
            win.show().ok();
            win.set_focus().ok();
        }
    }
}

fn open_settings_window(app: &AppHandle) {
    if let Some(win) = app.get_webview_window("settings") {
        win.show().ok();
        win.set_focus().ok();
    }
}

// ── Main ──────────────────────────────────────────────────────────────────────

fn main() {
    tauri::Builder::default()
        .plugin(tauri_plugin_global_shortcut::Builder::new().build())
        .plugin(tauri_plugin_store::Builder::default().build())
        .plugin(tauri_plugin_clipboard_manager::init())
        .manage(AppState(Mutex::new(RecordingInner::default())))
        .invoke_handler(tauri::generate_handler![
            get_settings,
            save_settings,
            save_api_key,
            delete_api_key,
            has_api_key,
            api_key_display,
            test_connection,
        ])
        .setup(|app| {
            // Statusfenster initial rechts-mittig platzieren (per Kopfzeile verschiebbar).
            if let Some(win) = app.get_webview_window("main") {
                position_right_center(&win);
            }

            // Tray mit Mikrofon-Icon (eingebettet -> funktioniert im portablen Build).
            let show_item =
                MenuItem::with_id(app, "show", "Fenster anzeigen", true, None::<&str>)?;
            let quit_item = MenuItem::with_id(app, "quit", "Blixt beenden", true, None::<&str>)?;
            let settings_item =
                MenuItem::with_id(app, "settings", "Einstellungen", true, None::<&str>)?;
            let menu = Menu::with_items(app, &[&show_item, &settings_item, &quit_item])?;

            let mic_icon = tauri::image::Image::from_bytes(include_bytes!("../icons/tray-mic.png"))?;

            TrayIconBuilder::new()
                .icon(mic_icon)
                .menu(&menu)
                .tooltip("Blixt")
                .on_menu_event(|app, event| match event.id.as_ref() {
                    "quit" => app.exit(0),
                    "settings" => open_settings_window(app),
                    "show" => {
                        if let Some(win) = app.get_webview_window("main") {
                            win.show().ok();
                            win.set_focus().ok();
                        }
                    }
                    _ => {}
                })
                .on_tray_icon_event(|tray, event| {
                    if let TrayIconEvent::Click {
                        button: MouseButton::Left,
                        ..
                    } = event
                    {
                        toggle_main_window(tray.app_handle());
                    }
                })
                .build(app)?;

            // Globale Hotkeys: ein eigener je Modus (Toggle: druecken = start, nochmal = stop).
            let gs = app.global_shortcut();
            for mode in Mode::all() {
                let hk = mode.default_hotkey();
                let shortcut: Shortcut = match hk.parse() {
                    Ok(s) => s,
                    Err(e) => {
                        eprintln!("[Blixt] Hotkey '{}' ungueltig: {:?}", hk, e);
                        continue;
                    }
                };
                let handle = app.handle().clone();
                let m = mode;
                if let Err(e) = gs.on_shortcut(shortcut, move |_app, _sc, event| {
                    if event.state == ShortcutState::Pressed {
                        let h = handle.clone();
                        tauri::async_runtime::spawn(async move {
                            if let Err(e) = do_toggle(&h, m).await {
                                h.emit("show-error", e).ok();
                            }
                        });
                    }
                }) {
                    eprintln!("[Blixt] Hotkey '{}' konnte nicht registriert werden: {}", hk, e);
                }
            }

            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("Blixt konnte nicht gestartet werden");
}
