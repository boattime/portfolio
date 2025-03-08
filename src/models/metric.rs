use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Metric {
    pub name: String,
    pub value: f64,
    pub timestamp: DateTime<Utc>,
    pub labels: HashMap<String, String>,
}

impl Metric {
    pub fn new(name: &str, value: f64) -> Self {
        Self {
            name: name.to_string(),
            value,
            timestamp: Utc::now(),
            labels: HashMap::new(),
        }
    }

    pub fn with_timestamp(name: &str, value: f64, timestamp: DateTime<Utc>) -> Self {
        Self {
            name: name.to_string(),
            value,
            timestamp,
            labels: HashMap::new(),
        }
    }

    pub fn with_label(mut self, key: &str, value: &str) -> Self {
        self.labels.insert(key.to_string(), value.to_string());
        self
    }

    pub fn with_labels(mut self, labels: HashMap<String, String>) -> Self {
        self.labels.extend(labels);
        self
    }

    pub fn get_label(&self, key: &str) -> Option<&String> {
        self.labels.get(key)
    }

    pub fn has_label(&self, key: &str) -> bool {
        self.labels.contains_key(key)
    }

    pub fn has_label_value(&self, key: &str, value: &str) -> bool {
        match self.labels.get(key) {
            Some(label_value) => label_value == value,
            None => false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeZone;

    #[test]
    fn test_new_metric() {
        let metric = Metric::new("cpu_usage", 85.5);
        assert_eq!(metric.name, "cpu_usage");
        assert_eq!(metric.value, 85.5);
        assert!(metric.labels.is_empty());
    }

    #[test]
    fn test_with_timestamp() {
        let timestamp = Utc.with_ymd_and_hms(2025, 3, 8, 12, 0, 0).unwrap();
        let metric = Metric::with_timestamp("memory_usage", 42.8, timestamp);
        assert_eq!(metric.name, "memory_usage");
        assert_eq!(metric.value, 42.8);
        assert_eq!(metric.timestamp, timestamp);
    }

    #[test]
    fn test_with_label() {
        let metric = Metric::new("disk_space", 75.2).with_label("unit", "percent");

        assert_eq!(metric.get_label("unit"), Some(&"percent".to_string()));
        assert_eq!(metric.labels.len(), 1);
    }

    #[test]
    fn test_with_labels() {
        let mut labels = HashMap::new();
        labels.insert("region".to_string(), "us-west-1".to_string());
        labels.insert("instance".to_string(), "i-1234abcd".to_string());

        let metric = Metric::new("latency", 123.4).with_labels(labels);

        assert_eq!(metric.get_label("region"), Some(&"us-west-1".to_string()));
        assert_eq!(
            metric.get_label("instance"),
            Some(&"i-1234abcd".to_string())
        );
        assert_eq!(metric.labels.len(), 2);
    }

    #[test]
    fn test_has_label_value() {
        let metric = Metric::new("response_time", 230.5)
            .with_label("status", "200")
            .with_label("method", "GET");

        assert!(metric.has_label_value("status", "200"));
        assert!(metric.has_label_value("method", "GET"));
        assert!(!metric.has_label_value("status", "404"));
    }
}
