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
    /// PNG data URL, only set for images.
    pub thumbnail: Option<String>,
}

pub fn preview_text(content_type: &ContentType, text: Option<&str>, blob_len: usize) -> String {
    match content_type {
        ContentType::Text | ContentType::RichText => {
            text.unwrap_or_default().chars().take(200).collect()
        }
        ContentType::Image => format!("Image ({} bytes)", blob_len),
        ContentType::Files => text.unwrap_or("Files").to_string(),
    }
}

impl From<ClipboardEntry> for ClipboardEntryPreview {
    fn from(entry: ClipboardEntry) -> Self {
        let preview = preview_text(
            &entry.content_type,
            entry.content_text.as_deref(),
            entry.content_blob.as_ref().map_or(0, |b| b.len()),
        );

        ClipboardEntryPreview {
            id: entry.id,
            content_type: entry.content_type,
            preview,
            timestamp: entry.timestamp,
            app_source: entry.app_source,
            pinned: entry.pinned,
            thumbnail: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_content_type_to_string() {
        assert_eq!(ContentType::Text.to_string(), "text");
        assert_eq!(ContentType::RichText.to_string(), "richtext");
        assert_eq!(ContentType::Image.to_string(), "image");
        assert_eq!(ContentType::Files.to_string(), "files");
    }

    #[test]
    fn test_content_type_from_string() {
        assert!(matches!(ContentType::from_string("text"), Some(ContentType::Text)));
        assert!(matches!(ContentType::from_string("richtext"), Some(ContentType::RichText)));
        assert!(matches!(ContentType::from_string("image"), Some(ContentType::Image)));
        assert!(matches!(ContentType::from_string("files"), Some(ContentType::Files)));
        assert!(ContentType::from_string("invalid").is_none());
    }

    #[test]
    fn test_new_text_entry() {
        let text = "Hello, World!".to_string();
        let app = Some("Test App".to_string());
        let entry = ClipboardEntry::new_text(text.clone(), app.clone());

        assert_eq!(entry.id, 0);
        assert!(matches!(entry.content_type, ContentType::Text));
        assert_eq!(entry.content_text, Some(text));
        assert!(entry.content_blob.is_none());
        assert_eq!(entry.app_source, app);
        assert!(!entry.pinned);
    }

    #[test]
    fn test_new_image_entry() {
        let image_data = vec![0u8, 1, 2, 3, 4, 5];
        let app = Some("Paint".to_string());
        let entry = ClipboardEntry::new_image(image_data.clone(), app.clone());

        assert_eq!(entry.id, 0);
        assert!(matches!(entry.content_type, ContentType::Image));
        assert!(entry.content_text.is_none());
        assert_eq!(entry.content_blob, Some(image_data));
        assert_eq!(entry.app_source, app);
        assert!(!entry.pinned);
    }

    #[test]
    fn test_preview_from_text_entry() {
        let long_text = "a".repeat(300);
        let entry = ClipboardEntry::new_text(long_text.clone(), None);
        let preview = ClipboardEntryPreview::from(entry);

        // Preview should be truncated to 200 chars
        assert_eq!(preview.preview.len(), 200);
        assert_eq!(preview.preview, "a".repeat(200));
    }

    #[test]
    fn test_preview_from_image_entry() {
        let image_data = vec![0u8; 1024];
        let entry = ClipboardEntry::new_image(image_data, None);
        let preview = ClipboardEntryPreview::from(entry);

        assert_eq!(preview.preview, "Image (1024 bytes)");
    }

    #[test]
    fn test_preview_preserves_metadata() {
        let entry = ClipboardEntry {
            id: 42,
            content_type: ContentType::Text,
            content_text: Some("Test".to_string()),
            content_blob: None,
            timestamp: Utc::now(),
            app_source: Some("App".to_string()),
            pinned: true,
        };
        
        let preview = ClipboardEntryPreview::from(entry.clone());
        
        assert_eq!(preview.id, entry.id);
        assert_eq!(preview.app_source, entry.app_source);
        assert_eq!(preview.pinned, entry.pinned);
    }
}
