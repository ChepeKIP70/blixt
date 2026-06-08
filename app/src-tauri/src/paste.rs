// Auto-Einfuegen am Cursor (Windows-Port der macOS-Logik aus blitztext-app / AppState.swift).
//
// Ablauf: 1) beim Aufnahme-Start aktives Fremd-Fenster merken (capture_foreground_window)
//         2) Transkript liegt in der Zwischenablage (main.rs)
//         3) Fokus auf das Ziel-Fenster zurueckholen (SetForegroundWindow + Wiederhol-Schleife)
//         4) Strg+V simulieren (enigo -> SendInput)
//         5) Text bleibt als Rueckfall in der Zwischenablage, falls das Einfuegen blockiert ist
//            (z.B. Ziel-Fenster als Administrator -> UIPI sperrt SendInput).
//
// Laeuft auf einem Blocking-Thread (spawn_blocking in main.rs): Win32-/Eingabe-Aufrufe sind
// synchron, und so wird nichts Nicht-Send-faehiges ueber einen .await-Punkt getragen.

#[cfg(windows)]
pub fn capture_foreground_window() -> Option<isize> {
    use windows::Win32::System::Threading::GetCurrentProcessId;
    use windows::Win32::UI::WindowsAndMessaging::{GetForegroundWindow, GetWindowThreadProcessId};

    unsafe {
        let hwnd = GetForegroundWindow();
        if hwnd.0.is_null() {
            return None;
        }
        let mut pid: u32 = 0;
        GetWindowThreadProcessId(hwnd, Some(&mut pid as *mut u32));
        if pid == GetCurrentProcessId() {
            return None; // eigenes Fenster ignorieren
        }
        Some(hwnd.0 as isize)
    }
}

#[cfg(not(windows))]
pub fn capture_foreground_window() -> Option<isize> {
    None
}

#[cfg(windows)]
pub fn paste_at_cursor(target: Option<isize>) -> Result<(), String> {
    use std::{thread, time::Duration};

    use enigo::{Direction, Enigo, Key, Keyboard, Settings};
    use windows::Win32::Foundation::HWND;
    use windows::Win32::UI::WindowsAndMessaging::{GetForegroundWindow, SetForegroundWindow};

    // Pflicht-Pause: gibt der Zwischenablage Zeit, den neuen Inhalt zu uebernehmen.
    thread::sleep(Duration::from_millis(50));

    // Fokus zurueckholen (max. 5 Versuche, je 50 ms).
    if let Some(raw) = target {
        let hwnd = HWND(raw as *mut core::ffi::c_void);
        unsafe {
            for _ in 0..5 {
                if GetForegroundWindow().0 == hwnd.0 {
                    break;
                }
                let _ = SetForegroundWindow(hwnd);
                thread::sleep(Duration::from_millis(50));
            }
        }
    }

    // Strg+V simulieren. Control wird IMMER wieder losgelassen, auch wenn der Druck dazwischen scheitert.
    let mut enigo = Enigo::new(&Settings::default())
        .map_err(|e| format!("Eingabe-Simulation nicht verfuegbar: {}", e))?;

    enigo
        .key(Key::Control, Direction::Press)
        .map_err(|e| format!("Strg konnte nicht gedrueckt werden: {}", e))?;
    let click = enigo.key(Key::Unicode('v'), Direction::Click);
    let release = enigo.key(Key::Control, Direction::Release);

    click.map_err(|e| format!("V konnte nicht gedrueckt werden: {}", e))?;
    release.map_err(|e| format!("Strg konnte nicht losgelassen werden: {}", e))?;

    Ok(())
}

#[cfg(not(windows))]
pub fn paste_at_cursor(_target: Option<isize>) -> Result<(), String> {
    Err("Auto-Einfuegen wird auf dieser Plattform nicht unterstuetzt.".to_string())
}
