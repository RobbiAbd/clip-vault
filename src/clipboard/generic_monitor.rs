use anyhow::Result;
use arboard::Clipboard;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

use crate::clipboard::{ClipboardMonitor, CLIPBOARD_LOCK};
use crate::models::ClipboardEntry;

pub struct GenericClipboardMonitor {
    running: Arc<Mutex<bool>>,
    latest_entry: Arc<Mutex<Option<ClipboardEntry>>>,
    last_content: Arc<Mutex<String>>,
    last_image_hash: Arc<Mutex<Option<u64>>>,
}

impl GenericClipboardMonitor {
    pub fn new() -> Result<Self> {
        Ok(Self {
            running: Arc::new(Mutex::new(false)),
            latest_entry: Arc::new(Mutex::new(None)),
            last_content: Arc::new(Mutex::new(String::new())),
            last_image_hash: Arc::new(Mutex::new(None)),
        })
    }
}

impl ClipboardMonitor for GenericClipboardMonitor {
    fn start(&mut self) -> Result<()> {
        let running = self.running.clone();
        let latest_entry = self.latest_entry.clone();
        let last_content = self.last_content.clone();
        let last_image_hash = self.last_image_hash.clone();

        *running.lock().unwrap() = true;

        thread::spawn(move || {
            log::info!("Generic clipboard monitor started (with image support)");
            
            let mut clipboard = Clipboard::new().expect("Failed to access clipboard");

            while *running.lock().unwrap() {
                let (image, text) = {
                    let _guard = CLIPBOARD_LOCK.lock().unwrap();
                    match clipboard.get_image() {
                        Ok(image) => (Some(image), None),
                        Err(_) => (None, clipboard.get_text().ok()),
                    }
                };

                // Try to get image first (higher priority)
                if let Some(image) = image {
                    let width = image.width;
                    let height = image.height;
                    let rgba_data = image.bytes.into_owned();

                    // Hash raw pixels so an unchanged image isn't re-encoded on every poll
                    let hash = calculate_hash(&rgba_data);
                    let mut last_hash = last_image_hash.lock().unwrap();
                    
                    if *last_hash != Some(hash) {
                        *last_hash = Some(hash);
                        drop(last_hash);

                        let mut png_data = Vec::new();
                        {
                            let mut encoder = png::Encoder::new(&mut png_data, width as u32, height as u32);
                            encoder.set_color(png::ColorType::Rgba);
                            encoder.set_depth(png::BitDepth::Eight);
                            let mut writer = encoder.write_header().expect("Failed to write PNG header");
                            writer.write_image_data(&rgba_data).expect("Failed to write PNG data");
                        }
                        
                        let entry = ClipboardEntry::new_image(png_data, None);
                        *latest_entry.lock().unwrap() = Some(entry);
                        log::info!("📷 Captured image from clipboard ({}x{})", width, height);
                    }
                } 
                // If no image, try text
                else if let Some(text) = text {
                    let mut last = last_content.lock().unwrap();
                    
                    if text != *last && !text.is_empty() {
                        *last = text.clone();
                        drop(last);

                        let entry = ClipboardEntry::new_text(text, None);
                        *latest_entry.lock().unwrap() = Some(entry);
                        log::debug!("📝 Captured text from clipboard");
                    }
                }

                thread::sleep(Duration::from_millis(250));
            }

            log::info!("Generic clipboard monitor stopped");
        });

        Ok(())
    }

    fn stop(&mut self) {
        *self.running.lock().unwrap() = false;
    }

    fn get_latest_entry(&self) -> Option<ClipboardEntry> {
        self.latest_entry.lock().unwrap().take()
    }
}

fn calculate_hash(data: &[u8]) -> u64 {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};
    
    let mut hasher = DefaultHasher::new();
    data.hash(&mut hasher);
    hasher.finish()
}
