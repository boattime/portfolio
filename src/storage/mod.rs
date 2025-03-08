use crate::error::{Error, Result};
use crate::models::{LogEntry, LogLevel, Metric, Trace};
use chrono::{DateTime, Utc};
use std::sync::{Arc, RwLock};

#[derive(Debug, Clone)]
pub struct MetricStorage {
    metrics: Arc<RwLock<Vec<Metric>>>,
}

impl MetricStorage {
    pub fn new() -> Self {
        Self {
            metrics: Arc::new(RwLock::new(Vec::new())),
        }
    }

    pub fn add(&self, metric: Metric) -> Result<()> {
        let mut metrics = self.metrics.write().map_err(|e| {
            Error::Unknown(format!(
                "Failed to acquire write lock on metric storage: {}",
                e
            ))
        })?;
        metrics.push(metric);
        Ok(())
    }

    pub fn get_all(&self) -> Result<Vec<Metric>> {
        let metrics = self.metrics.read().map_err(|e| {
            Error::Unknown(format!(
                "Failed to acquire read lock on metric storage: {}",
                e
            ))
        })?;
        Ok(metrics.clone())
    }

    pub fn get_by_name(&self, name: &str) -> Result<Vec<Metric>> {
        let metrics = self.metrics.read().map_err(|e| {
            Error::Unknown(format!(
                "Failed to acquire read lock on metric storage: {}",
                e
            ))
        })?;

        let filtered = metrics.iter().filter(|m| m.name == name).cloned().collect();

        Ok(filtered)
    }

    pub fn get_by_time_range(
        &self,
        start: DateTime<Utc>,
        end: DateTime<Utc>,
    ) -> Result<Vec<Metric>> {
        let metrics = self.metrics.read().map_err(|e| {
            Error::Unknown(format!(
                "Failed to acquire read lock on metric storage: {}",
                e
            ))
        })?;

        let filtered = metrics
            .iter()
            .filter(|m| m.timestamp >= start && m.timestamp <= end)
            .cloned()
            .collect();

        Ok(filtered)
    }

    pub fn get_by_label(&self, key: &str, value: &str) -> Result<Vec<Metric>> {
        let metrics = self.metrics.read().map_err(|e| {
            Error::Unknown(format!(
                "Failed to acquire read lock on metric storage: {}",
                e
            ))
        })?;

        let filtered = metrics
            .iter()
            .filter(|m| m.has_label_value(key, value))
            .cloned()
            .collect();

        Ok(filtered)
    }

    pub fn clear(&self) -> Result<()> {
        let mut metrics = self.metrics.write().map_err(|e| {
            Error::Unknown(format!(
                "Failed to acquire write lock on metric storage: {}",
                e
            ))
        })?;
        metrics.clear();
        Ok(())
    }

    pub fn count(&self) -> Result<usize> {
        let metrics = self.metrics.read().map_err(|e| {
            Error::Unknown(format!(
                "Failed to acquire read lock on metric storage: {}",
                e
            ))
        })?;
        Ok(metrics.len())
    }
}

impl Default for MetricStorage {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone)]
pub struct TraceStorage {
    traces: Arc<RwLock<Vec<Trace>>>,
}

impl TraceStorage {
    pub fn new() -> Self {
        Self {
            traces: Arc::new(RwLock::new(Vec::new())),
        }
    }

    pub fn add(&self, trace: Trace) -> Result<()> {
        let mut traces = self.traces.write().map_err(|e| {
            Error::Unknown(format!(
                "Failed to acquire write lock on trace storage: {}",
                e
            ))
        })?;
        traces.push(trace);
        Ok(())
    }

    pub fn get_all(&self) -> Result<Vec<Trace>> {
        let traces = self.traces.read().map_err(|e| {
            Error::Unknown(format!(
                "Failed to acquire read lock on trace storage: {}",
                e
            ))
        })?;
        Ok(traces.clone())
    }

    pub fn get_by_id(&self, span_id: &str) -> Result<Option<Trace>> {
        let traces = self.traces.read().map_err(|e| {
            Error::Unknown(format!(
                "Failed to acquire read lock on trace storage: {}",
                e
            ))
        })?;

        let trace = traces.iter().find(|t| t.span_id == span_id).cloned();
        Ok(trace)
    }

    pub fn get_by_name(&self, name: &str) -> Result<Vec<Trace>> {
        let traces = self.traces.read().map_err(|e| {
            Error::Unknown(format!(
                "Failed to acquire read lock on trace storage: {}",
                e
            ))
        })?;

        let filtered = traces.iter().filter(|t| t.name == name).cloned().collect();

        Ok(filtered)
    }

    pub fn get_children(&self, parent_id: &str) -> Result<Vec<Trace>> {
        let traces = self.traces.read().map_err(|e| {
            Error::Unknown(format!(
                "Failed to acquire read lock on trace storage: {}",
                e
            ))
        })?;

        let filtered = traces
            .iter()
            .filter(|t| t.parent_id.as_ref().map_or(false, |id| id == parent_id))
            .cloned()
            .collect();

        Ok(filtered)
    }

    pub fn get_roots(&self) -> Result<Vec<Trace>> {
        let traces = self.traces.read().map_err(|e| {
            Error::Unknown(format!(
                "Failed to acquire read lock on trace storage: {}",
                e
            ))
        })?;

        let filtered = traces.iter().filter(|t| t.is_root()).cloned().collect();

        Ok(filtered)
    }

    pub fn get_by_time_range(
        &self,
        start: DateTime<Utc>,
        end: DateTime<Utc>,
    ) -> Result<Vec<Trace>> {
        let traces = self.traces.read().map_err(|e| {
            Error::Unknown(format!(
                "Failed to acquire read lock on trace storage: {}",
                e
            ))
        })?;

        let filtered = traces
            .iter()
            .filter(|t| t.start_time >= start && t.end_time <= end)
            .cloned()
            .collect();

        Ok(filtered)
    }

    pub fn clear(&self) -> Result<()> {
        let mut traces = self.traces.write().map_err(|e| {
            Error::Unknown(format!(
                "Failed to acquire write lock on trace storage: {}",
                e
            ))
        })?;
        traces.clear();
        Ok(())
    }

    pub fn count(&self) -> Result<usize> {
        let traces = self.traces.read().map_err(|e| {
            Error::Unknown(format!(
                "Failed to acquire read lock on trace storage: {}",
                e
            ))
        })?;
        Ok(traces.len())
    }
}

impl Default for TraceStorage {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone)]
pub struct LogStorage {
    logs: Arc<RwLock<Vec<LogEntry>>>,
}

impl LogStorage {
    pub fn new() -> Self {
        Self {
            logs: Arc::new(RwLock::new(Vec::new())),
        }
    }

    pub fn add(&self, log: LogEntry) -> Result<()> {
        let mut logs = self.logs.write().map_err(|e| {
            Error::Unknown(format!(
                "Failed to acquire write lock on log storage: {}",
                e
            ))
        })?;
        logs.push(log);
        Ok(())
    }

    pub fn get_all(&self) -> Result<Vec<LogEntry>> {
        let logs = self.logs.read().map_err(|e| {
            Error::Unknown(format!("Failed to acquire read lock on log storage: {}", e))
        })?;
        Ok(logs.clone())
    }

    pub fn get_by_level(&self, min_level: LogLevel) -> Result<Vec<LogEntry>> {
        let logs = self.logs.read().map_err(|e| {
            Error::Unknown(format!("Failed to acquire read lock on log storage: {}", e))
        })?;

        let filtered = logs
            .iter()
            .filter(|l| l.is_level_at_least(min_level))
            .cloned()
            .collect();

        Ok(filtered)
    }

    pub fn get_by_source(&self, source: &str) -> Result<Vec<LogEntry>> {
        let logs = self.logs.read().map_err(|e| {
            Error::Unknown(format!("Failed to acquire read lock on log storage: {}", e))
        })?;

        let filtered = logs
            .iter()
            .filter(|l| l.source == source)
            .cloned()
            .collect();

        Ok(filtered)
    }

    pub fn get_by_message_contains(&self, substring: &str) -> Result<Vec<LogEntry>> {
        let logs = self.logs.read().map_err(|e| {
            Error::Unknown(format!("Failed to acquire read lock on log storage: {}", e))
        })?;

        let filtered = logs
            .iter()
            .filter(|l| l.message.contains(substring))
            .cloned()
            .collect();

        Ok(filtered)
    }

    pub fn get_by_time_range(
        &self,
        start: DateTime<Utc>,
        end: DateTime<Utc>,
    ) -> Result<Vec<LogEntry>> {
        let logs = self.logs.read().map_err(|e| {
            Error::Unknown(format!("Failed to acquire read lock on log storage: {}", e))
        })?;

        let filtered = logs
            .iter()
            .filter(|l| l.timestamp >= start && l.timestamp <= end)
            .cloned()
            .collect();

        Ok(filtered)
    }

    pub fn clear(&self) -> Result<()> {
        let mut logs = self.logs.write().map_err(|e| {
            Error::Unknown(format!(
                "Failed to acquire write lock on log storage: {}",
                e
            ))
        })?;
        logs.clear();
        Ok(())
    }

    pub fn count(&self) -> Result<usize> {
        let logs = self.logs.read().map_err(|e| {
            Error::Unknown(format!("Failed to acquire read lock on log storage: {}", e))
        })?;
        Ok(logs.len())
    }
}

impl Default for LogStorage {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Duration;

    #[test]
    fn test_metric_storage_add_and_get() {
        let storage = MetricStorage::new();
        let metric = Metric::new("cpu_usage", 75.5).with_label("host", "server-1");

        assert!(storage.add(metric.clone()).is_ok());

        let all_metrics = storage.get_all().unwrap();
        assert_eq!(all_metrics.len(), 1);
        assert_eq!(all_metrics[0], metric);
    }

    #[test]
    fn test_metric_storage_get_by_name() {
        let storage = MetricStorage::new();

        storage.add(Metric::new("cpu_usage", 75.5)).unwrap();
        storage.add(Metric::new("memory_usage", 42.8)).unwrap();
        storage.add(Metric::new("cpu_usage", 80.2)).unwrap();

        let cpu_metrics = storage.get_by_name("cpu_usage").unwrap();
        assert_eq!(cpu_metrics.len(), 2);
        assert!(cpu_metrics.iter().all(|m| m.name == "cpu_usage"));

        let memory_metrics = storage.get_by_name("memory_usage").unwrap();
        assert_eq!(memory_metrics.len(), 1);
        assert_eq!(memory_metrics[0].name, "memory_usage");
    }

    #[test]
    fn test_metric_storage_get_by_label() {
        let storage = MetricStorage::new();

        storage
            .add(Metric::new("cpu_usage", 75.5).with_label("host", "server-1"))
            .unwrap();
        storage
            .add(Metric::new("memory_usage", 42.8).with_label("host", "server-2"))
            .unwrap();
        storage
            .add(Metric::new("cpu_usage", 80.2).with_label("host", "server-1"))
            .unwrap();

        let server1_metrics = storage.get_by_label("host", "server-1").unwrap();
        assert_eq!(server1_metrics.len(), 2);
        assert!(server1_metrics
            .iter()
            .all(|m| m.has_label_value("host", "server-1")));

        let server2_metrics = storage.get_by_label("host", "server-2").unwrap();
        assert_eq!(server2_metrics.len(), 1);
        assert!(server2_metrics[0].has_label_value("host", "server-2"));
    }

    #[test]
    fn test_trace_storage_add_and_get() {
        let storage = TraceStorage::new();
        let trace = Trace::new("request_handler", 150);

        assert!(storage.add(trace.clone()).is_ok());

        let all_traces = storage.get_all().unwrap();
        assert_eq!(all_traces.len(), 1);
        assert_eq!(all_traces[0], trace);
    }

    #[test]
    fn test_trace_storage_get_by_id() {
        let storage = TraceStorage::new();
        let trace = Trace::new("request_handler", 150);
        let span_id = trace.span_id.clone();

        storage.add(trace.clone()).unwrap();
        storage.add(Trace::new("db_query", 50)).unwrap();

        let found_trace = storage.get_by_id(&span_id).unwrap();
        assert!(found_trace.is_some());
        assert_eq!(found_trace.unwrap(), trace);

        let not_found = storage.get_by_id("nonexistent-id").unwrap();
        assert!(not_found.is_none());
    }

    #[test]
    fn test_trace_storage_get_children() {
        let storage = TraceStorage::new();

        let parent = Trace::new("parent_op", 1000);
        let parent_id = parent.span_id.clone();

        let child1 = Trace::new("child1", 250).with_parent(&parent_id);
        let child2 = Trace::new("child2", 500).with_parent(&parent_id);
        let other = Trace::new("other_op", 750);

        storage.add(parent).unwrap();
        storage.add(child1.clone()).unwrap();
        storage.add(child2.clone()).unwrap();
        storage.add(other).unwrap();

        let children = storage.get_children(&parent_id).unwrap();
        assert_eq!(children.len(), 2);
        assert!(children.contains(&child1));
        assert!(children.contains(&child2));
    }

    #[test]
    fn test_log_storage_add_and_get() {
        let storage = LogStorage::new();
        let log = LogEntry::new("Server started", LogLevel::Info, "app");

        assert!(storage.add(log.clone()).is_ok());

        let all_logs = storage.get_all().unwrap();
        assert_eq!(all_logs.len(), 1);
        assert_eq!(all_logs[0], log);
    }

    #[test]
    fn test_log_storage_get_by_level() {
        let storage = LogStorage::new();

        storage
            .add(LogEntry::new("Debug message", LogLevel::Debug, "app"))
            .unwrap();
        storage
            .add(LogEntry::new("Info message", LogLevel::Info, "app"))
            .unwrap();
        storage
            .add(LogEntry::new("Warning message", LogLevel::Warning, "app"))
            .unwrap();
        storage
            .add(LogEntry::new("Error message", LogLevel::Error, "app"))
            .unwrap();

        let warning_and_above = storage.get_by_level(LogLevel::Warning).unwrap();
        assert_eq!(warning_and_above.len(), 2);
        assert!(warning_and_above
            .iter()
            .all(|l| l.level >= LogLevel::Warning));

        let all_logs = storage.get_by_level(LogLevel::Debug).unwrap();
        assert_eq!(all_logs.len(), 4);
    }

    #[test]
    fn test_log_storage_get_by_source() {
        let storage = LogStorage::new();

        storage
            .add(LogEntry::new("App started", LogLevel::Info, "app"))
            .unwrap();
        storage
            .add(LogEntry::new("DB connected", LogLevel::Info, "database"))
            .unwrap();
        storage
            .add(LogEntry::new("User logged in", LogLevel::Info, "auth"))
            .unwrap();
        storage
            .add(LogEntry::new("Request received", LogLevel::Info, "app"))
            .unwrap();

        let app_logs = storage.get_by_source("app").unwrap();
        assert_eq!(app_logs.len(), 2);
        assert!(app_logs.iter().all(|l| l.source == "app"));
    }

    #[test]
    fn test_log_storage_get_by_message_contains() {
        let storage = LogStorage::new();

        storage
            .add(LogEntry::new(
                "User login: successful",
                LogLevel::Info,
                "auth",
            ))
            .unwrap();
        storage
            .add(LogEntry::new(
                "User logout: successful",
                LogLevel::Info,
                "auth",
            ))
            .unwrap();
        storage
            .add(LogEntry::new(
                "User login: failed",
                LogLevel::Warning,
                "auth",
            ))
            .unwrap();

        let login_logs = storage.get_by_message_contains("login").unwrap();
        assert_eq!(login_logs.len(), 2);
        assert!(login_logs.iter().all(|l| l.message.contains("login")));
    }

    #[test]
    fn test_storage_clear() {
        let metric_storage = MetricStorage::new();
        let trace_storage = TraceStorage::new();
        let log_storage = LogStorage::new();

        metric_storage.add(Metric::new("cpu", 75.0)).unwrap();
        trace_storage.add(Trace::new("request", 100)).unwrap();
        log_storage
            .add(LogEntry::new("Test", LogLevel::Info, "test"))
            .unwrap();

        assert_eq!(metric_storage.count().unwrap(), 1);
        assert_eq!(trace_storage.count().unwrap(), 1);
        assert_eq!(log_storage.count().unwrap(), 1);

        metric_storage.clear().unwrap();
        trace_storage.clear().unwrap();
        log_storage.clear().unwrap();

        assert_eq!(metric_storage.count().unwrap(), 0);
        assert_eq!(trace_storage.count().unwrap(), 0);
        assert_eq!(log_storage.count().unwrap(), 0);
    }

    #[test]
    fn test_storage_time_range() {
        let now = Utc::now();
        let one_hour_ago = now - Duration::hours(1);
        let two_hours_ago = now - Duration::hours(2);

        let metric_storage = MetricStorage::new();
        let trace_storage = TraceStorage::new();
        let log_storage = LogStorage::new();

        metric_storage
            .add(Metric::with_timestamp("recent", 100.0, now))
            .unwrap();
        metric_storage
            .add(Metric::with_timestamp("old", 50.0, two_hours_ago))
            .unwrap();

        trace_storage
            .add(Trace::with_times("recent", now - Duration::minutes(5), now))
            .unwrap();
        trace_storage
            .add(Trace::with_times(
                "old",
                two_hours_ago,
                two_hours_ago + Duration::minutes(5),
            ))
            .unwrap();

        log_storage
            .add(LogEntry::with_timestamp(
                "Recent log",
                LogLevel::Info,
                "test",
                now,
            ))
            .unwrap();
        log_storage
            .add(LogEntry::with_timestamp(
                "Old log",
                LogLevel::Info,
                "test",
                two_hours_ago,
            ))
            .unwrap();

        let recent_metrics = metric_storage.get_by_time_range(one_hour_ago, now).unwrap();
        assert_eq!(recent_metrics.len(), 1);
        assert_eq!(recent_metrics[0].name, "recent");

        let recent_traces = trace_storage.get_by_time_range(one_hour_ago, now).unwrap();
        assert_eq!(recent_traces.len(), 1);
        assert_eq!(recent_traces[0].name, "recent");

        let recent_logs = log_storage.get_by_time_range(one_hour_ago, now).unwrap();
        assert_eq!(recent_logs.len(), 1);
        assert_eq!(recent_logs[0].message, "Recent log");
    }
}
