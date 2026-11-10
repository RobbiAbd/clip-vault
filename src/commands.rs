use anyhow::Result;
use std::sync::Mutex;
use tauri::{AppHandle, Emitter, State};
use tauri_plugin_autostart::ManagerExt;

use crate::models::ClipboardEntryPreview;
use crate::settings::{Settings, SettingsStore};
use crate::storage::DataStore;

pub struct AppState {
    pub data_store: Mutex<DataStore>,
    pub settings: Mutex<Settings>,
    pub settings_store: SettingsStore,
}

#[tauri::command]
pub fn get_settings(state: State<AppState>) -> Settings {
    state.settings.lock().unwrap().clone()
}

#[tauri::command]
pub fn save_settings(
    app: AppHandle,
    state: State<AppState>,
    settings: Settings,
) -> Result<Settings, String> {
    let autolaunch = app.autolaunch();
    let currently_enabled = autolaunch.is_enabled().unwrap_or(false);
    if settings.launch_at_startup != currently_enabled {
        let result = if settings.launch_at_startup {
            autolaunch.enable()
        } else {
            autolaunch.disable()
        };
        result.map_err(|e| format!("Failed to update launch at startup: {}", e))?;
    }

    state
        .settings_store
        .save(&settings)
        .map_err(|e| format!("Failed to save settings: {}", e))?;
    *state.settings.lock().unwrap() = settings.clone();

    let removed = state
        .data_store
        .lock()
        .unwrap()
        .apply_retention(settings.auto_delete_days, settings.max_history)
        .map_err(|e| format!("Failed to apply retention: {}", e))?;
    if removed > 0 {
        let _ = app.emit("clipboard-update", ());
    }

    Ok(settings)
}

#[tauri::command]
pub fn get_history(
    state: State<AppState>,
    limit: Option<usize>,
    offset: Option<usize>,
) -> Result<Vec<ClipboardEntryPreview>, String> {
    let store = state.data_store.lock().unwrap();
    
    let limit = limit.unwrap_or(50);
    let offset = offset.unwrap_or(0);
    
    store
        .get_history(limit, offset)
        .map_err(|e| format!("Failed to get history: {}", e))
}

#[tauri::command]
pub fn get_entry(state: State<AppState>, id: i64) -> Result<Option<String>, String> {
    let store = state.data_store.lock().unwrap();
    
    match store.get_entry(id) {
        Ok(Some(entry)) => {
            match entry.content_type {
                crate::models::ContentType::Text | crate::models::ContentType::RichText => {
                    Ok(entry.content_text)
                }
                crate::models::ContentType::Image => {
                    if let Some(blob) = entry.content_blob {
                        let base64 = base64_encode(&blob);
                        Ok(Some(format!("data:image/png;base64,{}", base64)))
                    } else {
                        Ok(None)
                    }
                }
                crate::models::ContentType::Files => Ok(entry.content_text),
            }
        }
        Ok(None) => Ok(None),
        Err(e) => Err(format!("Failed to get entry: {}", e)),
    }
}

#[tauri::command]
pub fn search_history(
    state: State<AppState>,
    query: String,
    limit: Option<usize>,
) -> Result<Vec<ClipboardEntryPreview>, String> {
    let store = state.data_store.lock().unwrap();
    
    let limit = limit.unwrap_or(50);
    
    store
        .search(&query, limit)
        .map_err(|e| format!("Failed to search: {}", e))
}

#[tauri::command]
pub fn delete_entry(state: State<AppState>, id: i64) -> Result<(), String> {
    let store = state.data_store.lock().unwrap();
    
    store
        .delete_entry(id)
        .map_err(|e| format!("Failed to delete entry: {}", e))
}

#[tauri::command]
pub fn toggle_pin(state: State<AppState>, id: i64) -> Result<bool, String> {
    let store = state.data_store.lock().unwrap();
    
    store
        .toggle_pin(id)
        .map_err(|e| format!("Failed to toggle pin: {}", e))
}

#[tauri::command]
pub fn clear_history(state: State<AppState>, keep_pinned: bool) -> Result<usize, String> {
    let store = state.data_store.lock().unwrap();
    
    store
        .clear_history(keep_pinned)
        .map_err(|e| format!("Failed to clear history: {}", e))
}

#[tauri::command]
pub fn get_stats(state: State<AppState>) -> Result<Stats, String> {
    let store = state.data_store.lock().unwrap();
    
    let total_count = store
        .get_count()
        .map_err(|e| format!("Failed to get count: {}", e))?;

    Ok(Stats {
        total_clips: total_count,
    })
}

const CLIPBOARD_ATTEMPTS: u32 = 10;
const CLIPBOARD_RETRY_DELAY: std::time::Duration = std::time::Duration::from_millis(50);

/// Other processes can still hold the clipboard briefly, so writes are retried.
fn with_clipboard<T>(
    mut op: impl FnMut(&mut arboard::Clipboard) -> Result<T, arboard::Error>,
) -> Result<T, String> {
    let _guard = crate::clipboard::CLIPBOARD_LOCK.lock().unwrap();
    let mut last_error = String::new();

    for attempt in 1..=CLIPBOARD_ATTEMPTS {
        match arboard::Clipboard::new().and_then(|mut clipboard| op(&mut clipboard)) {
            Ok(value) => return Ok(value),
            Err(e) => {
                log::warn!("Clipboard write attempt {} failed: {}", attempt, e);
                last_error = e.to_string();
                std::thread::sleep(CLIPBOARD_RETRY_DELAY);
            }
        }
    }

    Err(last_error)
}

#[tauri::command]
pub fn copy_to_clipboard(content: String) -> Result<(), String> {
    with_clipboard(|clipboard| clipboard.set_text(content.as_str()))
        .map_err(|e| format!("Failed to copy: {}", e))
}

#[tauri::command]
pub fn copy_image_to_clipboard(base64_data: String) -> Result<(), String> {
    use arboard::ImageData;
    
    // Remove data URI prefix if present
    let base64_str = base64_data
        .strip_prefix("data:image/png;base64,")
        .unwrap_or(&base64_data);
    
    // Decode base64 to PNG bytes
    let png_bytes = base64_decode(base64_str)
        .map_err(|e| format!("Failed to decode base64: {}", e))?;
    
    // Decode PNG to RGBA
    let decoder = png::Decoder::new(&png_bytes[..]);
    let mut reader = decoder.read_info()
        .map_err(|e| format!("Failed to read PNG: {}", e))?;
    
    let mut rgba_data = vec![0; reader.output_buffer_size()];
    let info = reader.next_frame(&mut rgba_data)
        .map_err(|e| format!("Failed to decode PNG frame: {}", e))?;
    
    let width = info.width as usize;
    let height = info.height as usize;
    
    // Create ImageData for arboard
    let img_data = ImageData {
        width,
        height,
        bytes: rgba_data.into(),
    };
    
    with_clipboard(|clipboard| clipboard.set_image(img_data.clone()))
        .map_err(|e| format!("Failed to copy image: {}", e))?;
    
    log::info!("✅ Image copied to clipboard ({}x{})", width, height);
    Ok(())
}

#[derive(serde::Serialize)]
pub struct Stats {
    pub total_clips: i64,
}

fn base64_encode(data: &[u8]) -> String {
    use std::io::Write;
    let mut buf = Vec::new();
    {
        let mut encoder = base64::write::EncoderWriter::new(&mut buf, &base64::engine::general_purpose::STANDARD);
        encoder.write_all(data).unwrap();
    }
    String::from_utf8(buf).unwrap()
}

fn base64_decode(data: &str) -> Result<Vec<u8>, String> {
    use base64::Engine;
    base64::engine::general_purpose::STANDARD
        .decode(data)
        .map_err(|e| format!("Base64 decode error: {}", e))
}

// TODO: Fix auto-updater commands for Tauri v2
// Auto-update commands  
// #[tauri::command]
// pub async fn check_update(app_handle: tauri::AppHandle) -> Result<UpdateInfo, String> {
//     use tauri_plugin_updater::UpdaterExt;
//     
//     match app_handle.updater().check().await {
//         Ok(Some(update)) => {
//             Ok(UpdateInfo {
//                 available: true,
//                 current_version: update.current_version,
//                 latest_version: update.version.clone(),
//                 date: update.date.clone(),
//                 body: update.body.clone(),
//             })
//         }
//         Ok(None) => {
//             let version = app_handle.package_info().version.to_string();
//             Ok(UpdateInfo {
//                 available: false,
//                 current_version: version.clone(),
//                 latest_version: version,
//                 date: None,
//                 body: None,
//             })
//         }
//         Err(e) => Err(format!("Failed to check for updates: {}", e)),
//     }
// }
// 
// #[tauri::command]
// pub async fn install_update(app_handle: tauri::AppHandle) -> Result<(), String> {
//     use tauri_plugin_updater::UpdaterExt;
//     
//     match app_handle.updater().check().await {
//         Ok(Some(update)) => {
//             log::info!("Installing update: {}", update.version);
//             
//             let mut downloaded = 0;
//             update
//                 .download_and_install(
//                     |chunk_length, content_length| {
//                         downloaded += chunk_length;
//                         log::debug!("Downloaded {} of {:?} bytes", downloaded, content_length);
//                     },
//                     || {
//                         log::info!("Download finished, installing...");
//                     }
//                 )
//                 .await
//                 .map_err(|e| format!("Failed to install update: {}", e))?;
//             
//             Ok(())
//         }
//         Ok(None) => {
//             Err("No update available".to_string())
//         }
//         Err(e) => Err(format!("Failed to check for updates: {}", e)),
//     }
// }
// 
// #[derive(serde::Serialize)]
// pub struct UpdateInfo {
//     pub available: bool,
//     pub current_version: String,
//     pub latest_version: String,
//     pub date: Option<String>,
//     pub body: Option<String>,
// }
