use crate::error::{Error, Result};
use crate::models::{LogEntry, Metric, Trace};
use crate::templating::renderer::{Block, Renderer, TemplateData};
use crate::templating::template::Template;
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::{Arc, RwLock};

#[derive(Debug, Clone, Default)]
pub struct TemplateContext {
    pub variables: HashMap<String, String>,
    pub metrics: Vec<Metric>,
    pub logs: Vec<LogEntry>,
    pub traces: Vec<Trace>,
    pub data: HashMap<String, serde_json::Value>,
}

impl TemplateContext {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_variable<S: Into<String>>(mut self, name: S, value: S) -> Self {
        self.variables.insert(name.into(), value.into());
        self
    }

    pub fn with_variables(mut self, vars: HashMap<String, String>) -> Self {
        self.variables.extend(vars);
        self
    }

    pub fn with_metrics(mut self, metrics: Vec<Metric>) -> Self {
        self.metrics = metrics;
        self
    }

    pub fn with_logs(mut self, logs: Vec<LogEntry>) -> Self {
        self.logs = logs;
        self
    }

    pub fn with_traces(mut self, traces: Vec<Trace>) -> Self {
        self.traces = traces;
        self
    }

    pub fn with_data<S: Into<String>>(mut self, key: S, value: serde_json::Value) -> Self {
        self.data.insert(key.into(), value);
        self
    }
}

pub struct TemplateEngine {
    template_dir: PathBuf,
    template_cache: Arc<RwLock<HashMap<String, Template>>>,
}

impl TemplateEngine {
    pub fn new<P: AsRef<Path>>(template_dir: P) -> Self {
        let template_dir = template_dir.as_ref().to_path_buf();

        Self {
            template_dir,
            template_cache: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub fn load_template(&self, name: &str) -> Result<Template> {
        {
            let cache = self.template_cache.read().map_err(|e| {
                Error::TemplateError(
                    format!("Failed to acquire read lock on template cache: {}", e).into(),
                )
            })?;

            if let Some(template) = cache.get(name) {
                return Ok(template.clone());
            }
        }

        let template_path = self.template_dir.join(format!("{}.tmpl", name));
        if !template_path.exists() {
            return Err(Error::TemplateError(
                format!("Template '{}' not found", name).into(),
            ));
        }

        let template = Template::from_file(template_path)?;

        {
            let mut cache = self.template_cache.write().map_err(|e| {
                Error::TemplateError(
                    format!("Failed to acquire write lock on template cache: {}", e).into(),
                )
            })?;

            cache.insert(name.to_string(), template.clone());
        }

        Ok(template)
    }

    pub fn clear_cache(&self) -> Result<()> {
        let mut cache = self.template_cache.write().map_err(|e| {
            Error::TemplateError(
                format!("Failed to acquire write lock on template cache: {}", e).into(),
            )
        })?;

        cache.clear();
        Ok(())
    }

    pub fn render<R: Renderer>(
        &self,
        template_name: &str,
        context: &TemplateContext,
        renderer: &R,
    ) -> Result<String> {
        let template = self.load_template(template_name)?;

        let processed_blocks = self.process_blocks(&template.blocks, context)?;

        let template_data = TemplateData {
            blocks: processed_blocks,
            template_name: template.name.clone(),
        };

        let rendered_content = renderer.render_template(&template_data)?;

        let final_content =
            self.substitute_variables_in_content(&rendered_content, &context.variables);

        Ok(final_content)
    }

    fn process_blocks(&self, blocks: &[Block], context: &TemplateContext) -> Result<Vec<Block>> {
        let mut processed_blocks = Vec::new();

        for block in blocks {
            match block {
                Block::Raw(content) if content.trim() == "@metrics" => {
                    processed_blocks.push(Block::Raw(format!(
                        "<!-- Metrics: {} -->",
                        context.metrics.len()
                    )));
                    for metric in &context.metrics {
                        let trend = metric
                            .get_label("trend")
                            .and_then(|t| t.parse::<f64>().ok());
                        let unit = metric.get_label("unit").map(|s| s.to_string());

                        processed_blocks.push(Block::Metric {
                            name: metric.name.clone(),
                            value: metric.value.to_string(),
                            unit,
                            trend,
                        });
                    }
                }

                Block::Raw(content) if content.trim() == "@logs" => {
                    processed_blocks
                        .push(Block::Raw(format!("<!-- Logs: {} -->", context.logs.len())));
                    if context.logs.is_empty() {
                        processed_blocks.push(Block::Paragraph("No logs available.".to_string()));
                    } else {
                        let headers = vec![
                            "Timestamp".to_string(),
                            "Level".to_string(),
                            "Source".to_string(),
                            "Message".to_string(),
                        ];

                        let rows: Vec<Vec<String>> = context
                            .logs
                            .iter()
                            .map(|log| {
                                vec![
                                    log.timestamp.to_rfc3339(),
                                    log.level.to_string(),
                                    log.source.clone(),
                                    log.message.clone(),
                                ]
                            })
                            .collect();

                        processed_blocks.push(Block::Table { headers, rows });
                    }
                }

                Block::Raw(content) if content.trim() == "@traces" => {
                    processed_blocks.push(Block::Raw(format!(
                        "<!-- Traces: {} -->",
                        context.traces.len()
                    )));
                    for trace in &context.traces {
                        let status = trace
                            .get_metadata("status")
                            .cloned()
                            .unwrap_or_else(|| "unknown".to_string());

                        processed_blocks.push(Block::Trace {
                            name: trace.name.clone(),
                            duration_ms: trace.duration_ms,
                            start_time: trace.start_time.to_rfc3339(),
                            status,
                            metadata: trace.metadata.clone(),
                        });
                    }
                }

                Block::Container(nested_blocks) => {
                    let processed_nested = self.process_blocks(nested_blocks, context)?;
                    processed_blocks.push(Block::Container(processed_nested));
                }

                Block::Frame { title, content } => {
                    let processed_content = self.process_blocks(content, context)?;
                    processed_blocks.push(Block::Frame {
                        title: title.clone(),
                        content: processed_content,
                    });
                }

                Block::Output(nested_blocks) => {
                    let processed_nested = self.process_blocks(nested_blocks, context)?;
                    processed_blocks.push(Block::Output(processed_nested));
                }

                _ => processed_blocks.push(block.clone()),
            }
        }

        Ok(processed_blocks)
    }

    fn substitute_variables_in_content(
        &self,
        content: &str,
        variables: &HashMap<String, String>,
    ) -> String {
        let mut result = content.to_string();

        for (name, value) in variables {
            let pattern = format!("[[{}]]", name);
            result = result.replace(&pattern, value);
        }

        result
    }

    pub fn write_output<P: AsRef<Path>>(
        &self,
        html_content: &str,
        text_content: &str,
        output_dir: P,
        base_name: &str,
    ) -> Result<()> {
        let output_dir = output_dir.as_ref();

        if !output_dir.exists() {
            fs::create_dir_all(output_dir).map_err(|e| {
                Error::TemplateError(format!("Failed to create output directory: {}", e).into())
            })?;
        }

        let html_path = output_dir.join(format!("{}.html", base_name));
        let text_path = output_dir.join(format!("{}.txt", base_name));

        fs::write(&html_path, html_content).map_err(|e| {
            Error::TemplateError(format!("Failed to write HTML output: {}", e).into())
        })?;

        fs::write(&text_path, text_content).map_err(|e| {
            Error::TemplateError(format!("Failed to write text output: {}", e).into())
        })?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{LogEntry, LogLevel, Metric, Trace};
    use crate::templating::html_renderer::HtmlRenderer;
    use crate::templating::text_renderer::TextRenderer;
    use chrono::Utc;
    use serial_test::serial;
    use std::fs;
    use std::io::Write;
    use tempfile::{tempdir, NamedTempFile};

    fn create_test_template() -> (NamedTempFile, String) {
        let mut temp_file = NamedTempFile::new().unwrap();
        let template_content = r#"@heading{1}{Dashboard Overview}

@paragraph{Welcome to the system dashboard. Current status as of [[current_time]].}

@command{system status}

@output{
  @metrics
}

@frame{Recent Logs}{
  @logs
}

@frame{System Performance}{
  @paragraph{System performance metrics show normal operation with slight increase in response time.}
  
  @traces
}
"#;

        write!(temp_file, "{}", template_content).unwrap();
        (temp_file, "dashboard".to_string())
    }

    fn create_test_context() -> TemplateContext {
        let now = Utc::now();

        let mut cpu_labels = HashMap::new();
        cpu_labels.insert("unit".to_string(), "%".to_string());
        cpu_labels.insert("trend".to_string(), "+2.3".to_string());

        let mut mem_labels = HashMap::new();
        mem_labels.insert("unit".to_string(), "GB".to_string());
        mem_labels.insert("trend".to_string(), "-0.5".to_string());

        let metrics = vec![
            Metric {
                name: "CPU Usage".to_string(),
                value: 78.5,
                timestamp: now,
                labels: cpu_labels,
            },
            Metric {
                name: "Memory".to_string(),
                value: 4.2,
                timestamp: now,
                labels: mem_labels,
            },
        ];

        let logs = vec![
            LogEntry::new("Server started", LogLevel::Info, "app"),
            LogEntry::new("Connection established", LogLevel::Debug, "network"),
            LogEntry::new("Warning: High CPU usage", LogLevel::Warning, "monitor"),
        ];

        let mut trace_metadata = HashMap::new();
        trace_metadata.insert("status".to_string(), "completed".to_string());

        let traces = vec![Trace::new("API Request", 157).with_metadata_map(trace_metadata)];

        let mut variables = HashMap::new();
        variables.insert("current_time".to_string(), now.to_rfc3339());

        TemplateContext::new()
            .with_variables(variables)
            .with_metrics(metrics)
            .with_logs(logs)
            .with_traces(traces)
    }

    #[test]
    #[serial]
    fn test_template_engine_rendering() {
        let (temp_file, template_name) = create_test_template();
        let template_dir = temp_file.path().parent().unwrap();

        let template_path = template_dir.join(format!("{}.tmpl", template_name));
        fs::copy(temp_file.path(), &template_path).unwrap();

        let engine = TemplateEngine::new(template_dir);
        let context = create_test_context();

        let html_renderer = HtmlRenderer::new();
        let text_renderer = TextRenderer::new();

        let html_result = engine.render(&template_name, &context, &html_renderer);
        assert!(html_result.is_ok());
        let html = html_result.unwrap();

        assert!(html.contains("<!DOCTYPE html>"));
        assert!(html.contains("Dashboard Overview"));
        assert!(html.contains("CPU Usage"));
        assert!(html.contains("Memory"));
        assert!(html.contains("Server started"));
        assert!(html.contains("API Request"));

        let text_result = engine.render(&template_name, &context, &text_renderer);
        assert!(text_result.is_ok());
        let text = text_result.unwrap();

        assert!(text.contains("Dashboard Overview"));
        assert!(text.contains("=================="));
        assert!(text.contains("CPU Usage"));
        assert!(text.contains("Memory"));
        assert!(text.contains("Server started"));
        assert!(text.contains("API Request"));
    }

    #[test]
    #[serial]
    fn test_special_directives_processing() {
        let (temp_file, template_name) = create_test_template();
        let template_dir = temp_file.path().parent().unwrap();

        let template_path = template_dir.join(format!("{}.tmpl", template_name));
        fs::copy(temp_file.path(), &template_path).unwrap();

        let engine = TemplateEngine::new(template_dir);
        let context = create_test_context();

        let template = engine.load_template(&template_name).unwrap();
        let processed_blocks = engine.process_blocks(&template.blocks, &context).unwrap();

        let mut found_metrics = false;
        let mut found_logs = false;
        let mut found_traces = false;

        fn find_special_blocks(
            blocks: &[Block],
            metrics: &mut bool,
            logs: &mut bool,
            traces: &mut bool,
        ) {
            for block in blocks {
                match block {
                    Block::Metric { .. } => *metrics = true,
                    Block::Table { .. } => *logs = true,
                    Block::Trace { .. } => *traces = true,
                    Block::Container(nested) | Block::Output(nested) => {
                        find_special_blocks(nested, metrics, logs, traces);
                    }
                    Block::Frame { content, .. } => {
                        find_special_blocks(content, metrics, logs, traces);
                    }
                    _ => {}
                }
            }
        }

        find_special_blocks(
            &processed_blocks,
            &mut found_metrics,
            &mut found_logs,
            &mut found_traces,
        );

        assert!(found_metrics, "No metric blocks found after processing");
        assert!(found_logs, "No log blocks found after processing");
        assert!(found_traces, "No trace blocks found after processing");
    }

    #[test]
    #[serial]
    fn test_output_writing() {
        let output_dir = tempdir().unwrap();
        let engine = TemplateEngine::new(output_dir.path());

        let html_content = "<html><body>Test HTML</body></html>";
        let text_content = "Test Text Content";

        let result =
            engine.write_output(html_content, text_content, output_dir.path(), "test_output");

        assert!(result.is_ok(), "Failed to write output: {:?}", result.err());

        let html_path = output_dir.path().join("test_output.html");
        let text_path = output_dir.path().join("test_output.txt");

        assert!(html_path.exists(), "HTML file was not created");
        assert!(text_path.exists(), "Text file was not created");

        let html_read = fs::read_to_string(html_path).unwrap();
        let text_read = fs::read_to_string(text_path).unwrap();

        assert_eq!(html_read, html_content);
        assert_eq!(text_read, text_content);
    }

    #[test]
    #[serial]
    fn test_template_cache() {
        let (temp_file, template_name) = create_test_template();
        let template_dir = temp_file.path().parent().unwrap();

        let template_path = template_dir.join(format!("{}.tmpl", template_name));
        fs::copy(temp_file.path(), &template_path).unwrap();

        let engine = TemplateEngine::new(template_dir);

        let template1 = engine.load_template(&template_name).unwrap();

        let template2 = engine.load_template(&template_name).unwrap();

        assert_eq!(template1.name, template2.name);
        assert_eq!(template1.content, template2.content);

        let mut file = fs::OpenOptions::new()
            .write(true)
            .truncate(true)
            .open(&template_path)
            .unwrap();

        write!(file, "@heading{{1}}{{Modified Template}}").unwrap();

        let template3 = engine.load_template(&template_name).unwrap();
        assert_eq!(template1.content, template3.content);

        engine.clear_cache().unwrap();
        let template4 = engine.load_template(&template_name).unwrap();
        assert_ne!(template1.content, template4.content);
        assert!(template4.content.contains("Modified Template"));
    }
}
