use std::time::Duration;
use tauri::{AppHandle, Emitter, Manager, PhysicalPosition, WebviewWindow};

const MAIN_WINDOW: &str = "main";
/// Time for the previously focused app to regain focus after Clip-Vault hides.
const PASTE_DELAY: Duration = Duration::from_millis(150);

/// `popup` means the window was summoned by the global shortcut: it opens on the monitor under
/// the cursor and the frontend pastes the chosen clip into the previously focused app.
pub fn show_main(app: &AppHandle, popup: bool) {
    let Some(window) = app.get_webview_window(MAIN_WINDOW) else { return };
    if popup {
        center_on_cursor_monitor(app, &window);
    }
    let _ = window.unminimize();
    let _ = window.show();
    let _ = window.set_focus();
    let _ = app.emit("window-shown", popup);
}

pub fn hide_main(app: &AppHandle) {
    if let Some(window) = app.get_webview_window(MAIN_WINDOW) {
        let _ = window.hide();
    }
}

pub fn toggle_main(app: &AppHandle, popup: bool) {
    let Some(window) = app.get_webview_window(MAIN_WINDOW) else { return };
    let active = window.is_visible().unwrap_or(false) && (!popup || window.is_focused().unwrap_or(false));
    if active {
        let _ = window.hide();
    } else {
        show_main(app, popup);
    }
}

/// Hides Clip-Vault so focus returns to the previous app, then sends the paste keystroke there.
pub fn paste_into_previous_app(app: &AppHandle) {
    hide_main(app);
    std::thread::spawn(|| {
        std::thread::sleep(PASTE_DELAY);
        send_paste_keystroke();
    });
}

fn center_on_cursor_monitor(app: &AppHandle, window: &WebviewWindow) {
    let Ok(cursor) = app.cursor_position() else { return };
    let Ok(Some(monitor)) = app.monitor_from_point(cursor.x, cursor.y) else { return };
    let Ok(size) = window.outer_size() else { return };

    let (origin, area) = (monitor.position(), monitor.size());
    let x = origin.x + (area.width as i32 - size.width as i32).max(0) / 2;
    let y = origin.y + (area.height as i32 - size.height as i32).max(0) / 2;
    let _ = window.set_position(PhysicalPosition::new(x, y));
}

#[cfg(windows)]
fn send_paste_keystroke() {
    use windows_sys::Win32::UI::Input::KeyboardAndMouse::{
        SendInput, INPUT, INPUT_0, INPUT_KEYBOARD, KEYBDINPUT, KEYBD_EVENT_FLAGS, KEYEVENTF_KEYUP, VIRTUAL_KEY,
        VK_CONTROL, VK_V,
    };

    let key = |vk: VIRTUAL_KEY, flags: KEYBD_EVENT_FLAGS| INPUT {
        r#type: INPUT_KEYBOARD,
        Anonymous: INPUT_0 {
            ki: KEYBDINPUT { wVk: vk, wScan: 0, dwFlags: flags, time: 0, dwExtraInfo: 0 },
        },
    };
    let inputs = [
        key(VK_CONTROL, 0),
        key(VK_V, 0),
        key(VK_V, KEYEVENTF_KEYUP),
        key(VK_CONTROL, KEYEVENTF_KEYUP),
    ];

    let sent = unsafe { SendInput(inputs.len() as u32, inputs.as_ptr(), std::mem::size_of::<INPUT>() as i32) };
    if sent as usize != inputs.len() {
        log::warn!("Auto-paste keystroke was blocked; the clip is still on the clipboard");
    }
}

#[cfg(not(windows))]
fn send_paste_keystroke() {
    log::info!("Auto-paste is only supported on Windows; the clip was copied to the clipboard");
}
