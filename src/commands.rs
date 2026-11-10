use anyhow::Result;
use std::sync::Mutex;
use tauri::State;

use crate::models::ClipboardEntryPreview;
use crate::storage::DataStore;

pub struct AppState {
    pub data_store: Mutex<DataStore>,
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

#[tauri::command]
pub fn copy_to_clipboard(content: String) -> Result<(), String> {
    use arboard::Clipboard;
    let mut clipboard = Clipboard::new().map_err(|e| format!("Failed to access clipboard: {}", e))?;
    clipboard.set_text(&content).map_err(|e| format!("Failed to copy: {}", e))
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
