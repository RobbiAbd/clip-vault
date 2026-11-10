// Prevents additional console window on Windows in release builds
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod clipboard;
mod commands;
mod models;
mod storage;

use anyhow::Result;
use std::path::PathBuf;
use std::sync::Mutex;
use std::thread;
use std::time::Duration;
use tauri::{
    CustomMenuItem, Manager, SystemTray, SystemTrayEvent, SystemTrayMenu, SystemTrayMenuItem,
    WindowEvent,
};

use clipboard::ClipboardManager;
use commands::AppState;
use storage::DataStore;

fn main() {
    env_logger::init();

    let app_result = tauri::Builder::default()
        .setup(|app| {
            let app_dir = app.path_resolver().app_data_dir().unwrap();
            std::fs::create_dir_all(&app_dir)?;

            let db_path = app_dir.join("clipboard.db");
            log::info!("Database path: {:?}", db_path);

            let data_store = DataStore::new(db_path)?;

            let state = AppState {
                data_store: Mutex::new(data_store),
            };

            app.manage(state);

            let app_handle = app.handle();
            start_clipboard_monitor(app_handle);

            Ok(())
        })
        .system_tray(create_system_tray())
        .on_system_tray_event(|app, event| match event {
            SystemTrayEvent::LeftClick { .. } => {
                let window = app.get_window("main").unwrap();
                if window.is_visible().unwrap() {
                    window.hide().unwrap();
                } else {
                    window.show().unwrap();
                    window.set_focus().unwrap();
                }
            }
            SystemTrayEvent::MenuItemClick { id, .. } => match id.as_str() {
                "show" => {
                    let window = app.get_window("main").unwrap();
                    window.show().unwrap();
                    window.set_focus().unwrap();
                }
                "quit" => {
                    std::process::exit(0);
                }
                _ => {}
            },
            _ => {}
        })
        .on_window_event(|event| {
            if let WindowEvent::CloseRequested { api, .. } = event.event() {
                event.window().hide().unwrap();
                api.prevent_close();
            }
        })
        .invoke_handler(tauri::generate_handler![
            commands::get_history,
            commands::get_entry,
            commands::search_history,
            commands::delete_entry,
            commands::toggle_pin,
            commands::clear_history,
            commands::get_stats,
            commands::copy_to_clipboard,
        ])
        .build(tauri::generate_context!())
        .expect("error while running tauri application");

    app_result.run(|_app_handle, event| {
        if let tauri::RunEvent::ExitRequested { api, .. } = event {
            api.prevent_exit();
        }
    });
}

fn create_system_tray() -> SystemTray {
    let show = CustomMenuItem::new("show".to_string(), "Show Clip-Vault");
    let quit = CustomMenuItem::new("quit".to_string(), "Quit");
    
    let tray_menu = SystemTrayMenu::new()
        .add_item(show)
        .add_native_item(SystemTrayMenuItem::Separator)
        .add_item(quit);

    SystemTray::new().with_menu(tray_menu)
}

fn start_clipboard_monitor(app_handle: tauri::AppHandle) {
    thread::spawn(move || {
        let mut manager = ClipboardManager::new().expect("Failed to create clipboard manager");
        
        manager.start().expect("Failed to start clipboard monitor");
        log::info!("Clipboard monitoring started");

        loop {
            if let Some(entry) = manager.get_latest_entry() {
                log::debug!("New clipboard entry captured");
                
                let state: tauri::State<AppState> = app_handle.state();
                let store = state.data_store.lock().unwrap();
                
                match store.save_entry(&entry) {
                    Ok(id) => {
                        log::info!("Saved clipboard entry with id: {}", id);
                        
                        app_handle.emit_all("clipboard-update", ()).ok();
                    }
                    Err(e) => {
                        log::error!("Failed to save clipboard entry: {}", e);
                    }
                }
            }

            thread::sleep(Duration::from_millis(200));
        }
    });
}
