use anyhow::{Context, Result};
use clipboard_win::{formats, get_clipboard, get_clipboard_string};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

use crate::clipboard::ClipboardMonitor;
use crate::models::ClipboardEntry;

pub struct WindowsClipboardMonitor {
    running: Arc<Mutex<bool>>,
    latest_entry: Arc<Mutex<Option<ClipboardEntry>>>,
    last_sequence: Arc<Mutex<u32>>,
}

impl WindowsClipboardMonitor {
    pub fn new() -> Result<Self> {
        Ok(Self {
            running: Arc::new(Mutex::new(false)),
            latest_entry: Arc::new(Mutex::new(None)),
            last_sequence: Arc::new(Mutex::new(0)),
        })
    }

    fn get_clipboard_sequence_number() -> u32 {
        use windows::Win32::System::DataExchange::GetClipboardSequenceNumber;
        unsafe { GetClipboardSequenceNumber() }
    }

    fn capture_clipboard_text() -> Option<String> {
        get_clipboard_string().ok()
    }

    fn capture_clipboard_image() -> Option<Vec<u8>> {
        match get_clipboard::<Vec<u8>, _>(formats::Bitmap) {
            Ok(data) => Some(data),
            Err(_) => None,
        }
    }
}

impl ClipboardMonitor for WindowsClipboardMonitor {
    fn start(&mut self) -> Result<()> {
        let running = self.running.clone();
        let latest_entry = self.latest_entry.clone();
        let last_sequence = self.last_sequence.clone();

        *running.lock().unwrap() = true;

        thread::spawn(move || {
            log::info!("Clipboard monitor started");
            
            while *running.lock().unwrap() {
                let current_sequence = Self::get_clipboard_sequence_number();
                let mut last_seq = last_sequence.lock().unwrap();

                if current_sequence != *last_seq && current_sequence > 0 {
                    *last_seq = current_sequence;
                    drop(last_seq);

                    if let Some(text) = Self::capture_clipboard_text() {
                        if !text.is_empty() {
                            let entry = ClipboardEntry::new_text(text, None);
                            *latest_entry.lock().unwrap() = Some(entry);
                            log::debug!("Captured text from clipboard");
                        }
                    } else if let Some(image_data) = Self::capture_clipboard_image() {
                        let entry = ClipboardEntry::new_image(image_data, None);
                        *latest_entry.lock().unwrap() = Some(entry);
                        log::debug!("Captured image from clipboard");
                    }
                }

                thread::sleep(Duration::from_millis(100));
            }

            log::info!("Clipboard monitor stopped");
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
