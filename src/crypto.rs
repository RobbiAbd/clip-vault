use aes_gcm::aead::{Aead, AeadCore, KeyInit, OsRng};
use aes_gcm::{Aes256Gcm, Key, Nonce};
use anyhow::{anyhow, bail, Result};
use hmac::{Hmac, Mac};
use sha2::Sha256;

pub const KEY_LEN: usize = 32;
const NONCE_LEN: usize = 12;
const TAG_LEN: usize = 16;
/// Bytes added to every encrypted value (nonce prefix + authentication tag).
pub const OVERHEAD: usize = NONCE_LEN + TAG_LEN;

type HmacSha256 = Hmac<Sha256>;

pub fn generate_key() -> [u8; KEY_LEN] {
    Aes256Gcm::generate_key(OsRng).into()
}

pub struct Cipher {
    aead: Aes256Gcm,
    fingerprint_key: [u8; 32],
}

impl Cipher {
    pub fn new(key: &[u8; KEY_LEN]) -> Self {
        let mut mac = <HmacSha256 as Mac>::new_from_slice(key).expect("HMAC accepts keys of any length");
        mac.update(b"clip-vault/fingerprint");

        Self {
            aead: Aes256Gcm::new(Key::<Aes256Gcm>::from_slice(key)),
            fingerprint_key: mac.finalize().into_bytes().into(),
        }
    }

    /// Output layout: `nonce || ciphertext || tag`.
    pub fn encrypt(&self, plaintext: &[u8]) -> Vec<u8> {
        let nonce = Aes256Gcm::generate_nonce(&mut OsRng);
        let ciphertext = self
            .aead
            .encrypt(&nonce, plaintext)
            .expect("AES-GCM encryption of an in-memory buffer cannot fail");

        let mut out = Vec::with_capacity(NONCE_LEN + ciphertext.len());
        out.extend_from_slice(&nonce);
        out.extend_from_slice(&ciphertext);
        out
    }

    pub fn decrypt(&self, data: &[u8]) -> Result<Vec<u8>> {
        if data.len() < OVERHEAD {
            bail!("Encrypted value is too short");
        }
        let (nonce, ciphertext) = data.split_at(NONCE_LEN);
        self.aead
            .decrypt(Nonce::from_slice(nonce), ciphertext)
            .map_err(|_| anyhow!("Failed to decrypt value (wrong key or corrupted data)"))
    }

    /// Keyed hash used to find identical clips without decrypting every row. It is keyed so the
    /// stored hash cannot be brute-forced back into short texts like PINs.
    pub fn fingerprint(&self, content_type: &str, text: Option<&str>, blob: Option<&[u8]>) -> Vec<u8> {
        let mut mac =
            <HmacSha256 as Mac>::new_from_slice(&self.fingerprint_key).expect("HMAC accepts keys of any length");
        mac.update(content_type.as_bytes());
        for part in [text.map(str::as_bytes), blob] {
            match part {
                Some(bytes) => {
                    mac.update(&[1]);
                    mac.update(&(bytes.len() as u64).to_le_bytes());
                    mac.update(bytes);
                }
                None => mac.update(&[0]),
            }
        }
        mac.finalize().into_bytes().to_vec()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encrypt_roundtrip_and_random_nonce() {
        let cipher = Cipher::new(&[1u8; KEY_LEN]);
        let a = cipher.encrypt(b"hello");
        let b = cipher.encrypt(b"hello");

        assert_ne!(a, b);
        assert_eq!(a.len(), 5 + OVERHEAD);
        assert_eq!(cipher.decrypt(&a).unwrap(), b"hello");
    }

    #[test]
    fn test_decrypt_with_wrong_key_fails() {
        let encrypted = Cipher::new(&[1u8; KEY_LEN]).encrypt(b"secret");
        assert!(Cipher::new(&[2u8; KEY_LEN]).decrypt(&encrypted).is_err());
    }

    #[test]
    fn test_fingerprint_distinguishes_missing_and_empty_text() {
        let cipher = Cipher::new(&[1u8; KEY_LEN]);
        assert_eq!(cipher.fingerprint("text", Some("a"), None), cipher.fingerprint("text", Some("a"), None));
        assert_ne!(cipher.fingerprint("text", Some(""), None), cipher.fingerprint("text", None, None));
        assert_ne!(cipher.fingerprint("text", Some("a"), None), cipher.fingerprint("richtext", Some("a"), None));
    }
}
