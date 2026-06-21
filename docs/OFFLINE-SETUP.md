# Offline setup

Blixt can run **completely offline** — no API key, no internet, your audio never leaves the machine. This requires two local servers that you install and run yourself:

1. **whisper.cpp server** — speech-to-text (transcription)
2. **Ollama** — the local text model (for Improve / Vent / Translate / Prompt …)

Both expose an **OpenAI-compatible API**, which is exactly what Blixt talks to. This is **not turnkey**: it takes ~15 minutes the first time.

> Replace `C:\tools\whisper` below with any folder you like. Commands are PowerShell.

---

## 1. Local text model — Ollama

1. Install [Ollama](https://ollama.com/download) (Windows installer). It starts a local server at `http://127.0.0.1:11434` and runs on login.
2. Pull a model. A 7–8B instruct model with good multilingual quality works well and fits in ~8 GB VRAM:

   ```powershell
   ollama pull qwen2.5:7b
   ```

That's it — Ollama's server is OpenAI-compatible out of the box.

---

## 2. Local transcription — whisper.cpp server

### 2a. Get the server binary

Download a prebuilt **whisper.cpp** release for Windows from the
[releases page](https://github.com/ggml-org/whisper.cpp/releases) and unzip it, e.g. into `C:\tools\whisper`.

- **NVIDIA GPU:** take a `whisper-cublas-*-bin-x64.zip` build (CUDA, fastest). The CUDA runtime DLLs are bundled.
- **AMD / Intel / any GPU:** take a Vulkan build if available.
- **No GPU / fallback:** take the plain `whisper-bin-x64.zip` (CPU).

The server executable is `whisper-server.exe` (inside a `Release` subfolder in the CUDA build).

### 2b. Get a model

Download a GGML model from [Hugging Face](https://huggingface.co/ggerganov/whisper.cpp/tree/main) into the same folder. Good German/English balance:

- `ggml-small.bin` (~466 MB) — fast, solid quality
- `ggml-medium.bin` (~1.5 GB) — better quality, slower

### 2c. ffmpeg (for audio conversion)

whisper.cpp needs 16 kHz audio; the `--convert` flag uses **ffmpeg** to resample. Install it and make sure it's on the `PATH`:

```powershell
winget install Gyan.FFmpeg
```

### 2d. Start the server

```powershell
cd C:\tools\whisper       # folder that contains whisper-server.exe
.\whisper-server.exe `
  -m .\ggml-small.bin `
  --host 127.0.0.1 --port 8765 `
  --inference-path "/v1/audio/transcriptions" `
  --convert -t 4
```

Leave this window open. The server now answers at `http://127.0.0.1:8765/v1/audio/transcriptions`.

> Tip: put that command into a small `start-whisper.ps1` so you can launch it with one double-click after each reboot.

---

## 3. Point Blixt at the local servers

**Transcription is always local** in Blixt — so the whisper.cpp server above is required either way. The **text model** for the smart modes can run locally (Ollama) or on Groq.

In Blixt → tray → **Settings**:

- **Local Whisper server** → `http://127.0.0.1:8765/v1`
- **Local model** (Ollama) → `qwen2.5:7b`
- **Save settings**

Now use any mode (`Ctrl+Shift+1` … `8`):

- **Fully offline:** with no Groq key set (or no internet), every step runs locally. You can pull the network cable / disable Wi-Fi and it keeps working.
- **Hybrid (default once a Groq key is set and you're online):** transcription stays local, the smart text step runs on Groq for speed/quality. Prefix your dictation with the codeword **`vertraulich`** to force that step local too — the codeword is stripped from the result.

> **Important:** always use `127.0.0.1`, **not** `localhost`. `localhost` can resolve to IPv6 (`::1`) and fail when offline or behind a VPN; the literal IPv4 loopback always works.

---

## Performance notes

- The **first** request after starting the whisper server is slow (model warm-up + GPU init). Subsequent ones are much faster.
- On a mid-range GPU, `ggml-small` transcribes a short clip in roughly 1–3 seconds once warm.
- Want better German accuracy? Swap `ggml-small.bin` for `ggml-medium.bin` and restart the server (slower, but more accurate).

## Troubleshooting

- **"Network error 127.0.0.1:8765"** → the whisper server isn't running. Start it (step 2d).
- **Text-model errors** → make sure Ollama is running (`ollama list`) and the model name matches.
- **Garbled / empty transcription** → confirm ffmpeg is on the `PATH` (the `--convert` step needs it).
