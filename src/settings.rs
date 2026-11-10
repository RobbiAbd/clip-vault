use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Theme {
    System,
    Light,
    Dark,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Language {
    System,
    En,
    Id,
}

impl Language {
    /// Resolves `System` against the OS locale; anything other than Indonesian falls back to English.
    pub fn resolve(self) -> Language {
        match self {
            Language::System => match sys_locale::get_locale() {
                Some(locale) if locale.to_lowercase().starts_with("id") => Language::Id,
                _ => Language::En,
            },
            explicit => explicit,
        }
    }
}

pub const DEFAULT_SHORTCUT: &str = "CommandOrControl+Alt+V";

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct Settings {
    pub launch_at_startup: bool,
    pub start_minimized: bool,
    pub close_to_tray: bool,
    /// 0 disables age-based cleanup.
    pub auto_delete_days: u32,
    /// 0 means unlimited.
    pub max_history: u32,
    pub capture_images: bool,
    pub theme: Theme,
    pub check_updates_on_startup: bool,
    pub global_shortcut_enabled: bool,
    /// Accelerator string understood by tauri-plugin-global-shortcut, e.g. `CommandOrControl+Alt+V`.
    pub global_shortcut: String,
    pub language: Language,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            launch_at_startup: false,
            start_minimized: false,
            close_to_tray: true,
            auto_delete_days: 0,
            max_history: 1000,
            capture_images: true,
            theme: Theme::System,
            check_updates_on_startup: true,
            global_shortcut_enabled: true,
            global_shortcut: DEFAULT_SHORTCUT.to_string(),
            language: Language::System,
        }
    }
}

pub struct SettingsStore {
    path: PathBuf,
}

impl SettingsStore {
    pub fn new(path: PathBuf) -> Self {
        Self { path }
    }

    /// Missing or unreadable files fall back to defaults so a corrupt file never blocks startup.
    pub fn load(&self) -> Settings {
        match std::fs::read_to_string(&self.path) {
            Ok(raw) => serde_json::from_str(&raw).unwrap_or_else(|e| {
                log::warn!("Invalid settings file, using defaults: {}", e);
                Settings::default()
            }),
            Err(_) => Settings::default(),
        }
    }

    pub fn save(&self, settings: &Settings) -> Result<()> {
        if let Some(dir) = self.path.parent() {
            std::fs::create_dir_all(dir)?;
        }
        let raw = serde_json::to_string_pretty(settings)?;
        write_atomic(&self.path, &raw).context("Failed to write settings")
    }
}

fn write_atomic(path: &Path, contents: &str) -> std::io::Result<()> {
    let tmp = path.with_extension("json.tmp");
    std::fs::write(&tmp, contents)?;
    std::fs::rename(&tmp, path)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn store_in(dir: &TempDir) -> SettingsStore {
        SettingsStore::new(dir.path().join("settings.json"))
    }

    #[test]
    fn test_load_missing_file_returns_defaults() {
        let dir = TempDir::new().unwrap();
        assert_eq!(store_in(&dir).load(), Settings::default());
    }

    #[test]
    fn test_save_then_load_roundtrip() {
        let dir = TempDir::new().unwrap();
        let store = store_in(&dir);
        let settings = Settings {
            launch_at_startup: true,
            start_minimized: true,
            close_to_tray: false,
            auto_delete_days: 7,
            max_history: 500,
            capture_images: false,
            theme: Theme::Dark,
            check_updates_on_startup: false,
            global_shortcut_enabled: false,
            global_shortcut: "Alt+Shift+V".to_string(),
            language: Language::Id,
        };

        store.save(&settings).unwrap();
        assert_eq!(store.load(), settings);
    }

    #[test]
    fn test_corrupt_file_returns_defaults() {
        let dir = TempDir::new().unwrap();
        let store = store_in(&dir);
        std::fs::write(dir.path().join("settings.json"), "{not json").unwrap();
        assert_eq!(store.load(), Settings::default());
    }

    #[test]
    fn test_partial_file_fills_missing_fields_with_defaults() {
        let dir = TempDir::new().unwrap();
        let store = store_in(&dir);
        std::fs::write(dir.path().join("settings.json"), r#"{"theme":"light"}"#).unwrap();

        let loaded = store.load();
        assert_eq!(loaded.theme, Theme::Light);
        assert_eq!(loaded.max_history, Settings::default().max_history);
        assert!(loaded.close_to_tray);
    }
}
