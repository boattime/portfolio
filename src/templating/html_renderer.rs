use std::collections::HashMap;

use crate::error::Result;
use crate::models::{LogEntry, Metric, Trace};
use crate::templating::renderer::{Block, Renderer, TemplateData};

pub struct HtmlRenderer {
    pub additional_classes: Vec<String>,
    pub include_inline_css: bool,
}

impl HtmlRenderer {
    pub fn new() -> Self {
        Self {
            additional_classes: Vec::new(),
            include_inline_css: true,
        }
    }

    pub fn with_classes(mut self, classes: Vec<String>) -> Self {
        self.additional_classes = classes;
        self
    }

    pub fn with_inline_css(mut self, include: bool) -> Self {
        self.include_inline_css = include;
        self
    }

    fn get_terminal_css(&self) -> &str {
        r#"
        .terminal {
            background-color: #1e1e1e;
            color: #f0f0f0;
            font-family: 'Courier New', monospace;
            padding: 1rem;
            border-radius: 0.5rem;
            overflow: auto;
            line-height: 1.5;
            max-width: 100%;
            box-sizing: border-box;
        }
        
        .terminal-command {
            color: #63c8ff;
            margin: 0.5rem 0;
        }
        
        .terminal-command::before {
            content: '$ ';
            color: #63c8ff;
        }
        
        .terminal-output {
            margin: 0.5rem 0 1.5rem 0;
            padding-left: 0.5rem;
            border-left: 2px solid #3a3a3a;
        }
        
        .terminal-frame {
            border: 1px solid #3a3a3a;
            padding: 0.5rem;
            margin: 0.5rem 0;
            border-radius: 0.3rem;
        }
        
        .terminal-frame-title {
            background-color: #3a3a3a;
            padding: 0.3rem 0.5rem;
            margin: -0.5rem -0.5rem 0.5rem -0.5rem;
            border-radius: 0.3rem 0.3rem 0 0;
            font-weight: bold;
        }
        
        .terminal-metric {
            display: flex;
            justify-content: space-between;
            padding: 0.3rem 0;
        }
        
        .terminal-metric-name {
            font-weight: bold;
        }
        
        .terminal-metric-value {
            color: #63c8ff;
        }
        
        .terminal-log {
            padding: 0.2rem 0;
        }
        
        .terminal-log-debug {
            color: #9e9e9e;
        }
        
        .terminal-log-info {
            color: #63c8ff;
        }
        
        .terminal-log-warning {
            color: #ffac35;
        }
        
        .terminal-log-error {
            color: #ff5b5b;
        }
        
        .terminal-table {
            border-collapse: collapse;
            width: 100%;
            margin: 0.5rem 0;
        }
        
        .terminal-table th {
            text-align: left;
            padding: 0.3rem;
            border-bottom: 1px solid #3a3a3a;
            color: #63c8ff;
        }
        
        .terminal-table td {
            padding: 0.3rem;
            border-bottom: 1px solid #2a2a2a;
        }
        
        .terminal-trace {
            padding: 0.3rem 0;
        }
        
        .terminal-trace-name {
            font-weight: bold;
        }
        
        .terminal-trace-duration {
            color: #63c8ff;
        }
        
        .terminal-trend-up::after {
            content: ' ▲';
            color: #4caf50;
        }
        
        .terminal-trend-down::after {
            content: ' ▼';
            color: #ff5b5b;
        }
        
        @media (max-width: 768px) {
            .terminal {
                padding: 0.5rem;
            }
            
            .terminal-table {
                font-size: 0.9rem;
            }
        }
        "#
    }

    fn escape_html(&self, text: &str) -> String {
        text.replace('&', "&amp;")
            .replace('<', "&lt;")
            .replace('>', "&gt;")
            .replace('"', "&quot;")
            .replace('\'', "&#39;")
    }

    fn get_trend_class(&self, trend: Option<f64>) -> &'static str {
        match trend {
            Some(t) if t > 0.0 => "terminal-trend-up",
            Some(t) if t < 0.0 => "terminal-trend-down",
            _ => "",
        }
    }
}

impl Default for HtmlRenderer {
    fn default() -> Self {
        Self::new()
    }
}

impl Renderer for HtmlRenderer {
    fn render_heading(&self, level: usize, text: &str) -> Result<String> {
        let level = level.min(6).max(1);
        let escaped_text = self.escape_html(text);
        Ok(format!(
            "<h{0} class=\"terminal-heading terminal-heading-{0}\">{1}</h{0}>",
            level, escaped_text
        ))
    }

    fn render_paragraph(&self, text: &str) -> Result<String> {
        let escaped_text = self.escape_html(text);
        Ok(format!(
            "<p class=\"terminal-paragraph\">{}</p>",
            escaped_text
        ))
    }

    fn render_command_prompt(&self, command: &str) -> Result<String> {
        let escaped_command = self.escape_html(command);
        Ok(format!(
            "<div class=\"terminal-command\">{}</div>",
            escaped_command
        ))
    }

    fn render_output(&self, blocks: &[Block]) -> Result<String> {
        let content = self.render_blocks(blocks)?;
        Ok(format!("<div class=\"terminal-output\">{}</div>", content))
    }

    fn render_frame(&self, title: Option<&str>, content: &str) -> Result<String> {
        let title_html = if let Some(title_text) = title {
            let escaped_title = self.escape_html(title_text);
            format!(
                "<div class=\"terminal-frame-title\">{}</div>",
                escaped_title
            )
        } else {
            String::new()
        };

        Ok(format!(
            "<div class=\"terminal-frame\">{}{}</div>",
            title_html, content
        ))
    }

    fn render_metric(
        &self,
        name: &str,
        value: &str,
        unit: Option<&str>,
        trend: Option<f64>,
    ) -> Result<String> {
        let escaped_name = self.escape_html(name);
        let escaped_value = self.escape_html(value);
        let trend_class = self.get_trend_class(trend);

        let value_with_unit = if let Some(u) = unit {
            format!("{} {}", escaped_value, self.escape_html(u))
        } else {
            escaped_value
        };

        Ok(format!(
            "<div class=\"terminal-metric\">
                <span class=\"terminal-metric-name\">{}</span>
                <span class=\"terminal-metric-value {}\">{}
                </span>
            </div>",
            escaped_name, trend_class, value_with_unit
        ))
    }

    fn render_log_entry(
        &self,
        message: &str,
        level: &str,
        timestamp: Option<&str>,
        source: Option<&str>,
    ) -> Result<String> {
        let escaped_message = self.escape_html(message);
        let log_level_class = match level.to_uppercase().as_str() {
            "DEBUG" => "terminal-log-debug",
            "INFO" => "terminal-log-info",
            "WARNING" | "WARN" => "terminal-log-warning",
            "ERROR" => "terminal-log-error",
            _ => "terminal-log-info",
        };

        let prefix = match (timestamp, source) {
            (Some(ts), Some(src)) => {
                format!("[{}] [{}] ", self.escape_html(ts), self.escape_html(src))
            }
            (Some(ts), None) => format!("[{}] ", self.escape_html(ts)),
            (None, Some(src)) => format!("[{}] ", self.escape_html(src)),
            (None, None) => String::new(),
        };

        Ok(format!(
            "<div class=\"terminal-log {}\">
                <span class=\"terminal-log-prefix\">{}</span>
                <span class=\"terminal-log-message\">{}</span>
            </div>",
            log_level_class, prefix, escaped_message
        ))
    }

    fn render_table(&self, headers: &[String], rows: &[Vec<String>]) -> Result<String> {
        let header_cells = headers
            .iter()
            .map(|h| format!("<th>{}</th>", self.escape_html(h)))
            .collect::<Vec<_>>()
            .join("");

        let header_row = if !headers.is_empty() {
            format!("<tr>{}</tr>", header_cells)
        } else {
            String::new()
        };

        let table_rows = rows
            .iter()
            .map(|row| {
                let cells = row
                    .iter()
                    .map(|c| format!("<td>{}</td>", self.escape_html(c)))
                    .collect::<Vec<_>>()
                    .join("");
                format!("<tr>{}</tr>", cells)
            })
            .collect::<Vec<_>>()
            .join("");

        Ok(format!(
            "<table class=\"terminal-table\">
                <thead>{}</thead>
                <tbody>{}</tbody>
            </table>",
            header_row, table_rows
        ))
    }

    fn render_trace(
        &self,
        name: &str,
        duration_ms: u64,
        start_time: &str,
        status: &str,
        metadata: &HashMap<String, String>,
    ) -> Result<String> {
        let escaped_name = self.escape_html(name);
        let escaped_status = self.escape_html(status);
        let escaped_start_time = self.escape_html(start_time);

        let metadata_html = if !metadata.is_empty() {
            let metadata_items = metadata
                .iter()
                .map(|(k, v)| {
                    format!(
                        "<span class=\"terminal-trace-metadata-item\">
                            <span class=\"terminal-trace-metadata-key\">{}</span>: 
                            <span class=\"terminal-trace-metadata-value\">{}</span>
                        </span>",
                        self.escape_html(k),
                        self.escape_html(v)
                    )
                })
                .collect::<Vec<_>>()
                .join(", ");

            format!(
                "<div class=\"terminal-trace-metadata\">{}</div>",
                metadata_items
            )
        } else {
            String::new()
        };

        Ok(format!(
            "<div class=\"terminal-trace\">
                <div class=\"terminal-trace-header\">
                    <span class=\"terminal-trace-name\">{}</span>
                    <span class=\"terminal-trace-duration\">{} ms</span>
                </div>
                <div class=\"terminal-trace-details\">
                    Started: {}, Status: {}
                </div>
                {}
            </div>",
            escaped_name, duration_ms, escaped_start_time, escaped_status, metadata_html
        ))
    }

    fn render_raw(&self, content: &str) -> Result<String> {
        Ok(content.to_string())
    }

    fn render_template(&self, template_data: &TemplateData) -> Result<String> {
        let content = self.render_blocks(&template_data.blocks)?;

        let class_list = if self.additional_classes.is_empty() {
            "terminal".to_string()
        } else {
            format!("terminal {}", self.additional_classes.join(" "))
        };

        let style_tag = if self.include_inline_css {
            format!("<style>{}</style>", self.get_terminal_css())
        } else {
            String::new()
        };

        Ok(format!(
            "<!DOCTYPE html>
            <html lang=\"en\">
            <head>
                <meta charset=\"UTF-8\">
                <meta name=\"viewport\" content=\"width=device-width, initial-scale=1.0\">
                <title>{}</title>
                {}
            </head>
            <body>
                <div class=\"{}\">{}
                </div>
            </body>
            </html>",
            template_data.template_name, style_tag, class_list, content
        ))
    }

    fn render_metrics(&self, metrics: &[Metric]) -> Result<String> {
        let blocks: Vec<Block> = metrics
            .iter()
            .map(|m| {
                let trend = m.get_label("trend").and_then(|t| t.parse::<f64>().ok());

                let unit = m.get_label("unit").map(|s| s.as_str());

                Block::Metric {
                    name: m.name.clone(),
                    value: m.value.to_string(),
                    unit: unit.map(|s| s.to_string()),
                    trend,
                }
            })
            .collect();

        self.render_blocks(&blocks)
    }

    fn render_logs(&self, logs: &[LogEntry]) -> Result<String> {
        let blocks: Vec<Block> = logs
            .iter()
            .map(|log| Block::LogEntry {
                message: log.message.clone(),
                level: log.level.to_string(),
                timestamp: Some(log.timestamp.to_rfc3339()),
                source: Some(log.source.clone()),
            })
            .collect();

        self.render_blocks(&blocks)
    }

    fn render_traces(&self, traces: &[Trace]) -> Result<String> {
        if traces.is_empty() {
            return Ok(
                "<div class=\"terminal-empty-message\">No traces available</div>".to_string(),
            );
        }

        let headers = vec![
            "Name".to_string(),
            "Duration".to_string(),
            "Started".to_string(),
            "Status".to_string(),
        ];

        let rows: Vec<Vec<String>> = traces
            .iter()
            .map(|trace| {
                let status = trace
                    .get_metadata("status")
                    .cloned()
                    .unwrap_or_else(|| "unknown".to_string());

                vec![
                    trace.name.clone(),
                    format!("{} ms", trace.duration_ms),
                    trace.start_time.to_rfc3339(),
                    status,
                ]
            })
            .collect();

        self.render_table(&headers, &rows)
    }
}

#[cfg(test)]
mod tests {
    use chrono::Utc;
    use std::collections::HashMap;

    use crate::models::{LogEntry, LogLevel, Metric, Trace};
    use crate::templating::html_renderer::HtmlRenderer;
    use crate::templating::renderer::{Block, Renderer, TemplateData};

    fn contains(haystack: &str, needle: &str) -> bool {
        haystack.contains(needle)
    }

    #[test]
    fn test_render_heading() {
        let renderer = HtmlRenderer::new();
        let result = renderer.render_heading(1, "Test Heading").unwrap();

        assert!(contains(&result, "<h1"));
        assert!(contains(&result, "Test Heading"));
        assert!(contains(&result, "terminal-heading"));
    }

    #[test]
    fn test_render_paragraph() {
        let renderer = HtmlRenderer::new();
        let result = renderer
            .render_paragraph("This is a test paragraph.")
            .unwrap();

        assert!(contains(&result, "<p"));
        assert!(contains(&result, "This is a test paragraph."));
        assert!(contains(&result, "terminal-paragraph"));
    }

    #[test]
    fn test_render_command_prompt() {
        let renderer = HtmlRenderer::new();
        let result = renderer.render_command_prompt("ls -la").unwrap();

        assert!(contains(&result, "terminal-command"));
        assert!(contains(&result, "ls -la"));
    }

    #[test]
    fn test_render_output() {
        let renderer = HtmlRenderer::new();
        let blocks = vec![Block::Paragraph("Output content".to_string())];

        let result = renderer.render_output(&blocks).unwrap();

        assert!(contains(&result, "terminal-output"));
        assert!(contains(&result, "Output content"));
    }

    #[test]
    fn test_render_frame() {
        let renderer = HtmlRenderer::new();

        let result_with_title = renderer
            .render_frame(Some("Frame Title"), "Frame content")
            .unwrap();
        assert!(contains(&result_with_title, "terminal-frame"));
        assert!(contains(&result_with_title, "terminal-frame-title"));
        assert!(contains(&result_with_title, "Frame Title"));
        assert!(contains(&result_with_title, "Frame content"));

        let result_without_title = renderer.render_frame(None, "Frame content").unwrap();
        assert!(contains(&result_without_title, "terminal-frame"));
        assert!(!contains(&result_without_title, "terminal-frame-title"));
        assert!(contains(&result_without_title, "Frame content"));
    }

    #[test]
    fn test_render_metric() {
        let renderer = HtmlRenderer::new();

        let result = renderer
            .render_metric("CPU Usage", "85.5", None, None)
            .unwrap();
        assert!(contains(&result, "terminal-metric"));
        assert!(contains(&result, "CPU Usage"));
        assert!(contains(&result, "85.5"));

        let result_with_unit = renderer
            .render_metric("Memory", "4.2", Some("GB"), None)
            .unwrap();
        assert!(contains(&result_with_unit, "Memory"));
        assert!(contains(&result_with_unit, "4.2 GB"));

        let result_positive = renderer
            .render_metric("Requests", "150", None, Some(0.5))
            .unwrap();
        assert!(contains(&result_positive, "Requests"));
        assert!(contains(&result_positive, "terminal-trend-up"));

        let result_negative = renderer
            .render_metric("Errors", "10", None, Some(-0.3))
            .unwrap();
        assert!(contains(&result_negative, "Errors"));
        assert!(contains(&result_negative, "terminal-trend-down"));
    }

    #[test]
    fn test_render_log_entry() {
        let renderer = HtmlRenderer::new();

        let result = renderer
            .render_log_entry("Application started", "INFO", None, None)
            .unwrap();

        assert!(contains(&result, "terminal-log"));
        assert!(contains(&result, "terminal-log-info"));
        assert!(contains(&result, "Application started"));

        let result_full = renderer
            .render_log_entry(
                "Database error",
                "ERROR",
                Some("2025-03-15T12:34:56Z"),
                Some("db_module"),
            )
            .unwrap();

        assert!(contains(&result_full, "terminal-log-error"));
        assert!(contains(&result_full, "Database error"));
        assert!(contains(&result_full, "2025-03-15T12:34:56Z"));
        assert!(contains(&result_full, "db_module"));
    }

    #[test]
    fn test_render_table() {
        let renderer = HtmlRenderer::new();
        let headers = vec![
            "Name".to_string(),
            "Value".to_string(),
            "Status".to_string(),
        ];

        let rows = vec![
            vec![
                "Server 1".to_string(),
                "85.5%".to_string(),
                "OK".to_string(),
            ],
            vec![
                "Server 2".to_string(),
                "92.1%".to_string(),
                "Warning".to_string(),
            ],
        ];

        let result = renderer.render_table(&headers, &rows).unwrap();

        assert!(contains(&result, "terminal-table"));
        assert!(contains(&result, "<thead>"));
        assert!(contains(&result, "<tbody>"));
        assert!(contains(&result, "Name"));
        assert!(contains(&result, "Value"));
        assert!(contains(&result, "Status"));
        assert!(contains(&result, "Server 1"));
        assert!(contains(&result, "85.5%"));
        assert!(contains(&result, "OK"));
        assert!(contains(&result, "Server 2"));
        assert!(contains(&result, "92.1%"));
        assert!(contains(&result, "Warning"));
    }

    #[test]
    fn test_render_trace() {
        let renderer = HtmlRenderer::new();
        let mut metadata = HashMap::new();
        metadata.insert("endpoint".to_string(), "/api/users".to_string());
        metadata.insert("method".to_string(), "GET".to_string());

        let result = renderer
            .render_trace(
                "Process Request",
                120,
                "2025-03-15T12:34:56Z",
                "completed",
                &metadata,
            )
            .unwrap();

        assert!(contains(&result, "terminal-trace"));
        assert!(contains(&result, "Process Request"));
        assert!(contains(&result, "120 ms"));
        assert!(contains(&result, "2025-03-15T12:34:56Z"));
        assert!(contains(&result, "completed"));
        assert!(contains(&result, "endpoint"));
        assert!(contains(&result, "/api/users"));
        assert!(contains(&result, "method"));
        assert!(contains(&result, "GET"));
    }

    #[test]
    fn test_render_block() {
        let renderer = HtmlRenderer::new();

        let heading_block = Block::Heading {
            level: 1,
            text: "Test Heading".to_string(),
        };

        let heading_result = renderer.render_block(&heading_block).unwrap();
        assert!(contains(&heading_result, "Test Heading"));
        assert!(contains(&heading_result, "<h1"));

        let container_block = Block::Container(vec![
            Block::Paragraph("First paragraph".to_string()),
            Block::Paragraph("Second paragraph".to_string()),
        ]);

        let container_result = renderer.render_block(&container_block).unwrap();
        assert!(contains(&container_result, "First paragraph"));
        assert!(contains(&container_result, "Second paragraph"));
    }

    #[test]
    fn test_render_blocks() {
        let renderer = HtmlRenderer::new();
        let blocks = vec![
            Block::Heading {
                level: 1,
                text: "Dashboard".to_string(),
            },
            Block::Paragraph("System Status".to_string()),
            Block::CommandPrompt("system info".to_string()),
        ];

        let result = renderer.render_blocks(&blocks).unwrap();

        assert!(contains(&result, "Dashboard"));
        assert!(contains(&result, "System Status"));
        assert!(contains(&result, "system info"));
    }

    #[test]
    fn test_render_template() {
        let renderer = HtmlRenderer::new();
        let mut variables = HashMap::new();
        variables.insert("title".to_string(), "Dashboard".to_string());

        let template_data = TemplateData {
            blocks: vec![
                Block::Heading {
                    level: 1,
                    text: "Dashboard".to_string(),
                },
                Block::Paragraph("System Status".to_string()),
            ],
            variables,
            template_name: "dashboard".to_string(),
        };

        let result = renderer.render_template(&template_data).unwrap();

        assert!(contains(&result, "<!DOCTYPE html>"));
        assert!(contains(&result, "terminal"));
        assert!(contains(&result, "Dashboard"));
        assert!(contains(&result, "System Status"));
    }

    #[test]
    fn test_render_metrics() {
        let renderer = HtmlRenderer::new();
        let now = Utc::now();

        let mut labels1 = HashMap::new();
        labels1.insert("unit".to_string(), "%".to_string());

        let mut labels2 = HashMap::new();
        labels2.insert("unit".to_string(), "MB".to_string());
        labels2.insert("trend".to_string(), "-0.5".to_string());

        let metrics = vec![
            Metric {
                name: "CPU Usage".to_string(),
                value: 85.5,
                timestamp: now,
                labels: labels1,
            },
            Metric {
                name: "Memory Usage".to_string(),
                value: 1024.0,
                timestamp: now,
                labels: labels2,
            },
        ];

        let result = renderer.render_metrics(&metrics).unwrap();

        assert!(contains(&result, "CPU Usage"));
        assert!(contains(&result, "85.5"));
        assert!(contains(&result, "%"));
        assert!(contains(&result, "Memory Usage"));
        assert!(contains(&result, "1024"));
        assert!(contains(&result, "MB"));
        assert!(contains(&result, "terminal-trend-down"));
    }

    #[test]
    fn test_render_logs() {
        let renderer = HtmlRenderer::new();

        let logs = vec![
            LogEntry::new("Application started", LogLevel::Info, "app"),
            LogEntry::new("Database connected", LogLevel::Debug, "db"),
            LogEntry::new("Warning: High CPU usage", LogLevel::Warning, "monitor"),
        ];

        let result = renderer.render_logs(&logs).unwrap();

        assert!(contains(&result, "Application started"));
        assert!(contains(&result, "terminal-log-info"));
        assert!(contains(&result, "app"));

        assert!(contains(&result, "Database connected"));
        assert!(contains(&result, "terminal-log-debug"));
        assert!(contains(&result, "db"));

        assert!(contains(&result, "Warning: High CPU usage"));
        assert!(contains(&result, "terminal-log-warning"));
        assert!(contains(&result, "monitor"));
    }

    #[test]
    fn test_render_traces() {
        let renderer = HtmlRenderer::new();

        let mut metadata1 = HashMap::new();
        metadata1.insert("status".to_string(), "completed".to_string());

        let mut metadata2 = HashMap::new();
        metadata2.insert("status".to_string(), "failed".to_string());

        let traces = vec![
            Trace::new("API Request", 120).with_metadata_map(metadata1),
            Trace::new("Database Query", 45).with_metadata_map(metadata2),
        ];

        let result = renderer.render_traces(&traces).unwrap();

        assert!(contains(&result, "Name"));
        assert!(contains(&result, "Duration"));
        assert!(contains(&result, "Started"));
        assert!(contains(&result, "Status"));

        assert!(contains(&result, "API Request"));
        assert!(contains(&result, "120 ms"));
        assert!(contains(&result, "completed"));

        assert!(contains(&result, "Database Query"));
        assert!(contains(&result, "45 ms"));
        assert!(contains(&result, "failed"));
    }
}
