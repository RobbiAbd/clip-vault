use tauri::plugin::TauriPlugin;
use tauri::{AppHandle, Wry};
use tauri_plugin_global_shortcut::{GlobalShortcutExt, Shortcut, ShortcutState};

pub fn plugin() -> TauriPlugin<Wry> {
    tauri_plugin_global_shortcut::Builder::new()
        .with_handler(|app, _shortcut, event| {
            if event.state() == ShortcutState::Pressed {
                crate::window::toggle_main(app, true);
            }
        })
        .build()
}

/// Replaces any registered shortcut with `accelerator`, or only clears it when disabled.
pub fn apply(app: &AppHandle, enabled: bool, accelerator: &str) -> Result<(), String> {
    let manager = app.global_shortcut();
    manager
        .unregister_all()
        .map_err(|e| format!("Failed to clear the global shortcut: {}", e))?;
    if !enabled {
        return Ok(());
    }

    let shortcut: Shortcut = accelerator
        .parse()
        .map_err(|e| format!("Invalid shortcut \"{}\": {}", accelerator, e))?;
    manager
        .register(shortcut)
        .map_err(|e| format!("Could not register {} (it may be used by another app): {}", accelerator, e))
}
