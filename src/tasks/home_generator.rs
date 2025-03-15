use crate::error::Result;
use crate::models::{LogEntry, Metric, Trace};
use crate::scheduler::Task;
use crate::storage::{LogStorage, MetricStorage, TraceStorage};
use crate::templating::{HtmlRenderer, TemplateContext, TemplateEngine, TextRenderer};
use async_trait::async_trait;
use chrono::{Duration, Utc};
use log::{info, warn};
use std::path::Path;
use std::sync::Arc;

pub struct HomeGeneratorTask {
    template_engine: Arc<TemplateEngine>,
    metric_storage: Arc<MetricStorage>,
    trace_storage: Arc<TraceStorage>,
    log_storage: Arc<LogStorage>,
    output_dir: String,
}

impl HomeGeneratorTask {
    pub fn new(
        template_engine: Arc<TemplateEngine>,
        metric_storage: Arc<MetricStorage>,
        trace_storage: Arc<TraceStorage>,
        log_storage: Arc<LogStorage>,
        output_dir: String,
    ) -> Self {
        Self {
            template_engine,
            metric_storage,
            trace_storage,
            log_storage,
            output_dir,
        }
    }

    async fn generate_site(&self) -> Result<()> {
        info!("Generating dashboard content");
        let (html_content, text_content) = self.generate_dashboard().await?;

        let output_path = Path::new(&self.output_dir);
        if !output_path.exists() {
            std::fs::create_dir_all(output_path)?;
        }

        self.template_engine.write_output(
            &html_content,
            &text_content,
            &self.output_dir,
            "index",
        )?;

        info!("Dashboard generation completed");
        Ok(())
    }

    async fn generate_dashboard(&self) -> Result<(String, String)> {
        let now = Utc::now();
        let one_hour_ago = now - Duration::hours(1);

        let metrics = match self.metric_storage.get_by_time_range(one_hour_ago, now) {
            Ok(metrics) => metrics,
            Err(e) => {
                warn!("Failed to retrieve metrics: {}", e);
                Vec::new()
            }
        };

        let traces = match self.trace_storage.get_by_time_range(one_hour_ago, now) {
            Ok(traces) => traces,
            Err(e) => {
                warn!("Failed to retrieve traces: {}", e);
                Vec::new()
            }
        };

        let logs = match self.log_storage.get_by_time_range(one_hour_ago, now) {
            Ok(logs) => logs,
            Err(e) => {
                warn!("Failed to retrieve logs: {}", e);
                Vec::new()
            }
        };

        let template_context = self.create_context(metrics, traces, logs);

        let html_renderer = HtmlRenderer::new();
        let text_renderer = TextRenderer::new();

        let html_content =
            self.template_engine
                .render("dashboard", &template_context, &html_renderer)?;
        let text_content =
            self.template_engine
                .render("dashboard", &template_context, &text_renderer)?;

        Ok((html_content, text_content))
    }

    fn create_context(
        &self,
        metrics: Vec<Metric>,
        traces: Vec<Trace>,
        logs: Vec<LogEntry>,
    ) -> TemplateContext {
        let mut context = TemplateContext::new();

        let current_time = Utc::now().to_rfc3339();
        context = context.with_variable("current_time", &current_time);

        context = context.with_metrics(metrics);
        context = context.with_traces(traces);
        context = context.with_logs(logs);

        context = context.with_variable("hostname", "maxteibel-server");

        let metric_count = format!("{}", context.metrics.len());
        context = context.with_variable("metric_count", &metric_count);

        let trace_count = format!("{}", context.traces.len());
        context = context.with_variable("trace_count", &trace_count);

        let log_count = format!("{}", context.logs.len());
        context = context.with_variable("log_count", &log_count);

        context
    }
}

#[async_trait]
impl Task for HomeGeneratorTask {
    fn name(&self) -> &str {
        "HomeGenerator"
    }

    async fn execute(&self) -> Result<()> {
        self.generate_site().await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::LogLevel;
    use crate::storage::{LogStorage, MetricStorage, TraceStorage};
    use tempfile::TempDir;

    #[tokio::test]
    async fn test_home_generator_task() {
        let template_dir = TempDir::new().unwrap();
        let output_dir = TempDir::new().unwrap();

        let template_content = r#"@heading{1}{Dashboard}
        
Welcome to the system dashboard. Current status as of @var{current_time}.

@command{system status}
@output{
  @metrics
}

@frame{Recent Logs}{
  @logs
}

@frame{Recent Traces}{
  @traces
}"#;

        std::fs::write(template_dir.path().join("dashboard.tmpl"), template_content).unwrap();

        // Create template engine
        let template_engine = Arc::new(TemplateEngine::new(template_dir.path()));

        // Create storages
        let metric_storage = Arc::new(MetricStorage::new());
        let trace_storage = Arc::new(TraceStorage::new());
        let log_storage = Arc::new(LogStorage::new());

        // Add sample data
        let metric = Metric::new("CPU Usage", 75.5)
            .with_label("host", "test-server")
            .with_label("unit", "%");
        metric_storage.add(metric).unwrap();

        let trace = Trace::new("API Request", 150).with_metadata("endpoint", "/api/status");
        trace_storage.add(trace).unwrap();

        let log = LogEntry::new("Server started", LogLevel::Info, "app");
        log_storage.add(log).unwrap();

        // Create task
        let task = HomeGeneratorTask::new(
            template_engine,
            metric_storage,
            trace_storage,
            log_storage,
            output_dir.path().to_string_lossy().to_string(),
        );

        // Execute task
        task.execute().await.unwrap();

        // Check if files were created
        let html_path = output_dir.path().join("index.html");
        let text_path = output_dir.path().join("index.txt");

        assert!(html_path.exists(), "HTML file was not created");
        assert!(text_path.exists(), "Text file was not created");

        // Check content
        let html_content = std::fs::read_to_string(&html_path).unwrap();
        let text_content = std::fs::read_to_string(&text_path).unwrap();

        // Verify HTML content
        assert!(html_content.contains("<!DOCTYPE html>"));
        assert!(html_content.contains("Dashboard"));
        assert!(html_content.contains("CPU Usage"));
        assert!(html_content.contains("Server started"));

        // Verify text content
        assert!(text_content.contains("Dashboard"));
        assert!(text_content.contains("CPU Usage"));
        assert!(text_content.contains("Server started"));

        // Clean up
        let _ = std::fs::remove_file(html_path);
        let _ = std::fs::remove_file(text_path);
    }
}
