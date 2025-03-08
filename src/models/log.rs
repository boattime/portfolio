use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum LogLevel {
    Debug,
    Info,
    Warning,
    Error,
}

impl LogLevel {
    pub fn as_str(&self) -> &'static str {
        match self {
            LogLevel::Debug => "DEBUG",
            LogLevel::Info => "INFO",
            LogLevel::Warning => "WARNING",
            LogLevel::Error => "ERROR",
        }
    }

    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_uppercase().as_str() {
            "DEBUG" => Some(LogLevel::Debug),
            "INFO" => Some(LogLevel::Info),
            "WARNING" | "WARN" => Some(LogLevel::Warning),
            "ERROR" | "ERR" => Some(LogLevel::Error),
            _ => None,
        }
    }
}

impl fmt::Display for LogLevel {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct LogEntry {
    pub message: String,
    pub level: LogLevel,
    pub timestamp: DateTime<Utc>,
    pub source: String,
    pub metadata: HashMap<String, String>,
}

impl LogEntry {
    pub fn new(message: &str, level: LogLevel, source: &str) -> Self {
        Self {
            message: message.to_string(),
            level,
            timestamp: Utc::now(),
            source: source.to_string(),
            metadata: HashMap::new(),
        }
    }

    pub fn with_timestamp(
        message: &str,
        level: LogLevel,
        source: &str,
        timestamp: DateTime<Utc>,
    ) -> Self {
        Self {
            message: message.to_string(),
            level,
            timestamp,
            source: source.to_string(),
            metadata: HashMap::new(),
        }
    }

    pub fn with_metadata(mut self, key: &str, value: &str) -> Self {
        self.metadata.insert(key.to_string(), value.to_string());
        self
    }

    pub fn with_metadata_map(mut self, metadata: HashMap<String, String>) -> Self {
        self.metadata.extend(metadata);
        self
    }

    pub fn has_metadata(&self, key: &str) -> bool {
        self.metadata.contains_key(key)
    }

    pub fn get_metadata(&self, key: &str) -> Option<&String> {
        self.metadata.get(key)
    }

    pub fn format(&self) -> String {
        format!(
            "[{}] [{}] [{}]: {}",
            self.timestamp.format("%Y-%m-%d %H:%M:%S%.3f"),
            self.level,
            self.source,
            self.message
        )
    }

    pub fn is_level_at_least(&self, level: LogLevel) -> bool {
        self.level >= level
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeZone;

    #[test]
    fn test_log_level_ordering() {
        assert!(LogLevel::Error > LogLevel::Warning);
        assert!(LogLevel::Warning > LogLevel::Info);
        assert!(LogLevel::Info > LogLevel::Debug);
    }

    #[test]
    fn test_log_level_from_str() {
        assert_eq!(LogLevel::from_str("DEBUG"), Some(LogLevel::Debug));
        assert_eq!(LogLevel::from_str("debug"), Some(LogLevel::Debug));
        assert_eq!(LogLevel::from_str("INFO"), Some(LogLevel::Info));
        assert_eq!(LogLevel::from_str("WARNING"), Some(LogLevel::Warning));
        assert_eq!(LogLevel::from_str("WARN"), Some(LogLevel::Warning));
        assert_eq!(LogLevel::from_str("ERROR"), Some(LogLevel::Error));
        assert_eq!(LogLevel::from_str("ERR"), Some(LogLevel::Error));
        assert_eq!(LogLevel::from_str("UNKNOWN"), None);
    }

    #[test]
    fn test_log_level_display() {
        assert_eq!(format!("{}", LogLevel::Debug), "DEBUG");
        assert_eq!(format!("{}", LogLevel::Info), "INFO");
        assert_eq!(format!("{}", LogLevel::Warning), "WARNING");
        assert_eq!(format!("{}", LogLevel::Error), "ERROR");
    }

    #[test]
    fn test_new_log_entry() {
        let log = LogEntry::new("Server started", LogLevel::Info, "app");

        assert_eq!(log.message, "Server started");
        assert_eq!(log.level, LogLevel::Info);
        assert_eq!(log.source, "app");
        assert!(log.metadata.is_empty());
    }

    #[test]
    fn test_with_timestamp() {
        let timestamp = Utc.with_ymd_and_hms(2025, 3, 8, 12, 0, 0).unwrap();
        let log = LogEntry::with_timestamp(
            "Database connection failed",
            LogLevel::Error,
            "db_connector",
            timestamp,
        );

        assert_eq!(log.message, "Database connection failed");
        assert_eq!(log.level, LogLevel::Error);
        assert_eq!(log.timestamp, timestamp);
    }

    #[test]
    fn test_with_metadata() {
        let log = LogEntry::new("Request processed", LogLevel::Info, "api")
            .with_metadata("duration_ms", "120");

        assert_eq!(log.get_metadata("duration_ms"), Some(&"120".to_string()));
        assert!(log.has_metadata("duration_ms"));
        assert!(!log.has_metadata("status"));
    }

    #[test]
    fn test_with_metadata_map() {
        let mut metadata = HashMap::new();
        metadata.insert("status".to_string(), "200".to_string());
        metadata.insert("bytes_sent".to_string(), "1024".to_string());

        let log = LogEntry::new("Response sent", LogLevel::Debug, "http_server")
            .with_metadata_map(metadata);

        assert_eq!(log.get_metadata("status"), Some(&"200".to_string()));
        assert_eq!(log.get_metadata("bytes_sent"), Some(&"1024".to_string()));
        assert_eq!(log.metadata.len(), 2);
    }

    #[test]
    fn test_format() {
        let timestamp = Utc.with_ymd_and_hms(2025, 3, 8, 12, 0, 0).unwrap();
        let log = LogEntry::with_timestamp(
            "Processing request",
            LogLevel::Info,
            "request_handler",
            timestamp,
        );

        let formatted = log.format();
        assert!(formatted.contains("[2025-03-08 12:00:00"));
        assert!(formatted.contains("[INFO]"));
        assert!(formatted.contains("[request_handler]"));
        assert!(formatted.contains("Processing request"));
    }

    #[test]
    fn test_is_level_at_least() {
        let debug_log = LogEntry::new("Debug message", LogLevel::Debug, "test");
        let info_log = LogEntry::new("Info message", LogLevel::Info, "test");
        let warning_log = LogEntry::new("Warning message", LogLevel::Warning, "test");
        let error_log = LogEntry::new("Error message", LogLevel::Error, "test");

        assert!(debug_log.is_level_at_least(LogLevel::Debug));
        assert!(!debug_log.is_level_at_least(LogLevel::Info));

        assert!(info_log.is_level_at_least(LogLevel::Debug));
        assert!(info_log.is_level_at_least(LogLevel::Info));
        assert!(!info_log.is_level_at_least(LogLevel::Warning));

        assert!(warning_log.is_level_at_least(LogLevel::Debug));
        assert!(warning_log.is_level_at_least(LogLevel::Info));
        assert!(warning_log.is_level_at_least(LogLevel::Warning));
        assert!(!warning_log.is_level_at_least(LogLevel::Error));

        assert!(error_log.is_level_at_least(LogLevel::Debug));
        assert!(error_log.is_level_at_least(LogLevel::Info));
        assert!(error_log.is_level_at_least(LogLevel::Warning));
        assert!(error_log.is_level_at_least(LogLevel::Error));
    }
}
