<div align="center">

# Blixt

**Press a hotkey, speak, and the text appears right where your cursor is.**
Windows speech-to-text with smart modes — cloud or fully offline.

[![License: MIT](https://img.shields.io/badge/License-MIT-orange.svg)](LICENSE)
![Platform: Windows](https://img.shields.io/badge/Platform-Windows-blue.svg)
![Built with Tauri](https://img.shields.io/badge/Built%20with-Tauri%202-24C8DB.svg)

</div>

<!-- DEMO: record a ~60s clip, save it as docs/demo.gif, then uncomment the next line.
     Suggested shots: dictation in Notepad -> "Vent" mode calming an angry rant ->
     pull the network cable and run it again fully offline (whisper.cpp + Ollama).
![Blixt demo](docs/demo.gif)
-->

---

## Table of Contents

- [What is Blixt?](#what-is-blixt)
- [Features](#features)
- [Quick Start](#quick-start)
- [Configuration](#configuration)
- [Offline mode](#offline-mode)
- [Architecture](#architecture)
- [Roadmap](#roadmap)
- [Built with an AI agent](#built-with-an-ai-agent)
- [Credits](#credits)
- [Contributing](#contributing)
- [License](#license)
- [Deutsch 🇩🇪](#blixt-deutsch)

---

## What is Blixt?

Blixt is a small Windows system-tray app for **speech-to-text**: press a hotkey, speak, release — the text is inserted at your cursor automatically. Beyond plain dictation, it offers several **smart modes** (clean-up, calmer rewriting, emojis, translation, prompt-building).

It runs against **swappable providers**: in the cloud (Groq by default, or OpenAI), or **fully offline** on your own machine (whisper.cpp for transcription + Ollama for the text model) — no API key, no internet, your voice never leaves the PC.

Blixt is an independent **Windows** take, derived from the macOS app [blitztext-app](https://github.com/cmagnussen/blitztext-app) (see [Credits](#credits)).

## Features

| Hotkey | Mode | What it does |
|--------|------|--------------|
| `Ctrl+Shift+1` | **Dictate** | Plain transcription, speech → text |
| `Ctrl+Shift+2` | **Improve** | Tidy up spelling, grammar, flow |
| `Ctrl+Shift+3` | **Vent** | Turn an angry rant into a calm, clear message |
| `Ctrl+Shift+4` | **Emoji** | Add fitting emojis to your text |
| `Ctrl+Shift+5` | **Translate** | Speak German → get English text |
| `Ctrl+Shift+6` | **Prompt (EN)** | Speak a rough idea → a structured AI prompt in English |
| `Ctrl+Shift+7` | **Prompt (DE)** | Same, output in German |

Toggle recording: press once to start, press again to finish. The result lands on the clipboard and is auto-pasted at the cursor.

## Quick Start

**Prerequisites:** [Rust](https://rustup.rs) (MSVC toolchain), [Node.js](https://nodejs.org), and WebView2 (pre-installed on Windows 11).

```powershell
cd app
npm install
npx tauri icon src-tauri/icons/icon-source.png   # generate the icon set (once)
npm run build -- --no-bundle                      # portable .exe
```

The binary is created at `app/src-tauri/target/release/blixt.exe`.

### First run

Launch `blixt.exe`. **No window opens — Blixt lives in the system tray** (the orange microphone icon, bottom-right; it may be hidden behind the `^` arrow). Left-click it for the status window, right-click for Settings / Quit.

**Fastest way to try it (cloud, ~2 min):**

1. Tray icon → **Settings** → **Provider: Groq**.
2. Grab a free API key at [console.groq.com/keys](https://console.groq.com/keys), paste it, and click **Test connection**.
3. Click into any text field, press **`Ctrl+Shift+1`**, speak, then press it again — your text appears at the cursor.

No API key wanted? Run it fully offline instead → see [Offline mode](#offline-mode).

## Configuration

Open the tray icon → **Settings**. Transcription and the text model are chosen **independently**:

- **Cloud (default):** pick Groq (free tier) or OpenAI and paste your own API key. The key is stored in the **Windows Credential Manager**, never in the code.
- **Offline:** set both providers to *Local* — see below.

## Offline mode

Blixt can run **completely offline** using local servers:

- **Transcription** → a local [whisper.cpp](https://github.com/ggml-org/whisper.cpp) server (GPU via CUDA/Vulkan, or CPU)
- **Text model** → [Ollama](https://ollama.com) running a local model (e.g. `qwen2.5:7b`)

This is **not turnkey** — you install and run those two local servers yourself. Step-by-step instructions: **[docs/OFFLINE-SETUP.md](docs/OFFLINE-SETUP.md)**.

## Architecture

A thin Tauri 2 app (Rust core + WebView2 UI). Rust modules under `app/src-tauri/src/`:

| Module | Responsibility |
|--------|----------------|
| `main.rs` | Tray, global hotkeys, orchestration |
| `modes.rs` | The 7 modes (labels, hotkeys, system prompts, temperatures) |
| `provider.rs` | Provider layer — OpenAI-compatible transcription + chat (Groq / OpenAI / local) |
| `audio.rs` | Microphone capture (`cpal`) → WAV (`hound`) |
| `paste.rs` | Auto-paste at the cursor (`enigo` + Win32 focus restore) |
| `settings.rs` | Settings + API keys (Windows Credential Manager) |

The provider layer is the key idea: Groq, OpenAI and the local servers all speak the same OpenAI-compatible API, so switching between cloud and offline is just a base URL.

## Roadmap

- ✅ 7 modes, auto-paste, cloud + offline
- ⏳ One-step offline setup (bundled local servers)
- ⏳ User-configurable hotkeys & hold-to-talk mode
- ⏳ Signed release builds + installer
- ⏳ More target languages for translate

## Built with an AI agent

Blixt was built in a single focused session by someone who is **not a Rust developer**, working with an AI coding agent — from analysing the original macOS app, to a working, offline-capable native Windows port. It is an honest demonstration of how far AI-assisted development can take a real native application. The code is human-reviewed and runs; it is intentionally small and hackable, not a polished commercial product.

## Credits

Based on [**blitztext-app**](https://github.com/cmagnussen/blitztext-app) by **cmagnussen** (MIT License). Blixt is an independent re-implementation for Windows (Tauri/Rust), not a code fork — with its **own name, icon and branding** as required by the original project's trademark notice.

**What's different from the original:** Windows instead of macOS, a swappable provider layer (Groq/OpenAI/local) instead of OpenAI-only, a fully offline mode, and two extra modes (Translate, Prompt).

## Contributing

Small, hackable, PRs welcome — see [CONTRIBUTING.md](CONTRIBUTING.md).

## License

[MIT](LICENSE).

<br>

---
---

<br>

<div align="center">

# Blixt (Deutsch)

**Taste drücken, sprechen — der Text erscheint direkt am Cursor.**
Windows-Sprache-zu-Text mit cleveren Modi — Cloud oder komplett offline.

</div>

## Was ist Blixt?

Blixt ist eine kleine Windows-App (im System-Tray, also dem Symbolbereich unten rechts) für **Sprache-zu-Text**: Taste drücken, sprechen, loslassen — der Text wird automatisch am Cursor eingefügt. Neben reinem Diktat gibt es mehrere **clevere Modi** (Aufpolieren, Entschärfen, Emojis, Übersetzen, Prompt-Bauen).

Es nutzt **austauschbare Anbieter**: in der Cloud (Groq als Standard, oder OpenAI) oder **komplett offline** auf deinem eigenen Rechner (whisper.cpp für die Transkription + Ollama für das Textmodell) — kein Schlüssel, kein Internet, deine Stimme verlässt den PC nie.

Blixt ist eine eigenständige **Windows**-Variante, abgeleitet aus der macOS-App [blitztext-app](https://github.com/cmagnussen/blitztext-app) (siehe [Credits](#credits-1)).

## Funktionen

| Hotkey | Modus | Funktion |
|--------|-------|----------|
| `Ctrl+Shift+1` | **Diktat** | Reine Transkription, Sprache → Text |
| `Ctrl+Shift+2` | **Verbessern** | Rechtschreibung, Grammatik, Lesefluss |
| `Ctrl+Shift+3` | **Entschärfen** | Wütendes Reden → ruhige, klare Nachricht |
| `Ctrl+Shift+4` | **Emoji** | Passende Emojis in den Text |
| `Ctrl+Shift+5` | **Übersetzen** | Deutsch sprechen → englischer Text |
| `Ctrl+Shift+6` | **Prompt (EN)** | Grobe Idee sprechen → strukturierter KI-Prompt auf Englisch |
| `Ctrl+Shift+7` | **Prompt (DE)** | Dasselbe, Ausgabe auf Deutsch |

Toggle-Aufnahme: einmal drücken = Start, nochmal drücken = fertig. Das Ergebnis landet in der Zwischenablage und wird am Cursor eingefügt.

## Schnellstart

**Voraussetzungen:** [Rust](https://rustup.rs) (MSVC-Toolchain), [Node.js](https://nodejs.org), WebView2 (auf Windows 11 vorinstalliert).

```powershell
cd app
npm install
npx tauri icon src-tauri/icons/icon-source.png   # Icon-Set einmalig erzeugen
npm run build -- --no-bundle                      # portable .exe
```

Ergebnis: `app/src-tauri/target/release/blixt.exe`.

### Erster Start

`blixt.exe` starten. **Es öffnet sich kein Fenster — Blixt lebt im System-Tray** (oranges Mikrofon-Symbol unten rechts, evtl. hinter dem `^`-Pfeil versteckt). Linksklick zeigt das Statusfenster, Rechtsklick öffnet Einstellungen / Beenden.

**Schnellster Test (Cloud, ~2 Min):**

1. Tray-Symbol → **Einstellungen** → **Anbieter: Groq**.
2. Kostenlosen API-Schlüssel auf [console.groq.com/keys](https://console.groq.com/keys) holen, einfügen, **Verbindung testen**.
3. In ein beliebiges Textfeld klicken, **`Strg+Umschalt+1`** drücken, sprechen, nochmal drücken — der Text erscheint am Cursor.

Keinen Schlüssel gewünscht? Stattdessen komplett offline → siehe [Offline-Modus](#offline-modus).

## Konfiguration

Tray-Symbol → **Einstellungen**. Transkription und Textmodell wählst du **getrennt**:

- **Cloud (Standard):** Groq (kostenloses Kontingent) oder OpenAI + eigener API-Schlüssel. Der Schlüssel liegt im **Windows Credential Manager**, nie im Code.
- **Offline:** beide Anbieter auf *Lokal* — siehe unten.

## Offline-Modus

Blixt läuft **komplett offline** über lokale Server:

- **Transkription** → lokaler [whisper.cpp](https://github.com/ggml-org/whisper.cpp)-Server (GPU via CUDA/Vulkan oder CPU)
- **Textmodell** → [Ollama](https://ollama.com) mit lokalem Modell (z.B. `qwen2.5:7b`)

Das ist **nicht turnkey** — die zwei lokalen Server richtest du selbst ein. Schritt für Schritt: **[docs/OFFLINE-SETUP.md](docs/OFFLINE-SETUP.md)**.

## Architektur

Schlanke Tauri-2-App (Rust-Kern + WebView2-Oberfläche). Rust-Module unter `app/src-tauri/src/`:

| Modul | Aufgabe |
|-------|---------|
| `main.rs` | Tray, globale Hotkeys, Ablaufsteuerung |
| `modes.rs` | Die 7 Modi (Bezeichnung, Hotkey, System-Prompt, Temperatur) |
| `provider.rs` | Anbieter-Schicht — OpenAI-kompatible Transkription + Chat (Groq / OpenAI / lokal) |
| `audio.rs` | Mikrofon (`cpal`) → WAV (`hound`) |
| `paste.rs` | Auto-Einfügen am Cursor (`enigo` + Win32-Fokus-Wiederherstellung) |
| `settings.rs` | Einstellungen + API-Schlüssel (Windows Credential Manager) |

Kernidee ist die Anbieter-Schicht: Groq, OpenAI und die lokalen Server sprechen dieselbe OpenAI-kompatible Schnittstelle — der Wechsel Cloud ↔ offline ist nur eine andere Basis-URL.

## Roadmap

- ✅ 7 Modi, Auto-Einfügen, Cloud + offline
- ⏳ Ein-Schritt-Offline-Setup (gebündelte lokale Server)
- ⏳ Frei konfigurierbare Hotkeys & Halten-zum-Sprechen
- ⏳ Signierte Release-Builds + Installer
- ⏳ Mehr Zielsprachen beim Übersetzen

## Mit einem KI-Agenten gebaut

Blixt entstand in einer einzigen fokussierten Sitzung — von jemandem, der **kein Rust-Entwickler** ist, zusammen mit einem KI-Coding-Agenten: von der Analyse der Original-macOS-App bis zu einem lauffähigen, offline-fähigen nativen Windows-Port. Es ist eine ehrliche Demonstration, wie weit KI-gestützte Entwicklung eine echte native Anwendung tragen kann. Der Code ist menschlich geprüft und läuft; er ist bewusst klein und hackbar, kein poliertes Kommerzprodukt.

## Credits

Basiert auf [**blitztext-app**](https://github.com/cmagnussen/blitztext-app) von **cmagnussen** (MIT-Lizenz). Blixt ist eine eigenständige Neu-Implementierung für Windows (Tauri/Rust), **kein** Code-Fork — mit **eigenem Namen, Icon und Branding**, wie es der Marken-Hinweis des Originals verlangt.

**Was anders ist:** Windows statt macOS, austauschbare Anbieter-Schicht (Groq/OpenAI/lokal) statt nur OpenAI, ein komplett offline-Modus, und zwei zusätzliche Modi (Übersetzen, Prompt).

## Mitwirken

Klein, hackbar, PRs willkommen — siehe [CONTRIBUTING.md](CONTRIBUTING.md).

## Lizenz

[MIT](LICENSE).
