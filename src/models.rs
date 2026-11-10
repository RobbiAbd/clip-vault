use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ContentType {
    Text,
    RichText,
    Image,
    Files,
}

impl ContentType {
    pub fn to_string(&self) -> String {
        match self {
            ContentType::Text => "text".to_string(),
            ContentType::RichText => "richtext".to_string(),
            ContentType::Image => "image".to_string(),
            ContentType::Files => "files".to_string(),
        }
    }

    pub fn from_string(s: &str) -> Option<Self> {
        match s {
            "text" => Some(ContentType::Text),
            "richtext" => Some(ContentType::RichText),
            "image" => Some(ContentType::Image),
            "files" => Some(ContentType::Files),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClipboardEntry {
    pub id: i64,
    pub content_type: ContentType,
    pub content_text: Option<String>,
    pub content_blob: Option<Vec<u8>>,
    pub timestamp: DateTime<Utc>,
    pub app_source: Option<String>,
    pub pinned: bool,
}

impl ClipboardEntry {
    pub fn new_text(text: String, app_source: Option<String>) -> Self {
        Self {
            id: 0,
            content_type: ContentType::Text,
            content_text: Some(text),
            content_blob: None,
            timestamp: Utc::now(),
            app_source,
            pinned: false,
        }
    }

    pub fn new_image(image_data: Vec<u8>, app_source: Option<String>) -> Self {
        Self {
            id: 0,
            content_type: ContentType::Image,
            content_text: None,
            content_blob: Some(image_data),
            timestamp: Utc::now(),
            app_source,
            pinned: false,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClipboardEntryPreview {
    pub id: i64,
    pub content_type: ContentType,
    pub preview: String,
    pub timestamp: DateTime<Utc>,
    pub app_source: Option<String>,
    pub pinned: bool,
}

impl From<ClipboardEntry> for ClipboardEntryPreview {
    fn from(entry: ClipboardEntry) -> Self {
        let preview = match entry.content_type {
            ContentType::Text | ContentType::RichText => {
                entry
                    .content_text
                    .clone()
                    .unwrap_or_default()
                    .chars()
                    .take(200)
                    .collect::<String>()
            }
            ContentType::Image => format!("Image ({} bytes)", entry.content_blob.as_ref().map_or(0, |b| b.len())),
            ContentType::Files => entry.content_text.clone().unwrap_or_else(|| "Files".to_string()),
        };

        ClipboardEntryPreview {
            id: entry.id,
            content_type: entry.content_type,
            preview,
            timestamp: entry.timestamp,
            app_source: entry.app_source,
            pinned: entry.pinned,
        }
    }
}
