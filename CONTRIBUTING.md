# Contributing to Blixt

Blixt is intentionally small and hackable. Contributions, bug reports and ideas are welcome.

## Building

Prerequisites: [Rust](https://rustup.rs) (MSVC toolchain), [Node.js](https://nodejs.org), WebView2 (pre-installed on Windows 11).

```powershell
cd app
npm install
npx tauri icon src-tauri/icons/icon-source.png   # once
npm run build -- --no-bundle
```

A debug build (`npm run build -- --debug --no-bundle`) enables the WebView DevTools (right-click → Inspect) for the settings/status windows.

## Project layout

- `app/src-tauri/src/` — Rust core (see the architecture table in the README)
- `app/src/` — frontend (plain HTML/JS/CSS, no framework)
- `docs/` — additional documentation (e.g. offline setup)

## Code style

- Keep it small and readable; match the surrounding style.
- Rust: idiomatic, `Result<_, String>` for user-facing errors, German user-facing strings.
- New modes live entirely in `modes.rs` (a label, hotkey, system prompt, temperature) — the hotkey loop in `main.rs` picks them up automatically.

## Pull requests

- One focused change per PR.
- Make sure `cargo check` is clean and the app still builds.
- Describe what you changed and why.

## Scope

This is an experimental preview, not a polished product. Feature requests are welcome, but the goal is to stay small and understandable rather than feature-complete.
