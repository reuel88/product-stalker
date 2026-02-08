//! Notification helper utilities
//!
//! This module provides Tauri-agnostic notification data structures
//! that can be used by the domain layer for building notification content.

use serde::Serialize;

/// Data needed to display a notification (Tauri-agnostic)
#[derive(Debug, Clone, Serialize)]
pub struct NotificationData {
    pub title: String,
    pub body: String,
}

impl NotificationData {
    /// Create a new notification with the given title and body
    pub fn new(title: impl Into<String>, body: impl Into<String>) -> Self {
        Self {
            title: title.into(),
            body: body.into(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_notification_data_new() {
        let notification = NotificationData::new("Test Title", "Test Body");
        assert_eq!(notification.title, "Test Title");
        assert_eq!(notification.body, "Test Body");
    }

    #[test]
    fn test_notification_data_serialize() {
        let notification = NotificationData::new("Title", "Body");
        let json = serde_json::to_string(&notification).unwrap();
        assert!(json.contains("\"title\":\"Title\""));
        assert!(json.contains("\"body\":\"Body\""));
    }

    #[test]
    fn test_notification_data_clone() {
        let notification = NotificationData::new("Original", "Content");
        let cloned = notification.clone();
        assert_eq!(cloned.title, "Original");
        assert_eq!(cloned.body, "Content");
    }
}
