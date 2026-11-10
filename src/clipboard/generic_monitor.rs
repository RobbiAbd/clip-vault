use anyhow::Result;
use arboard::Clipboard;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

use crate::clipboard::ClipboardMonitor;
use crate::models::ClipboardEntry;

pub struct GenericClipboardMonitor {
    running: Arc<Mutex<bool>>,
    latest_entry: Arc<Mutex<Option<ClipboardEntry>>>,
    last_content: Arc<Mutex<String>>,
}

impl GenericClipboardMonitor {
    pub fn new() -> Result<Self> {
        Ok(Self {
            running: Arc::new(Mutex::new(false)),
            latest_entry: Arc::new(Mutex::new(None)),
            last_content: Arc::new(Mutex::new(String::new())),
        })
    }
}

impl ClipboardMonitor for GenericClipboardMonitor {
    fn start(&mut self) -> Result<()> {
        let running = self.running.clone();
        let latest_entry = self.latest_entry.clone();
        let last_content = self.last_content.clone();

        *running.lock().unwrap() = true;

        thread::spawn(move || {
            log::info!("Generic clipboard monitor started");
            
            let mut clipboard = Clipboard::new().expect("Failed to access clipboard");

            while *running.lock().unwrap() {
                if let Ok(text) = clipboard.get_text() {
                    let mut last = last_content.lock().unwrap();
                    
                    if text != *last && !text.is_empty() {
                        *last = text.clone();
                        drop(last);

                        let entry = ClipboardEntry::new_text(text, None);
                        *latest_entry.lock().unwrap() = Some(entry);
                        log::debug!("Captured text from clipboard");
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
