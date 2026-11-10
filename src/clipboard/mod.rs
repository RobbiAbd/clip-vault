use anyhow::Result;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use crate::models::ClipboardEntry;

mod generic_monitor;

/// Windows lets only one thread hold the clipboard open. Without this, the monitor's polling
/// can open it mid-write and the write fails with ERROR_CLIPBOARD_NOT_OPEN (1418).
pub static CLIPBOARD_LOCK: Mutex<()> = Mutex::new(());

pub trait ClipboardMonitor: Send {
    fn start(&mut self) -> Result<()>;
    fn stop(&mut self);
    fn get_latest_entry(&self) -> Option<ClipboardEntry>;
}

pub struct ClipboardManager {
    monitor: generic_monitor::GenericClipboardMonitor,
    last_captured: Arc<Mutex<Option<Instant>>>,
    debounce_duration: Duration,
}

impl ClipboardManager {
    pub fn new() -> Result<Self> {
        let monitor = generic_monitor::GenericClipboardMonitor::new()?;

        Ok(Self {
            monitor,
            last_captured: Arc::new(Mutex::new(None)),
            debounce_duration: Duration::from_millis(500),
        })
    }

    pub fn start(&mut self) -> Result<()> {
        self.monitor.start()
    }

    pub fn stop(&mut self) {
        self.monitor.stop()
    }

    pub fn get_latest_entry(&self) -> Option<ClipboardEntry> {
        let now = Instant::now();
        let mut last = self.last_captured.lock().unwrap();
        
        if let Some(last_time) = *last {
            if now.duration_since(last_time) < self.debounce_duration {
                return None;
            }
        }
        
        *last = Some(now);
        self.monitor.get_latest_entry()
    }

    pub fn should_capture(&self) -> bool {
        let last = self.last_captured.lock().unwrap();
        match *last {
            Some(last_time) => Instant::now().duration_since(last_time) >= self.debounce_duration,
            None => true,
        }
    }
}
