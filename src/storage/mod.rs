use anyhow::{Context, Result};
use base64::{engine::general_purpose::STANDARD, Engine};
use chrono::{DateTime, Utc};
use rusqlite::{params, Connection, OptionalExtension};
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

use crate::crypto::{Cipher, KEY_LEN, OVERHEAD};
use crate::models::{preview_text, ClipboardEntry, ClipboardEntryPreview, ContentType};
use crate::thumbnail::make_thumbnail;

pub const THUMBNAIL_MAX_SIDE: u32 = 480;

/// `content_text`, `content_blob` and `thumbnail` hold AES-256-GCM ciphertext once `encrypted = 1`.
const PREVIEW_COLUMNS: &str =
    "id, content_type, content_text, length(content_blob), timestamp, app_source, pinned, thumbnail";

const KEY_CHECK_PLAINTEXT: &[u8] = b"clip-vault-key-check";

fn parse_timestamp(raw: &str) -> DateTime<Utc> {
    DateTime::parse_from_rfc3339(raw)
        .map(|dt| dt.with_timezone(&Utc))
        .unwrap_or_else(|_| Utc::now())
}

fn add_column_if_missing(conn: &Connection, column: &str, definition: &str) -> Result<()> {
    if conn.prepare(&format!("SELECT {column} FROM clips LIMIT 0")).is_err() {
        conn.execute(&format!("ALTER TABLE clips ADD COLUMN {column} {definition}"), [])?;
    }
    Ok(())
}

pub struct DataStore {
    conn: Arc<Mutex<Connection>>,
    cipher: Cipher,
}

impl DataStore {
    pub fn new(db_path: PathBuf, key: &[u8; KEY_LEN]) -> Result<Self> {
        let conn = Connection::open(db_path).context("Failed to open database")?;
        // Zero deleted content so removed clips (and pre-encryption plaintext) don't linger in free pages.
        conn.pragma_update(None, "secure_delete", "ON")?;

        let store = DataStore {
            conn: Arc::new(Mutex::new(conn)),
            cipher: Cipher::new(key),
        };

        store.init_schema()?;
        store.verify_key()?;
        store.encrypt_legacy_rows()?;
        Ok(store)
    }

    fn init_schema(&self) -> Result<()> {
        let conn = self.conn.lock().unwrap();

        conn.execute(
            "CREATE TABLE IF NOT EXISTS clips (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                content_type TEXT NOT NULL,
                content_text BLOB,
                content_blob BLOB,
                timestamp TEXT NOT NULL,
                app_source TEXT,
                pinned INTEGER NOT NULL DEFAULT 0,
                thumbnail BLOB,
                content_hash BLOB,
                encrypted INTEGER NOT NULL DEFAULT 0
            )",
            [],
        )?;

        add_column_if_missing(&conn, "thumbnail", "BLOB")?;
        add_column_if_missing(&conn, "content_hash", "BLOB")?;
        add_column_if_missing(&conn, "encrypted", "INTEGER NOT NULL DEFAULT 0")?;

        conn.execute_batch(
            "CREATE INDEX IF NOT EXISTS idx_timestamp ON clips(timestamp DESC);
             CREATE INDEX IF NOT EXISTS idx_pinned ON clips(pinned DESC, timestamp DESC);
             CREATE INDEX IF NOT EXISTS idx_content_hash ON clips(content_hash);
             CREATE TABLE IF NOT EXISTS meta (key TEXT PRIMARY KEY, value BLOB);",
        )?;

        Ok(())
    }

    /// If the key changed (e.g. the stored credential was deleted), clips encrypted with the old
    /// key can never be read again, so they are dropped instead of showing as garbage.
    fn verify_key(&self) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        let check: Option<Vec<u8>> = conn
            .query_row("SELECT value FROM meta WHERE key = 'key_check'", [], |row| row.get(0))
            .optional()?;

        match check {
            Some(check) if self.cipher.decrypt(&check).is_ok() => return Ok(()),
            Some(_) => {
                let removed = conn.execute("DELETE FROM clips WHERE encrypted = 1", [])?;
                log::warn!("History key changed; removed {} clips that can no longer be decrypted", removed);
            }
            None => {}
        }

        conn.execute(
            "INSERT OR REPLACE INTO meta (key, value) VALUES ('key_check', ?1)",
            params![self.cipher.encrypt(KEY_CHECK_PLAINTEXT)],
        )?;
        Ok(())
    }

    /// Encrypts rows saved by versions before encryption existed.
    fn encrypt_legacy_rows(&self) -> Result<()> {
        let mut conn = self.conn.lock().unwrap();
        let tx = conn.transaction()?;

        type LegacyRow = (i64, String, Option<String>, Option<Vec<u8>>, Option<Vec<u8>>);
        let rows: Vec<LegacyRow> = {
            let mut stmt = tx.prepare(
                "SELECT id, content_type, content_text, content_blob, thumbnail FROM clips WHERE encrypted = 0",
            )?;
            let rows = stmt
                .query_map([], |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?, row.get(4)?)))?
                .collect::<Result<_, _>>()?;
            rows
        };
        if rows.is_empty() {
            return Ok(());
        }

        for (id, content_type, text, blob, thumbnail) in &rows {
            let thumbnail = match thumbnail {
                None if content_type == "image" => {
                    blob.as_deref().and_then(|b| make_thumbnail(b, THUMBNAIL_MAX_SIDE))
                }
                other => other.clone(),
            };
            tx.execute(
                "UPDATE clips
                 SET content_text = ?1, content_blob = ?2, thumbnail = ?3, content_hash = ?4, encrypted = 1
                 WHERE id = ?5",
                params![
                    self.seal(text.as_deref().map(str::as_bytes)),
                    self.seal(blob.as_deref()),
                    self.seal(thumbnail.as_deref()),
                    self.cipher.fingerprint(content_type, text.as_deref(), blob.as_deref()),
                    id
                ],
            )?;
        }
        tx.commit()?;

        // Rebuild the file so the old plaintext pages are physically gone.
        conn.execute_batch("VACUUM")?;
        log::info!("Encrypted {} existing clips", rows.len());
        Ok(())
    }

    fn seal(&self, data: Option<&[u8]>) -> Option<Vec<u8>> {
        data.map(|d| self.cipher.encrypt(d))
    }

    fn open(&self, data: Option<Vec<u8>>) -> Option<Vec<u8>> {
        let data = data?;
        match self.cipher.decrypt(&data) {
            Ok(plain) => Some(plain),
            Err(e) => {
                log::error!("{}", e);
                None
            }
        }
    }

    fn open_text(&self, data: Option<Vec<u8>>) -> Option<String> {
        self.open(data).map(|bytes| String::from_utf8_lossy(&bytes).into_owned())
    }

    /// Returns the preview together with the full decrypted text, which search needs.
    fn row_to_preview(&self, row: &rusqlite::Row) -> rusqlite::Result<(ClipboardEntryPreview, Option<String>)> {
        let content_type = ContentType::from_string(&row.get::<_, String>(1)?).unwrap_or(ContentType::Text);
        let text = self.open_text(row.get(2)?);
        let blob_len = (row.get::<_, Option<i64>>(3)?.unwrap_or(0) as usize).saturating_sub(OVERHEAD);
        let thumbnail = self.open(row.get(7)?);

        let preview = ClipboardEntryPreview {
            id: row.get(0)?,
            preview: preview_text(&content_type, text.as_deref(), blob_len),
            content_type,
            timestamp: parse_timestamp(&row.get::<_, String>(4)?),
            app_source: row.get(5)?,
            pinned: row.get::<_, i32>(6)? != 0,
            thumbnail: thumbnail.map(|t| format!("data:image/png;base64,{}", STANDARD.encode(t))),
        };
        Ok((preview, text))
    }

    /// Saving content identical to an existing clip moves that clip to the top instead of
    /// adding a duplicate; a pinned match keeps its pin.
    pub fn save_entry(&self, entry: &ClipboardEntry) -> Result<i64> {
        let thumbnail = match entry.content_type {
            ContentType::Image => entry
                .content_blob
                .as_deref()
                .and_then(|blob| make_thumbnail(blob, THUMBNAIL_MAX_SIDE)),
            _ => None,
        };

        let content_type = entry.content_type.to_string();
        let timestamp = entry.timestamp.to_rfc3339();
        let hash = self.cipher.fingerprint(
            &content_type,
            entry.content_text.as_deref(),
            entry.content_blob.as_deref(),
        );
        let text = self.seal(entry.content_text.as_deref().map(str::as_bytes));
        let blob = self.seal(entry.content_blob.as_deref());
        let thumbnail = self.seal(thumbnail.as_deref());

        let conn = self.conn.lock().unwrap();

        let pinned_match: Option<i64> = conn
            .query_row(
                "SELECT id FROM clips WHERE pinned = 1 AND content_hash = ?1 LIMIT 1",
                params![hash],
                |row| row.get(0),
            )
            .optional()?;
        if let Some(id) = pinned_match {
            conn.execute("UPDATE clips SET timestamp = ?1 WHERE id = ?2", params![timestamp, id])?;
            return Ok(id);
        }

        conn.execute("DELETE FROM clips WHERE pinned = 0 AND content_hash = ?1", params![hash])?;

        conn.execute(
            "INSERT INTO clips
                (content_type, content_text, content_blob, timestamp, app_source, pinned, thumbnail, content_hash, encrypted)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, 1)",
            params![
                content_type,
                text,
                blob,
                timestamp,
                entry.app_source,
                if entry.pinned { 1 } else { 0 },
                thumbnail,
                hash
            ],
        )?;

        Ok(conn.last_insert_rowid())
    }

    pub fn get_history(&self, limit: usize, offset: usize) -> Result<Vec<ClipboardEntryPreview>> {
        let conn = self.conn.lock().unwrap();

        let mut stmt = conn.prepare(&format!(
            "SELECT {PREVIEW_COLUMNS}
             FROM clips
             ORDER BY pinned DESC, timestamp DESC
             LIMIT ?1 OFFSET ?2"
        ))?;

        let entries = stmt
            .query_map(params![limit, offset], |row| self.row_to_preview(row).map(|(preview, _)| preview))?
            .collect::<Result<Vec<_>, _>>()?;

        Ok(entries)
    }

    pub fn get_entry(&self, id: i64) -> Result<Option<ClipboardEntry>> {
        let conn = self.conn.lock().unwrap();

        type RawEntry = (String, Option<Vec<u8>>, Option<Vec<u8>>, String, Option<String>, i32);
        let raw: Option<RawEntry> = conn
            .query_row(
                "SELECT content_type, content_text, content_blob, timestamp, app_source, pinned
                 FROM clips
                 WHERE id = ?1",
                params![id],
                |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?, row.get(4)?, row.get(5)?)),
            )
            .optional()?;

        Ok(raw.map(|(content_type, text, blob, timestamp, app_source, pinned)| ClipboardEntry {
            id,
            content_type: ContentType::from_string(&content_type).unwrap_or(ContentType::Text),
            content_text: self.open_text(text),
            content_blob: self.open(blob),
            timestamp: parse_timestamp(&timestamp),
            app_source,
            pinned: pinned != 0,
        }))
    }

    pub fn delete_entry(&self, id: i64) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute("DELETE FROM clips WHERE id = ?1", params![id])?;
        Ok(())
    }

    pub fn toggle_pin(&self, id: i64) -> Result<bool> {
        let conn = self.conn.lock().unwrap();

        let current_pinned: i32 = conn.query_row(
            "SELECT pinned FROM clips WHERE id = ?1",
            params![id],
            |row| row.get(0),
        )?;

        let new_pinned = if current_pinned == 0 { 1 } else { 0 };

        conn.execute(
            "UPDATE clips SET pinned = ?1 WHERE id = ?2",
            params![new_pinned, id],
        )?;

        Ok(new_pinned == 1)
    }

    /// Text is encrypted, so matching happens in memory after decryption.
    pub fn search(&self, query: &str, limit: usize) -> Result<Vec<ClipboardEntryPreview>> {
        let needle = query.to_lowercase();
        let conn = self.conn.lock().unwrap();

        let mut stmt = conn.prepare(&format!(
            "SELECT {PREVIEW_COLUMNS}
             FROM clips
             WHERE content_text IS NOT NULL
             ORDER BY pinned DESC, timestamp DESC"
        ))?;
        let mut rows = stmt.query([])?;

        let mut results = Vec::new();
        while let Some(row) = rows.next()? {
            let (preview, text) = self.row_to_preview(row)?;
            if text.is_some_and(|t| t.to_lowercase().contains(&needle)) {
                results.push(preview);
                if results.len() >= limit {
                    break;
                }
            }
        }

        Ok(results)
    }

    pub fn clear_history(&self, keep_pinned: bool) -> Result<usize> {
        let conn = self.conn.lock().unwrap();

        let query = if keep_pinned {
            "DELETE FROM clips WHERE pinned = 0"
        } else {
            "DELETE FROM clips"
        };

        let rows_affected = conn.execute(query, [])?;
        Ok(rows_affected)
    }

    pub fn get_count(&self) -> Result<i64> {
        let conn = self.conn.lock().unwrap();
        let count: i64 = conn.query_row("SELECT COUNT(*) FROM clips", [], |row| row.get(0))?;
        Ok(count)
    }

    pub fn delete_older_than(&self, cutoff: DateTime<Utc>) -> Result<usize> {
        let conn = self.conn.lock().unwrap();
        let rows = conn.execute(
            "DELETE FROM clips WHERE pinned = 0 AND timestamp < ?1",
            params![cutoff.to_rfc3339()],
        )?;
        Ok(rows)
    }

    pub fn trim_to_max(&self, max_unpinned: usize) -> Result<usize> {
        let conn = self.conn.lock().unwrap();
        let rows = conn.execute(
            "DELETE FROM clips WHERE pinned = 0 AND id NOT IN (
                SELECT id FROM clips WHERE pinned = 0
                ORDER BY timestamp DESC, id DESC
                LIMIT ?1
            )",
            params![max_unpinned as i64],
        )?;
        Ok(rows)
    }

    /// `auto_delete_days` and `max_history` of 0 disable that rule. Pinned clips are never removed.
    pub fn apply_retention(&self, auto_delete_days: u32, max_history: u32) -> Result<usize> {
        let mut removed = 0;
        if auto_delete_days > 0 {
            let cutoff = Utc::now() - chrono::Duration::days(auto_delete_days as i64);
            removed += self.delete_older_than(cutoff)?;
        }
        if max_history > 0 {
            removed += self.trim_to_max(max_history as usize)?;
        }
        Ok(removed)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    const TEST_KEY: [u8; KEY_LEN] = [7u8; KEY_LEN];

    fn create_test_store() -> (DataStore, NamedTempFile) {
        let temp_file = NamedTempFile::new().unwrap();
        let store = DataStore::new(temp_file.path().to_path_buf(), &TEST_KEY).unwrap();
        (store, temp_file)
    }

    fn file_contains(path: &std::path::Path, needle: &[u8]) -> bool {
        std::fs::read(path).unwrap().windows(needle.len()).any(|w| w == needle)
    }

    fn sample_png() -> Vec<u8> {
        let mut png_data = Vec::new();
        {
            let mut encoder = png::Encoder::new(&mut png_data, 2, 2);
            encoder.set_color(png::ColorType::Rgba);
            encoder.set_depth(png::BitDepth::Eight);
            encoder.write_header().unwrap().write_image_data(&[255u8; 16]).unwrap();
        }
        png_data
    }

    #[test]
    fn test_init_schema_creates_table() {
        let (store, _temp) = create_test_store();

        // If we can get count, table exists
        let count = store.get_count().unwrap();
        assert_eq!(count, 0);
    }

    #[test]
    fn test_save_and_retrieve_text_entry() {
        let (store, _temp) = create_test_store();

        let entry = ClipboardEntry::new_text(
            "Test content".to_string(),
            Some("TestApp".to_string())
        );

        let id = store.save_entry(&entry).unwrap();
        assert!(id > 0);

        let retrieved = store.get_entry(id).unwrap().unwrap();
        assert_eq!(retrieved.content_text, Some("Test content".to_string()));
        assert_eq!(retrieved.app_source, Some("TestApp".to_string()));
    }

    #[test]
    fn test_save_and_retrieve_image_entry() {
        let (store, _temp) = create_test_store();

        let image_data = vec![1u8, 2, 3, 4, 5];
        let entry = ClipboardEntry::new_image(image_data.clone(), None);

        let id = store.save_entry(&entry).unwrap();
        let retrieved = store.get_entry(id).unwrap().unwrap();

        assert_eq!(retrieved.content_blob, Some(image_data));
        assert!(matches!(retrieved.content_type, ContentType::Image));
    }

    #[test]
    fn test_get_history_returns_newest_first() {
        let (store, _temp) = create_test_store();

        // Save multiple entries
        let entry1 = ClipboardEntry::new_text("First".to_string(), None);
        let entry2 = ClipboardEntry::new_text("Second".to_string(), None);
        let entry3 = ClipboardEntry::new_text("Third".to_string(), None);

        store.save_entry(&entry1).unwrap();
        std::thread::sleep(std::time::Duration::from_millis(10));
        store.save_entry(&entry2).unwrap();
        std::thread::sleep(std::time::Duration::from_millis(10));
        store.save_entry(&entry3).unwrap();

        let history = store.get_history(10, 0).unwrap();

        assert_eq!(history.len(), 3);
        assert_eq!(history[0].preview, "Third"); // Newest first
        assert_eq!(history[2].preview, "First"); // Oldest last
    }

    #[test]
    fn test_delete_entry() {
        let (store, _temp) = create_test_store();

        let entry = ClipboardEntry::new_text("To delete".to_string(), None);
        let id = store.save_entry(&entry).unwrap();

        assert_eq!(store.get_count().unwrap(), 1);

        store.delete_entry(id).unwrap();

        assert_eq!(store.get_count().unwrap(), 0);
        assert!(store.get_entry(id).unwrap().is_none());
    }

    #[test]
    fn test_toggle_pin() {
        let (store, _temp) = create_test_store();

        let entry = ClipboardEntry::new_text("Pin test".to_string(), None);
        let id = store.save_entry(&entry).unwrap();

        // Initially not pinned
        let retrieved = store.get_entry(id).unwrap().unwrap();
        assert!(!retrieved.pinned);

        // Toggle to pinned
        let is_pinned = store.toggle_pin(id).unwrap();
        assert!(is_pinned);

        let retrieved = store.get_entry(id).unwrap().unwrap();
        assert!(retrieved.pinned);

        // Toggle back to unpinned
        let is_pinned = store.toggle_pin(id).unwrap();
        assert!(!is_pinned);
    }

    #[test]
    fn test_pinned_entries_appear_first() {
        let (store, _temp) = create_test_store();

        let entry1 = ClipboardEntry::new_text("Normal 1".to_string(), None);
        let entry2 = ClipboardEntry::new_text("Will be pinned".to_string(), None);
        let entry3 = ClipboardEntry::new_text("Normal 2".to_string(), None);

        store.save_entry(&entry1).unwrap();
        let id2 = store.save_entry(&entry2).unwrap();
        store.save_entry(&entry3).unwrap();

        // Pin the middle entry
        store.toggle_pin(id2).unwrap();

        let history = store.get_history(10, 0).unwrap();

        // Pinned entry should be first despite being older
        assert_eq!(history[0].preview, "Will be pinned");
        assert!(history[0].pinned);
    }

    #[test]
    fn test_search_finds_matching_entries() {
        let (store, _temp) = create_test_store();

        store.save_entry(&ClipboardEntry::new_text("apple pie".to_string(), None)).unwrap();
        store.save_entry(&ClipboardEntry::new_text("banana bread".to_string(), None)).unwrap();
        store.save_entry(&ClipboardEntry::new_text("apple juice".to_string(), None)).unwrap();

        let results = store.search("apple", 10).unwrap();

        assert_eq!(results.len(), 2);
        assert!(results.iter().all(|r| r.preview.contains("apple")));
    }

    #[test]
    fn test_search_is_case_insensitive() {
        let (store, _temp) = create_test_store();

        store.save_entry(&ClipboardEntry::new_text("Hello World".to_string(), None)).unwrap();

        let results = store.search("hello", 10).unwrap();
        assert_eq!(results.len(), 1);

        let results = store.search("WORLD", 10).unwrap();
        assert_eq!(results.len(), 1);
    }

    #[test]
    fn test_search_respects_limit() {
        let (store, _temp) = create_test_store();
        for i in 0..5 {
            store.save_entry(&ClipboardEntry::new_text(format!("match {}", i), None)).unwrap();
        }

        assert_eq!(store.search("match", 3).unwrap().len(), 3);
    }

    #[test]
    fn test_clear_history_keeps_pinned() {
        let (store, _temp) = create_test_store();

        let entry1 = ClipboardEntry::new_text("Normal".to_string(), None);
        let entry2 = ClipboardEntry::new_text("Pinned".to_string(), None);

        store.save_entry(&entry1).unwrap();
        let id2 = store.save_entry(&entry2).unwrap();

        store.toggle_pin(id2).unwrap();

        let deleted = store.clear_history(true).unwrap();

        assert_eq!(deleted, 1); // Only non-pinned deleted
        assert_eq!(store.get_count().unwrap(), 1);

        let remaining = store.get_history(10, 0).unwrap();
        assert_eq!(remaining[0].preview, "Pinned");
    }

    #[test]
    fn test_clear_history_deletes_all() {
        let (store, _temp) = create_test_store();

        store.save_entry(&ClipboardEntry::new_text("Entry 1".to_string(), None)).unwrap();
        store.save_entry(&ClipboardEntry::new_text("Entry 2".to_string(), None)).unwrap();

        let deleted = store.clear_history(false).unwrap();

        assert_eq!(deleted, 2);
        assert_eq!(store.get_count().unwrap(), 0);
    }

    #[test]
    fn test_pagination() {
        let (store, _temp) = create_test_store();

        // Create 10 entries
        for i in 0..10 {
            let entry = ClipboardEntry::new_text(format!("Entry {}", i), None);
            store.save_entry(&entry).unwrap();
        }

        // Get first page
        let page1 = store.get_history(5, 0).unwrap();
        assert_eq!(page1.len(), 5);

        // Get second page
        let page2 = store.get_history(5, 5).unwrap();
        assert_eq!(page2.len(), 5);

        // Ensure no overlap
        assert_ne!(page1[0].id, page2[0].id);
    }

    fn save_text_aged(store: &DataStore, text: &str, days_ago: i64, pinned: bool) -> i64 {
        let mut entry = ClipboardEntry::new_text(text.to_string(), None);
        entry.timestamp = Utc::now() - chrono::Duration::days(days_ago);
        entry.pinned = pinned;
        store.save_entry(&entry).unwrap()
    }

    #[test]
    fn test_retention_deletes_old_unpinned_only() {
        let (store, _temp) = create_test_store();
        let fresh = save_text_aged(&store, "fresh", 1, false);
        let old = save_text_aged(&store, "old", 10, false);
        let old_pinned = save_text_aged(&store, "old pinned", 10, true);

        let removed = store.apply_retention(7, 0).unwrap();

        assert_eq!(removed, 1);
        assert!(store.get_entry(fresh).unwrap().is_some());
        assert!(store.get_entry(old).unwrap().is_none());
        assert!(store.get_entry(old_pinned).unwrap().is_some());
    }

    #[test]
    fn test_retention_trims_to_max_keeping_newest_and_pinned() {
        let (store, _temp) = create_test_store();
        let pinned = save_text_aged(&store, "pinned", 30, true);
        let oldest = save_text_aged(&store, "a", 3, false);
        let middle = save_text_aged(&store, "b", 2, false);
        let newest = save_text_aged(&store, "c", 1, false);

        let removed = store.apply_retention(0, 2).unwrap();

        assert_eq!(removed, 1);
        assert!(store.get_entry(oldest).unwrap().is_none());
        assert!(store.get_entry(middle).unwrap().is_some());
        assert!(store.get_entry(newest).unwrap().is_some());
        assert!(store.get_entry(pinned).unwrap().is_some());
    }

    #[test]
    fn test_saving_identical_text_moves_it_to_top_without_duplicate() {
        let (store, _temp) = create_test_store();
        save_text_aged(&store, "repeat", 2, false);
        save_text_aged(&store, "other", 1, false);

        store.save_entry(&ClipboardEntry::new_text("repeat".to_string(), None)).unwrap();

        let history = store.get_history(10, 0).unwrap();
        assert_eq!(history.len(), 2);
        assert_eq!(history[0].preview, "repeat");
    }

    #[test]
    fn test_saving_identical_pinned_text_keeps_single_pinned_entry() {
        let (store, _temp) = create_test_store();
        let pinned = save_text_aged(&store, "keep me", 5, true);

        let id = store.save_entry(&ClipboardEntry::new_text("keep me".to_string(), None)).unwrap();

        assert_eq!(id, pinned);
        assert_eq!(store.get_count().unwrap(), 1);
        assert!(store.get_entry(pinned).unwrap().unwrap().pinned);
    }

    #[test]
    fn test_image_history_includes_thumbnail_data_url() {
        let (store, _temp) = create_test_store();
        store.save_entry(&ClipboardEntry::new_image(sample_png(), None)).unwrap();
        store.save_entry(&ClipboardEntry::new_text("text".to_string(), None)).unwrap();

        let history = store.get_history(10, 0).unwrap();
        let image = history.iter().find(|c| matches!(c.content_type, ContentType::Image)).unwrap();
        let text = history.iter().find(|c| matches!(c.content_type, ContentType::Text)).unwrap();

        assert!(image.thumbnail.as_deref().unwrap().starts_with("data:image/png;base64,"));
        assert!(text.thumbnail.is_none());
    }

    #[test]
    fn test_retention_disabled_removes_nothing() {
        let (store, _temp) = create_test_store();
        save_text_aged(&store, "ancient", 365, false);
        save_text_aged(&store, "recent", 0, false);

        assert_eq!(store.apply_retention(0, 0).unwrap(), 0);
        assert_eq!(store.get_count().unwrap(), 2);
    }

    #[test]
    fn test_database_file_contains_no_plaintext() {
        let (store, temp) = create_test_store();
        store.save_entry(&ClipboardEntry::new_text("my-secret-password-123".to_string(), None)).unwrap();
        let png = sample_png();
        store.save_entry(&ClipboardEntry::new_image(png.clone(), None)).unwrap();
        drop(store);

        assert!(!file_contains(temp.path(), b"my-secret-password-123"));
        assert!(!file_contains(temp.path(), &png));
    }

    #[test]
    fn test_legacy_plaintext_rows_are_encrypted_on_open() {
        let temp = NamedTempFile::new().unwrap();
        {
            let conn = Connection::open(temp.path()).unwrap();
            conn.execute_batch(
                "CREATE TABLE clips (
                    id INTEGER PRIMARY KEY AUTOINCREMENT,
                    content_type TEXT NOT NULL,
                    content_text TEXT,
                    content_blob BLOB,
                    timestamp TEXT NOT NULL,
                    app_source TEXT,
                    pinned INTEGER NOT NULL DEFAULT 0
                );
                INSERT INTO clips (content_type, content_text, timestamp, pinned)
                VALUES ('text', 'legacy-plaintext-clip', '2026-01-01T00:00:00+00:00', 1);",
            )
            .unwrap();
        }

        let store = DataStore::new(temp.path().to_path_buf(), &TEST_KEY).unwrap();
        let history = store.get_history(10, 0).unwrap();
        assert_eq!(history.len(), 1);
        assert_eq!(history[0].preview, "legacy-plaintext-clip");
        assert!(history[0].pinned);
        assert_eq!(store.search("LEGACY", 10).unwrap().len(), 1);

        // Dedupe must still recognise the migrated row.
        store.save_entry(&ClipboardEntry::new_text("legacy-plaintext-clip".to_string(), None)).unwrap();
        assert_eq!(store.get_count().unwrap(), 1);
        drop(store);

        assert!(!file_contains(temp.path(), b"legacy-plaintext-clip"));
    }

    #[test]
    fn test_reopening_with_same_key_keeps_history() {
        let temp = NamedTempFile::new().unwrap();
        let store = DataStore::new(temp.path().to_path_buf(), &TEST_KEY).unwrap();
        store.save_entry(&ClipboardEntry::new_text("persisted".to_string(), None)).unwrap();
        drop(store);

        let store = DataStore::new(temp.path().to_path_buf(), &TEST_KEY).unwrap();
        assert_eq!(store.get_history(10, 0).unwrap()[0].preview, "persisted");
    }

    #[test]
    fn test_changed_key_discards_unreadable_clips() {
        let temp = NamedTempFile::new().unwrap();
        let store = DataStore::new(temp.path().to_path_buf(), &TEST_KEY).unwrap();
        store.save_entry(&ClipboardEntry::new_text("old key".to_string(), None)).unwrap();
        drop(store);

        let store = DataStore::new(temp.path().to_path_buf(), &[9u8; KEY_LEN]).unwrap();
        assert_eq!(store.get_count().unwrap(), 0);

        store.save_entry(&ClipboardEntry::new_text("new key".to_string(), None)).unwrap();
        assert_eq!(store.get_history(10, 0).unwrap()[0].preview, "new key");
    }
}
