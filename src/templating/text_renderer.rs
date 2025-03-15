use std::collections::HashMap;

use crate::error::Result;
use crate::models::{LogEntry, Metric, Trace};
use crate::templating::renderer::{Block, Renderer, TemplateData};

const DEFAULT_TERMINAL_WIDTH: usize = 80;

pub struct TextRenderer {
    pub terminal_width: usize,
    pub ascii_only: bool,
}

impl TextRenderer {
    pub fn new() -> Self {
        Self {
            terminal_width: DEFAULT_TERMINAL_WIDTH,
            ascii_only: false,
        }
    }

    pub fn with_width(mut self, width: usize) -> Self {
        self.terminal_width = width;
        self
    }

    pub fn with_ascii_only(mut self, ascii_only: bool) -> Self {
        self.ascii_only = ascii_only;
        self
    }

    fn box_chars(&self) -> BoxChars {
        if self.ascii_only {
            BoxChars::ascii()
        } else {
            BoxChars::unicode()
        }
    }

    fn wrap_text(&self, text: &str, indent: usize) -> String {
        let available_width = self.terminal_width.saturating_sub(indent);
        if available_width <= 10 {
            return text.to_string();
        }

        let mut result = String::new();
        let mut current_line = String::new();
        let mut current_width = 0;

        for word in text.split_whitespace() {
            let word_width = word.chars().count();

            if current_width + word_width + 1 > available_width {
                if !current_line.is_empty() {
                    result.push_str(&current_line);
                    result.push('\n');
                    current_line = " ".repeat(indent);
                    current_width = indent;
                }
            }

            if !current_line.is_empty() {
                current_line.push(' ');
                current_width += 1;
            }

            current_line.push_str(word);
            current_width += word_width;
        }

        if !current_line.is_empty() {
            result.push_str(&current_line);
        }

        result
    }

    fn create_box(&self, text: &str, title: Option<&str>) -> String {
        let box_chars = self.box_chars();
        let lines: Vec<&str> = text.lines().collect();

        let max_line_width = lines
            .iter()
            .map(|line| line.chars().count())
            .max()
            .unwrap_or(0);

        let title_width = title.map(|t| t.chars().count() + 2).unwrap_or(0);
        let box_width = max_line_width.max(title_width).min(self.terminal_width - 4) + 4;

        let mut result = String::new();

        if let Some(title_text) = title {
            let title_text = format!(" {} ", title_text);
            let padding = box_width - title_text.len() - 2;
            result.push_str(&format!(
                "{}{}{}{}",
                box_chars.top_left,
                title_text,
                box_chars.horizontal.repeat(padding),
                box_chars.top_right
            ));
        } else {
            result.push_str(&format!(
                "{}{}{}",
                box_chars.top_left,
                box_chars.horizontal.repeat(box_width - 2),
                box_chars.top_right
            ));
        }
        result.push('\n');

        for line in lines {
            let line_width = line.chars().count();
            let padding = box_width - line_width - 4;
            result.push_str(&format!(
                "{} {} {} {}\n",
                box_chars.vertical,
                line,
                " ".repeat(padding),
                box_chars.vertical
            ));
        }

        result.push_str(&format!(
            "{}{}{}",
            box_chars.bottom_left,
            box_chars.horizontal.repeat(box_width - 2),
            box_chars.bottom_right
        ));

        result
    }

    fn format_table(&self, headers: &[String], rows: &[Vec<String>]) -> String {
        if headers.is_empty() && rows.is_empty() {
            return String::new();
        }

        let box_chars = self.box_chars();

        let mut col_widths = vec![
            0;
            headers
                .len()
                .max(rows.iter().map(|row| row.len()).max().unwrap_or(0))
        ];

        for (i, header) in headers.iter().enumerate() {
            if i < col_widths.len() {
                col_widths[i] = col_widths[i].max(header.chars().count());
            }
        }

        for row in rows {
            for (i, cell) in row.iter().enumerate() {
                if i < col_widths.len() {
                    col_widths[i] = col_widths[i].max(cell.chars().count());
                }
            }
        }

        let mut result = String::new();

        result.push_str(&format!("{}", box_chars.top_left));

        for (i, width) in col_widths.iter().enumerate() {
            result.push_str(&box_chars.horizontal.repeat(width + 2));
            if i < col_widths.len() - 1 {
                result.push_str(&box_chars.tee_down);
            }
        }

        result.push_str(&format!("{}\n", box_chars.top_right));

        if !headers.is_empty() {
            result.push_str(&box_chars.vertical);

            for (i, header) in headers.iter().enumerate() {
                let width = if i < col_widths.len() {
                    col_widths[i]
                } else {
                    0
                };
                result.push_str(&format!(" {:<width$} ", header, width = width));
                result.push_str(&box_chars.vertical);
            }

            result.push('\n');

            result.push_str(&box_chars.tee_right);

            for (i, width) in col_widths.iter().enumerate() {
                result.push_str(&box_chars.horizontal.repeat(width + 2));
                if i < col_widths.len() - 1 {
                    result.push_str(&box_chars.cross);
                }
            }

            result.push_str(&format!("{}\n", box_chars.tee_left));
        }

        for (row_idx, row) in rows.iter().enumerate() {
            result.push_str(&box_chars.vertical);

            for (i, cell) in row.iter().enumerate() {
                let width = if i < col_widths.len() {
                    col_widths[i]
                } else {
                    0
                };
                result.push_str(&format!(" {:<width$} ", cell, width = width));
                result.push_str(&box_chars.vertical);
            }

            result.push('\n');

            if row_idx < rows.len() - 1 {
                result.push_str(&box_chars.tee_right);

                for (i, width) in col_widths.iter().enumerate() {
                    result.push_str(&box_chars.horizontal.repeat(width + 2));
                    if i < col_widths.len() - 1 {
                        result.push_str(&box_chars.cross);
                    }
                }

                result.push_str(&format!("{}\n", box_chars.tee_left));
            }
        }

        result.push_str(&format!("{}", box_chars.bottom_left));

        for (i, width) in col_widths.iter().enumerate() {
            result.push_str(&box_chars.horizontal.repeat(width + 2));
            if i < col_widths.len() - 1 {
                result.push_str(&box_chars.tee_up);
            }
        }

        result.push_str(&format!("{}\n", box_chars.bottom_right));

        result
    }
}

struct BoxChars {
    horizontal: String,
    vertical: String,
    top_left: String,
    top_right: String,
    bottom_left: String,
    bottom_right: String,
    cross: String,
    tee_right: String,
    tee_left: String,
    tee_down: String,
    tee_up: String,
}

impl BoxChars {
    fn unicode() -> Self {
        Self {
            horizontal: "─".to_string(),
            vertical: "│".to_string(),
            top_left: "┌".to_string(),
            top_right: "┐".to_string(),
            bottom_left: "└".to_string(),
            bottom_right: "┘".to_string(),
            cross: "┼".to_string(),
            tee_right: "├".to_string(),
            tee_left: "┤".to_string(),
            tee_down: "┬".to_string(),
            tee_up: "┴".to_string(),
        }
    }

    fn ascii() -> Self {
        Self {
            horizontal: "-".to_string(),
            vertical: "|".to_string(),
            top_left: "+".to_string(),
            top_right: "+".to_string(),
            bottom_left: "+".to_string(),
            bottom_right: "+".to_string(),
            cross: "+".to_string(),
            tee_right: "+".to_string(),
            tee_left: "+".to_string(),
            tee_down: "+".to_string(),
            tee_up: "+".to_string(),
        }
    }
}

impl Default for TextRenderer {
    fn default() -> Self {
        Self::new()
    }
}

impl Renderer for TextRenderer {
    fn render_heading(&self, level: usize, text: &str) -> Result<String> {
        let level = level.min(6).max(1);

        let underline_char = match level {
            1 => "=",
            2 => "-",
            3 => "~",
            _ => "",
        };

        let mut result = String::new();
        result.push_str(text);
        result.push('\n');

        if !underline_char.is_empty() && level <= 3 {
            result.push_str(&underline_char.repeat(text.chars().count()));
            result.push_str("\n\n");
        } else {
            result.push('\n');
        }

        Ok(result)
    }

    fn render_paragraph(&self, text: &str) -> Result<String> {
        Ok(format!("{}\n\n", self.wrap_text(text, 0)))
    }

    fn render_command_prompt(&self, command: &str) -> Result<String> {
        Ok(format!("$ {}\n", command))
    }

    fn render_output(&self, blocks: &[Block]) -> Result<String> {
        let content = self.render_blocks(blocks)?;
        Ok(format!("{}\n", content))
    }

    fn render_frame(&self, title: Option<&str>, content: &str) -> Result<String> {
        Ok(format!("{}\n", self.create_box(content, title)))
    }

    fn render_metric(
        &self,
        name: &str,
        value: &str,
        unit: Option<&str>,
        trend: Option<f64>,
    ) -> Result<String> {
        let value_with_unit = if let Some(u) = unit {
            format!("{} {}", value, u)
        } else {
            value.to_string()
        };

        let trend_indicator = match trend {
            Some(t) if t > 0.0 => " ▲",
            Some(t) if t < 0.0 => " ▼",
            _ => "",
        };

        let formatted_value = format!("{}{}", value_with_unit, trend_indicator);

        let padding = self
            .terminal_width
            .saturating_sub(name.chars().count())
            .saturating_sub(formatted_value.chars().count())
            .saturating_sub(3);

        Ok(format!(
            "{}: {}{}\n",
            name,
            " ".repeat(padding),
            formatted_value
        ))
    }

    fn render_log_entry(
        &self,
        message: &str,
        level: &str,
        timestamp: Option<&str>,
        source: Option<&str>,
    ) -> Result<String> {
        let level_str = match level.to_uppercase().as_str() {
            "DEBUG" => "DEBUG",
            "INFO" => "INFO ",
            "WARNING" | "WARN" => "WARN ",
            "ERROR" => "ERROR",
            _ => "INFO ",
        };

        let prefix = match (timestamp, source) {
            (Some(ts), Some(src)) => format!("[{}] [{}] [{}] ", ts, level_str, src),
            (Some(ts), None) => format!("[{}] [{}] ", ts, level_str),
            (None, Some(src)) => format!("[{}] [{}] ", level_str, src),
            (None, None) => format!("[{}] ", level_str),
        };

        let indent = prefix.chars().count();
        let wrapped_message = self.wrap_text(message, indent);

        let mut result = String::new();
        let lines: Vec<&str> = wrapped_message.lines().collect();

        if let Some(first_line) = lines.first() {
            result.push_str(&format!("{}{}\n", prefix, first_line));

            for line in lines.iter().skip(1) {
                result.push_str(&format!("{}{}\n", " ".repeat(indent), line));
            }
        }

        Ok(result)
    }

    fn render_table(&self, headers: &[String], rows: &[Vec<String>]) -> Result<String> {
        Ok(self.format_table(headers, rows))
    }

    fn render_trace(
        &self,
        name: &str,
        duration_ms: u64,
        start_time: &str,
        status: &str,
        metadata: &HashMap<String, String>,
    ) -> Result<String> {
        let mut content = format!("{} ({} ms)\n", name, duration_ms);
        content.push_str(&format!("Started: {}, Status: {}\n", start_time, status));

        if !metadata.is_empty() {
            content.push_str("Metadata:\n");
            for (key, value) in metadata.iter() {
                content.push_str(&format!("  {}: {}\n", key, value));
            }
        }

        Ok(content)
    }

    fn render_raw(&self, content: &str) -> Result<String> {
        Ok(content.to_string())
    }

    fn render_template(&self, template_data: &TemplateData) -> Result<String> {
        let content = self.render_blocks(&template_data.blocks)?;

        let header = format!("# {}\n\n", template_data.template_name);

        let timestamp = chrono::Utc::now().to_rfc3339();
        let footer = format!("\n--- Generated at {} ---\n", timestamp);

        Ok(format!("{}{}{}", header, content, footer))
    }

    fn render_metrics(&self, metrics: &[Metric]) -> Result<String> {
        if metrics.is_empty() {
            return Ok("No metrics available\n".to_string());
        }

        let mut result = String::new();

        for metric in metrics {
            let trend = metric
                .get_label("trend")
                .and_then(|t| t.parse::<f64>().ok());

            let unit = metric.get_label("unit").map(|s| s.as_str());

            result.push_str(&self.render_metric(
                &metric.name,
                &metric.value.to_string(),
                unit,
                trend,
            )?);
        }

        Ok(result)
    }

    fn render_logs(&self, logs: &[LogEntry]) -> Result<String> {
        if logs.is_empty() {
            return Ok("No logs available\n".to_string());
        }

        let mut result = String::new();

        for log in logs {
            result.push_str(&self.render_log_entry(
                &log.message,
                &log.level.to_string(),
                Some(&log.timestamp.to_rfc3339()),
                Some(&log.source),
            )?);
        }

        Ok(result)
    }

    fn render_traces(&self, traces: &[Trace]) -> Result<String> {
        if traces.is_empty() {
            return Ok("No traces available\n".to_string());
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
    use crate::templating::renderer::{Block, Renderer, TemplateData};
    use crate::templating::text_renderer::TextRenderer;

    fn contains(haystack: &str, needle: &str) -> bool {
        haystack.contains(needle)
    }

    #[test]
    fn test_render_heading() {
        let renderer = TextRenderer::new();

        let h1_result = renderer.render_heading(1, "Level 1 Heading").unwrap();
        assert!(contains(&h1_result, "Level 1 Heading"));
        assert!(contains(&h1_result, "============="));

        let h2_result = renderer.render_heading(2, "Level 2 Heading").unwrap();
        assert!(contains(&h2_result, "Level 2 Heading"));
        assert!(contains(&h2_result, "---------------"));

        let h3_result = renderer.render_heading(3, "Level 3 Heading").unwrap();
        assert!(contains(&h3_result, "Level 3 Heading"));
        assert!(contains(&h3_result, "~~~~~~~~~~~~~~~"));

        let h4_result = renderer.render_heading(4, "Level 4 Heading").unwrap();
        assert!(contains(&h4_result, "Level 4 Heading"));
        assert!(!contains(&h4_result, "="));
        assert!(!contains(&h4_result, "-"));
        assert!(!contains(&h4_result, "~"));
    }

    #[test]
    fn test_render_paragraph() {
        let renderer = TextRenderer::new();
        let result = renderer
            .render_paragraph("This is a test paragraph.")
            .unwrap();

        assert!(contains(&result, "This is a test paragraph."));
        assert!(contains(&result, "\n\n"));
    }

    #[test]
    fn test_render_command_prompt() {
        let renderer = TextRenderer::new();
        let result = renderer.render_command_prompt("ls -la").unwrap();

        assert!(contains(&result, "$ ls -la"));
    }

    #[test]
    fn test_render_output() {
        let renderer = TextRenderer::new();
        let blocks = vec![Block::Paragraph("Output content".to_string())];

        let result = renderer.render_output(&blocks).unwrap();

        assert!(contains(&result, "Output content"));
    }

    #[test]
    fn test_render_frame() {
        let renderer = TextRenderer::new();

        let result_with_title = renderer
            .render_frame(Some("Frame Title"), "Frame content")
            .unwrap();

        assert!(contains(&result_with_title, "Frame Title"));
        assert!(contains(&result_with_title, "Frame content"));

        let has_unicode_box = contains(&result_with_title, "┌")
            && contains(&result_with_title, "┐")
            && contains(&result_with_title, "└")
            && contains(&result_with_title, "┘");

        let has_ascii_box = contains(&result_with_title, "+");

        assert!(has_unicode_box || has_ascii_box);

        let result_without_title = renderer.render_frame(None, "Frame content").unwrap();
        assert!(contains(&result_without_title, "Frame content"));
        assert!(!contains(&result_without_title, "Frame Title"));
    }

    #[test]
    fn test_render_metric() {
        let renderer = TextRenderer::new();

        let result = renderer
            .render_metric("CPU Usage", "85.5", None, None)
            .unwrap();
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
        assert!(contains(&result_positive, "▲"));

        let result_negative = renderer
            .render_metric("Errors", "10", None, Some(-0.3))
            .unwrap();
        assert!(contains(&result_negative, "Errors"));
        assert!(contains(&result_negative, "▼"));
    }

    #[test]
    fn test_render_log_entry() {
        let renderer = TextRenderer::new();

        let result = renderer
            .render_log_entry("Application started", "INFO", None, None)
            .unwrap();

        assert!(contains(&result, "INFO"));
        assert!(contains(&result, "Application started"));

        let result_full = renderer
            .render_log_entry(
                "Database error",
                "ERROR",
                Some("2025-03-15T12:34:56Z"),
                Some("db_module"),
            )
            .unwrap();

        assert!(contains(&result_full, "ERROR"));
        assert!(contains(&result_full, "Database error"));
        assert!(contains(&result_full, "2025-03-15T12:34:56Z"));
        assert!(contains(&result_full, "db_module"));
    }

    #[test]
    fn test_render_table() {
        let renderer = TextRenderer::new();
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

        assert!(contains(&result, "Name"));
        assert!(contains(&result, "Value"));
        assert!(contains(&result, "Status"));
        assert!(contains(&result, "Server 1"));
        assert!(contains(&result, "85.5%"));
        assert!(contains(&result, "OK"));
        assert!(contains(&result, "Server 2"));
        assert!(contains(&result, "92.1%"));
        assert!(contains(&result, "Warning"));

        let has_unicode_table = contains(&result, "┌")
            && contains(&result, "┐")
            && contains(&result, "└")
            && contains(&result, "┘")
            && contains(&result, "┼");

        let has_ascii_table =
            contains(&result, "+") && contains(&result, "-") && contains(&result, "|");

        assert!(has_unicode_table || has_ascii_table);
    }

    #[test]
    fn test_render_trace() {
        let renderer = TextRenderer::new();
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
        let renderer = TextRenderer::new();

        let heading_block = Block::Heading {
            level: 1,
            text: "Test Heading".to_string(),
        };

        let heading_result = renderer.render_block(&heading_block).unwrap();
        assert!(contains(&heading_result, "Test Heading"));
        assert!(contains(&heading_result, "============"));

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
        let renderer = TextRenderer::new();
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
        assert!(contains(&result, "========="));
        assert!(contains(&result, "System Status"));
        assert!(contains(&result, "$ system info"));
    }

    #[test]
    fn test_render_template() {
        let renderer = TextRenderer::new();
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

        assert!(contains(&result, "# dashboard"));
        assert!(contains(&result, "Dashboard"));
        assert!(contains(&result, "========="));
        assert!(contains(&result, "System Status"));
        assert!(contains(&result, "Generated at"));
    }

    #[test]
    fn test_render_metrics() {
        let renderer = TextRenderer::new();
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
        assert!(contains(&result, "▼"));
    }

    #[test]
    fn test_render_logs() {
        let renderer = TextRenderer::new();

        let logs = vec![
            LogEntry::new("Application started", LogLevel::Info, "app"),
            LogEntry::new("Database connected", LogLevel::Debug, "db"),
            LogEntry::new("Warning: High CPU usage", LogLevel::Warning, "monitor"),
        ];

        let result = renderer.render_logs(&logs).unwrap();

        assert!(contains(&result, "Application started"));
        assert!(contains(&result, "INFO"));
        assert!(contains(&result, "app"));

        assert!(contains(&result, "Database connected"));
        assert!(contains(&result, "DEBUG"));
        assert!(contains(&result, "db"));

        assert!(contains(&result, "Warning: High CPU usage"));
        assert!(contains(&result, "WARN"));
        assert!(contains(&result, "monitor"));
    }

    #[test]
    fn test_render_traces() {
        let renderer = TextRenderer::new();

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

    #[test]
    fn test_ascii_mode() {
        let renderer = TextRenderer::new().with_ascii_only(true);

        let frame_result = renderer
            .render_frame(Some("ASCII Frame"), "Content")
            .unwrap();
        assert!(contains(&frame_result, "+"));
        assert!(contains(&frame_result, "-"));
        assert!(contains(&frame_result, "|"));
        assert!(!contains(&frame_result, "┌"));
        assert!(!contains(&frame_result, "─"));
        assert!(!contains(&frame_result, "│"));

        let headers = vec!["Col1".to_string(), "Col2".to_string()];
        let rows = vec![vec!["A".to_string(), "B".to_string()]];

        let table_result = renderer.render_table(&headers, &rows).unwrap();
        assert!(contains(&table_result, "+"));
        assert!(contains(&table_result, "-"));
        assert!(contains(&table_result, "|"));
        assert!(!contains(&table_result, "┌"));
        assert!(!contains(&table_result, "─"));
        assert!(!contains(&table_result, "│"));
    }

    #[test]
    fn test_text_wrapping() {
        let renderer = TextRenderer::new().with_width(40);
        let long_text = "This is a very long paragraph that should be wrapped to multiple lines when the terminal width is limited to just 40 characters.";

        let result = renderer.render_paragraph(long_text).unwrap();

        let lines: Vec<&str> = result.lines().collect();
        assert!(lines.len() > 1);

        for line in lines {
            assert!(line.len() <= 40);
        }
    }
}
