// Prevents additional console window on Windows in release builds
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod clipboard;
mod commands;
mod models;
mod settings;
mod storage;
mod thumbnail;

use anyhow::Result;
use std::sync::Mutex;
use std::thread;
use std::time::Duration;
use tauri::{
    Manager, WindowEvent, Emitter,
    menu::{Menu, MenuItem},
    tray::{TrayIconBuilder, TrayIconEvent, MouseButton, MouseButtonState},
};

use clipboard::ClipboardManager;
use commands::AppState;
use models::ContentType;
use settings::SettingsStore;
use storage::DataStore;
use tauri_plugin_autostart::{MacosLauncher, ManagerExt};

const RETENTION_INTERVAL: Duration = Duration::from_secs(60 * 60);

fn main() {
    env_logger::init();

    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_notification::init())
        .plugin(tauri_plugin_updater::Builder::new().build())
        .plugin(tauri_plugin_autostart::init(MacosLauncher::LaunchAgent, None))
        .setup(|app| {
            let app_dir = app.path().app_data_dir().unwrap();
            std::fs::create_dir_all(&app_dir)?;

            let db_path = app_dir.join("clipboard.db");
            log::info!("Database path: {:?}", db_path);

            let data_store = DataStore::new(db_path)?;

            let settings_store = SettingsStore::new(app_dir.join("settings.json"));
            let mut settings = settings_store.load();
            // The OS registration is the source of truth; the user may have toggled it outside the app.
            settings.launch_at_startup = app.autolaunch().is_enabled().unwrap_or(false);

            if let Err(e) = data_store.apply_retention(settings.auto_delete_days, settings.max_history) {
                log::error!("Startup retention failed: {}", e);
            }

            if let Some(window) = app.get_webview_window("main") {
                if !settings.start_minimized {
                    let _ = window.show();
                    let _ = window.set_focus();
                }
            }

            app.manage(AppState {
                data_store: Mutex::new(data_store),
                settings: Mutex::new(settings),
                settings_store,
            });

            // Setup tray icon
            setup_tray(app)?;

            let app_handle = app.handle().clone();
            start_clipboard_monitor(app_handle.clone());
            start_retention_timer(app_handle.clone());

            // TODO: Re-enable auto-updater once API is verified
            // Check for updates on startup (in background)
            // let update_handle = app_handle.clone();
            // tauri::async_runtime::spawn(async move {
            //     tokio::time::sleep(std::time::Duration::from_secs(5)).await;
            //     check_for_updates_on_startup(update_handle).await;
            // });

            Ok(())
        })
        .on_window_event(|window, event| {
            if let WindowEvent::CloseRequested { api, .. } = event {
                let state: tauri::State<AppState> = window.state();
                let close_to_tray = state.settings.lock().unwrap().close_to_tray;
                if close_to_tray {
                    let _ = window.hide();
                    api.prevent_close();
                } else {
                    window.app_handle().exit(0);
                }
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
            commands::copy_image_to_clipboard,
            commands::get_settings,
            commands::save_settings,
            // TODO: Re-enable after fixing updater API
            // commands::check_update,
            // commands::install_update,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

fn setup_tray(app: &mut tauri::App) -> Result<()> {
    let show = MenuItem::with_id(app, "show", "Show Clip-Vault", true, None::<&str>)?;
    let quit = MenuItem::with_id(app, "quit", "Quit", true, None::<&str>)?;
    
    let menu = Menu::with_items(app, &[&show, &quit])?;
    
    let _tray = TrayIconBuilder::new()
        .menu(&menu)
        .icon(app.default_window_icon().unwrap().clone())
        .on_tray_icon_event(|tray, event| {
            if let TrayIconEvent::Click { button, button_state, .. } = event {
                if button == MouseButton::Left && button_state == MouseButtonState::Up {
                    let app = tray.app_handle();
                    if let Some(window) = app.get_webview_window("main") {
                        if window.is_visible().unwrap_or(false) {
                            let _ = window.hide();
                        } else {
                            let _ = window.show();
                            let _ = window.set_focus();
                        }
                    }
                }
            }
        })
        .on_menu_event(|app, event| {
            match event.id.as_ref() {
                "show" => {
                    if let Some(window) = app.get_webview_window("main") {
                        let _ = window.show();
                        let _ = window.set_focus();
                    }
                }
                "quit" => {
                    std::process::exit(0);
                }
                _ => {}
            }
        })
        .build(app)?;

    Ok(())
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
                let settings = state.settings.lock().unwrap().clone();

                if matches!(entry.content_type, ContentType::Image) && !settings.capture_images {
                    log::debug!("Image capture disabled, skipping entry");
                    thread::sleep(Duration::from_millis(200));
                    continue;
                }

                let store = state.data_store.lock().unwrap();
                
                match store.save_entry(&entry) {
                    Ok(id) => {
                        log::info!("Saved clipboard entry with id: {}", id);
                        if let Err(e) = store.apply_retention(settings.auto_delete_days, settings.max_history) {
                            log::error!("Retention failed: {}", e);
                        }
                        
                        let _ = app_handle.emit("clipboard-update", ());
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

fn start_retention_timer(app_handle: tauri::AppHandle) {
    thread::spawn(move || loop {
        thread::sleep(RETENTION_INTERVAL);

        let state: tauri::State<AppState> = app_handle.state();
        let settings = state.settings.lock().unwrap().clone();
        let removed = state
            .data_store
            .lock()
            .unwrap()
            .apply_retention(settings.auto_delete_days, settings.max_history);

        match removed {
            Ok(n) if n > 0 => {
                log::info!("Retention removed {} clips", n);
                let _ = app_handle.emit("clipboard-update", ());
            }
            Ok(_) => {}
            Err(e) => log::error!("Retention failed: {}", e),
        }
    });
}

// TODO: Fix auto-updater API for Tauri v2
// async fn check_for_updates_on_startup(app_handle: tauri::AppHandle) {
//     use tauri_plugin_updater::UpdaterExt;
//     
//     log::info!("Checking for updates...");
//     
//     match app_handle.updater().check().await {
//         Ok(Some(update)) => {
//             log::info!("Update available: {}", update.version);
//             
//             // Emit event to frontend to show update notification
//             let _ = app_handle.emit("update-available", &update.version);
//         }
//         Ok(None) => {
//             log::info!("App is up to date");
//         }
//         Err(e) => {
//             log::warn!("Failed to check for updates: {}", e);
//         }
//     }
// }
