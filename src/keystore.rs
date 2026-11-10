use anyhow::{anyhow, Context, Result};
use base64::{engine::general_purpose::STANDARD, Engine};
use std::path::Path;

use crate::crypto::{generate_key, KEY_LEN};

/// Loads the history encryption key, creating one on first run. On Windows and macOS the key
/// lives in the OS credential store, so a copied database file cannot be read elsewhere.
pub fn load_or_create_key(app_dir: &Path) -> Result<[u8; KEY_LEN]> {
    imp::load_or_create(app_dir)
}

fn decode(raw: &str) -> Result<[u8; KEY_LEN]> {
    let bytes = STANDARD
        .decode(raw.trim())
        .context("Stored history key is not valid base64")?;
    bytes
        .try_into()
        .map_err(|_| anyhow!("Stored history key has the wrong length"))
}

#[cfg(any(windows, target_os = "macos"))]
mod imp {
    use super::*;

    const SERVICE: &str = "com.clipvault.app";
    const ACCOUNT: &str = "history-encryption-key";

    pub fn load_or_create(_app_dir: &Path) -> Result<[u8; KEY_LEN]> {
        let entry = keyring::Entry::new(SERVICE, ACCOUNT).context("Failed to open the credential store")?;
        match entry.get_password() {
            Ok(raw) => decode(&raw),
            // Only a confirmed-missing key may be replaced; any other error must not
            // overwrite a key that existing clips depend on.
            Err(keyring::Error::NoEntry) => {
                let key = generate_key();
                entry
                    .set_password(&STANDARD.encode(key))
                    .context("Failed to save the history key to the credential store")?;
                log::info!("Created a new history encryption key");
                Ok(key)
            }
            Err(e) => Err(e).context("Failed to read the history key from the credential store"),
        }
    }
}

#[cfg(not(any(windows, target_os = "macos")))]
mod imp {
    use super::*;
    use std::os::unix::fs::OpenOptionsExt;

    const KEY_FILE: &str = "history.key";

    pub fn load_or_create(app_dir: &Path) -> Result<[u8; KEY_LEN]> {
        let path = app_dir.join(KEY_FILE);
        match std::fs::read_to_string(&path) {
            Ok(raw) => decode(&raw),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
                use std::io::Write;
                let key = generate_key();
                let mut file = std::fs::OpenOptions::new()
                    .write(true)
                    .create_new(true)
                    .mode(0o600)
                    .open(&path)
                    .context("Failed to create the history key file")?;
                file.write_all(STANDARD.encode(key).as_bytes())?;
                Ok(key)
            }
            Err(e) => Err(e).context("Failed to read the history key file"),
        }
    }
}
