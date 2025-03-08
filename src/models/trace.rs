use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Trace {
    pub name: String,
    pub duration_ms: u64,
    pub start_time: DateTime<Utc>,
    pub end_time: DateTime<Utc>,
    pub parent_id: Option<String>,
    pub span_id: String,
    pub metadata: HashMap<String, String>,
}

impl Trace {
    pub fn new(name: &str, duration_ms: u64) -> Self {
        let end_time = Utc::now();
        let start_time = end_time - Duration::milliseconds(duration_ms as i64);

        Self {
            name: name.to_string(),
            duration_ms,
            start_time,
            end_time,
            parent_id: None,
            span_id: Uuid::new_v4().to_string(),
            metadata: HashMap::new(),
        }
    }

    pub fn with_times(name: &str, start_time: DateTime<Utc>, end_time: DateTime<Utc>) -> Self {
        let duration = end_time.signed_duration_since(start_time);
        let duration_ms = duration.num_milliseconds().max(0) as u64;

        Self {
            name: name.to_string(),
            duration_ms,
            start_time,
            end_time,
            parent_id: None,
            span_id: Uuid::new_v4().to_string(),
            metadata: HashMap::new(),
        }
    }

    pub fn with_parent(mut self, parent_id: &str) -> Self {
        self.parent_id = Some(parent_id.to_string());
        self
    }

    pub fn with_metadata(mut self, key: &str, value: &str) -> Self {
        self.metadata.insert(key.to_string(), value.to_string());
        self
    }

    pub fn with_metadata_map(mut self, metadata: HashMap<String, String>) -> Self {
        self.metadata.extend(metadata);
        self
    }

    pub fn is_root(&self) -> bool {
        self.parent_id.is_none()
    }

    pub fn has_metadata(&self, key: &str) -> bool {
        self.metadata.contains_key(key)
    }

    pub fn get_metadata(&self, key: &str) -> Option<&String> {
        self.metadata.get(key)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeZone;

    #[test]
    fn test_new_trace() {
        let trace = Trace::new("request_handler", 150);

        assert_eq!(trace.name, "request_handler");
        assert_eq!(trace.duration_ms, 150);
        assert!((trace.end_time - trace.start_time).num_milliseconds() == 150);
        assert!(trace.is_root());
        assert!(trace.metadata.is_empty());
    }

    #[test]
    fn test_with_times() {
        let start = Utc.with_ymd_and_hms(2025, 3, 8, 12, 0, 0).unwrap();
        let end = Utc.with_ymd_and_hms(2025, 3, 8, 12, 0, 1).unwrap();

        let trace = Trace::with_times("db_query", start, end);

        assert_eq!(trace.name, "db_query");
        assert_eq!(trace.start_time, start);
        assert_eq!(trace.end_time, end);
    }

    #[test]
    fn test_with_parent() {
        let parent_id = Uuid::new_v4().to_string();
        let trace = Trace::new("child_operation", 50).with_parent(&parent_id);

        assert_eq!(trace.parent_id, Some(parent_id));
        assert!(!trace.is_root());
    }

    #[test]
    fn test_with_metadata() {
        let trace = Trace::new("api_call", 200).with_metadata("method", "GET");

        assert_eq!(trace.get_metadata("method"), Some(&"GET".to_string()));
        assert!(!trace.has_metadata("endpoint"));
        assert!(!trace.has_metadata("status"));
    }

    #[test]
    fn test_with_metadata_map() {
        let mut metadata = HashMap::new();
        metadata.insert("status".to_string(), "200".to_string());
        metadata.insert("bytes_sent".to_string(), "1024".to_string());

        let trace = Trace::new("response", 75).with_metadata_map(metadata);

        assert_eq!(trace.get_metadata("status"), Some(&"200".to_string()));
        assert_eq!(trace.get_metadata("bytes_sent"), Some(&"1024".to_string()));
        assert_eq!(trace.metadata.len(), 2);
    }
}
