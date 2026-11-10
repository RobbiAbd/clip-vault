use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use rusqlite::{params, Connection};
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

use crate::models::{ClipboardEntry, ClipboardEntryPreview, ContentType};

pub struct DataStore {
    conn: Arc<Mutex<Connection>>,
}

impl DataStore {
    pub fn new(db_path: PathBuf) -> Result<Self> {
        let conn = Connection::open(db_path).context("Failed to open database")?;
        
        let store = DataStore {
            conn: Arc::new(Mutex::new(conn)),
        };
        
        store.init_schema()?;
        Ok(store)
    }

    fn init_schema(&self) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        
        conn.execute(
            "CREATE TABLE IF NOT EXISTS clips (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                content_type TEXT NOT NULL,
                content_text TEXT,
                content_blob BLOB,
                timestamp TEXT NOT NULL,
                app_source TEXT,
                pinned INTEGER NOT NULL DEFAULT 0
            )",
            [],
        )?;

        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_timestamp ON clips(timestamp DESC)",
            [],
        )?;

        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_pinned ON clips(pinned DESC, timestamp DESC)",
            [],
        )?;

        Ok(())
    }

    pub fn save_entry(&self, entry: &ClipboardEntry) -> Result<i64> {
        let conn = self.conn.lock().unwrap();
        
        conn.execute(
            "INSERT INTO clips (content_type, content_text, content_blob, timestamp, app_source, pinned)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            params![
                entry.content_type.to_string(),
                entry.content_text,
                entry.content_blob,
                entry.timestamp.to_rfc3339(),
                entry.app_source,
                if entry.pinned { 1 } else { 0 }
            ],
        )?;

        Ok(conn.last_insert_rowid())
    }

    pub fn get_history(&self, limit: usize, offset: usize) -> Result<Vec<ClipboardEntryPreview>> {
        let conn = self.conn.lock().unwrap();
        
        let mut stmt = conn.prepare(
            "SELECT id, content_type, content_text, content_blob, timestamp, app_source, pinned
             FROM clips
             ORDER BY pinned DESC, timestamp DESC
             LIMIT ?1 OFFSET ?2"
        )?;

        let entries = stmt.query_map(params![limit, offset], |row| {
            let content_type_str: String = row.get(1)?;
            let content_type = ContentType::from_string(&content_type_str)
                .unwrap_or(ContentType::Text);
            
            let timestamp_str: String = row.get(4)?;
            let timestamp = DateTime::parse_from_rfc3339(&timestamp_str)
                .map(|dt| dt.with_timezone(&Utc))
                .unwrap_or_else(|_| Utc::now());

            let entry = ClipboardEntry {
                id: row.get(0)?,
                content_type: content_type.clone(),
                content_text: row.get(2)?,
                content_blob: row.get(3)?,
                timestamp,
                app_source: row.get(5)?,
                pinned: row.get::<_, i32>(6)? != 0,
            };

            Ok(ClipboardEntryPreview::from(entry))
        })?
        .collect::<Result<Vec<_>, _>>()?;

        Ok(entries)
    }

    pub fn get_entry(&self, id: i64) -> Result<Option<ClipboardEntry>> {
        let conn = self.conn.lock().unwrap();
        
        let mut stmt = conn.prepare(
            "SELECT id, content_type, content_text, content_blob, timestamp, app_source, pinned
             FROM clips
             WHERE id = ?1"
        )?;

        let entry = stmt.query_row(params![id], |row| {
            let content_type_str: String = row.get(1)?;
            let content_type = ContentType::from_string(&content_type_str)
                .unwrap_or(ContentType::Text);
            
            let timestamp_str: String = row.get(4)?;
            let timestamp = DateTime::parse_from_rfc3339(&timestamp_str)
                .map(|dt| dt.with_timezone(&Utc))
                .unwrap_or_else(|_| Utc::now());

            Ok(ClipboardEntry {
                id: row.get(0)?,
                content_type,
                content_text: row.get(2)?,
                content_blob: row.get(3)?,
                timestamp,
                app_source: row.get(5)?,
                pinned: row.get::<_, i32>(6)? != 0,
            })
        });

        match entry {
            Ok(e) => Ok(Some(e)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(e.into()),
        }
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

    pub fn search(&self, query: &str, limit: usize) -> Result<Vec<ClipboardEntryPreview>> {
        let conn = self.conn.lock().unwrap();
        
        let search_pattern = format!("%{}%", query);
        
        let mut stmt = conn.prepare(
            "SELECT id, content_type, content_text, content_blob, timestamp, app_source, pinned
             FROM clips
             WHERE content_text LIKE ?1
             ORDER BY pinned DESC, timestamp DESC
             LIMIT ?2"
        )?;

        let entries = stmt.query_map(params![search_pattern, limit], |row| {
            let content_type_str: String = row.get(1)?;
            let content_type = ContentType::from_string(&content_type_str)
                .unwrap_or(ContentType::Text);
            
            let timestamp_str: String = row.get(4)?;
            let timestamp = DateTime::parse_from_rfc3339(&timestamp_str)
                .map(|dt| dt.with_timezone(&Utc))
                .unwrap_or_else(|_| Utc::now());

            let entry = ClipboardEntry {
                id: row.get(0)?,
                content_type: content_type.clone(),
                content_text: row.get(2)?,
                content_blob: row.get(3)?,
                timestamp,
                app_source: row.get(5)?,
                pinned: row.get::<_, i32>(6)? != 0,
            };

            Ok(ClipboardEntryPreview::from(entry))
        })?
        .collect::<Result<Vec<_>, _>>()?;

        Ok(entries)
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
}
