use std::time::Duration;
use tauri::{
    AppHandle, Emitter, Manager, PhysicalPosition, Runtime, WebviewUrl, WebviewWindow, WebviewWindowBuilder,
};

pub const MAIN_WINDOW: &str = "main";
pub const POPUP_WINDOW: &str = "popup";
const POPUP_SIZE: (f64, f64) = (380.0, 500.0);
/// Gap between the cursor and the popup so the pointer doesn't cover the first item.
const POPUP_CURSOR_OFFSET: i32 = 12;
/// Time for the previously focused app to regain focus after Clip-Vault hides.
const PASTE_DELAY: Duration = Duration::from_millis(150);

pub fn show_main(app: &AppHandle) {
    hide_popup(app);
    let Some(window) = app.get_webview_window(MAIN_WINDOW) else { return };
    let _ = window.unminimize();
    let _ = window.show();
    let _ = window.set_focus();
}

pub fn toggle_main(app: &AppHandle) {
    let Some(window) = app.get_webview_window(MAIN_WINDOW) else { return };
    if window.is_visible().unwrap_or(false) {
        let _ = window.hide();
    } else {
        show_main(app);
    }
}

/// The popup is created hidden at startup so the shortcut opens it instantly.
pub fn create_popup<R: Runtime>(app: &tauri::App<R>) -> tauri::Result<()> {
    WebviewWindowBuilder::new(app, POPUP_WINDOW, WebviewUrl::App("popup.html".into()))
        .title("Clip-Vault")
        .inner_size(POPUP_SIZE.0, POPUP_SIZE.1)
        .decorations(false)
        .resizable(false)
        .skip_taskbar(true)
        .always_on_top(true)
        .visible(false)
        .build()?;
    Ok(())
}

pub fn toggle_popup(app: &AppHandle) {
    let Some(popup) = app.get_webview_window(POPUP_WINDOW) else { return };
    if popup.is_visible().unwrap_or(false) {
        let _ = popup.hide();
        return;
    }

    place_near_cursor(app, &popup);
    let _ = popup.show();
    let _ = popup.set_focus();
    let _ = popup.emit("popup-shown", ());
}

pub fn hide_popup(app: &AppHandle) {
    if let Some(popup) = app.get_webview_window(POPUP_WINDOW) {
        let _ = popup.hide();
    }
}

/// Hides `window` so focus returns to the previous app, then sends the paste keystroke there.
pub fn paste_into_previous_app(window: &WebviewWindow) {
    let _ = window.hide();
    std::thread::spawn(|| {
        std::thread::sleep(PASTE_DELAY);
        send_paste_keystroke();
    });
}

/// Opens below-right of the cursor, flipping to the other side when it would leave the monitor.
fn place_near_cursor(app: &AppHandle, popup: &WebviewWindow) {
    let Ok(cursor) = app.cursor_position() else { return };
    let Ok(Some(monitor)) = app.monitor_from_point(cursor.x, cursor.y) else { return };
    let Ok(size) = popup.outer_size() else { return };

    let area = monitor.work_area();
    let (width, height) = (size.width as i32, size.height as i32);
    let (cx, cy) = (cursor.x as i32, cursor.y as i32);
    let (left, top) = (area.position.x, area.position.y);
    let (right, bottom) = (left + area.size.width as i32, top + area.size.height as i32);

    let mut x = cx + POPUP_CURSOR_OFFSET;
    if x + width > right {
        x = cx - POPUP_CURSOR_OFFSET - width;
    }
    let mut y = cy + POPUP_CURSOR_OFFSET;
    if y + height > bottom {
        y = cy - POPUP_CURSOR_OFFSET - height;
    }

    let x = x.clamp(left, (right - width).max(left));
    let y = y.clamp(top, (bottom - height).max(top));
    let _ = popup.set_position(PhysicalPosition::new(x, y));
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

    // SAFETY: `inputs` is a live, fully initialised array and the count/size arguments describe it exactly.
    let sent = unsafe { SendInput(inputs.len() as u32, inputs.as_ptr(), std::mem::size_of::<INPUT>() as i32) };
    if sent as usize != inputs.len() {
        log::warn!("Auto-paste keystroke was blocked; the clip is still on the clipboard");
    }
}

#[cfg(not(windows))]
fn send_paste_keystroke() {
    log::info!("Auto-paste is only supported on Windows; the clip was copied to the clipboard");
}
