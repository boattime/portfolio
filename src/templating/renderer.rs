use crate::error::Result;
use crate::models::{LogEntry, Metric, Trace};
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub enum Block {
    Heading {
        level: usize,
        text: String,
    },

    Paragraph(String),

    CommandPrompt(String),

    Output(Vec<Block>),

    Frame {
        title: Option<String>,
        content: Vec<Block>,
    },

    Metric {
        name: String,
        value: String,
        unit: Option<String>,
        trend: Option<f64>,
    },

    LogEntry {
        message: String,
        level: String,
        timestamp: Option<String>,
        source: Option<String>,
    },

    Table {
        headers: Vec<String>,
        rows: Vec<Vec<String>>,
    },

    Trace {
        name: String,
        duration_ms: u64,
        start_time: String,
        status: String,
        metadata: HashMap<String, String>,
    },

    Raw(String),

    Container(Vec<Block>),
}

#[derive(Debug, Clone)]
pub struct TemplateData {
    pub blocks: Vec<Block>,
    pub template_name: String,
}

pub trait Renderer {
    fn render_heading(&self, level: usize, text: &str) -> Result<String>;

    fn render_paragraph(&self, text: &str) -> Result<String>;

    fn render_command_prompt(&self, command: &str) -> Result<String>;

    fn render_output(&self, blocks: &[Block]) -> Result<String>;

    fn render_frame(&self, title: Option<&str>, content: &str) -> Result<String>;

    fn render_metric(
        &self,
        name: &str,
        value: &str,
        unit: Option<&str>,
        trend: Option<f64>,
    ) -> Result<String>;

    fn render_log_entry(
        &self,
        message: &str,
        level: &str,
        timestamp: Option<&str>,
        source: Option<&str>,
    ) -> Result<String>;

    fn render_table(&self, headers: &[String], rows: &[Vec<String>]) -> Result<String>;

    fn render_trace(
        &self,
        name: &str,
        duration_ms: u64,
        start_time: &str,
        status: &str,
        metadata: &HashMap<String, String>,
    ) -> Result<String>;

    fn render_raw(&self, content: &str) -> Result<String>;

    fn render_block(&self, block: &Block) -> Result<String> {
        match block {
            Block::Heading { level, text } => self.render_heading(*level, text),
            Block::Paragraph(text) => self.render_paragraph(text),
            Block::CommandPrompt(command) => self.render_command_prompt(command),
            Block::Output(blocks) => self.render_output(blocks),
            Block::Frame { title, content } => {
                let rendered_content = self.render_blocks(content)?;
                self.render_frame(title.as_deref(), &rendered_content)
            }
            Block::Metric {
                name,
                value,
                unit,
                trend,
            } => self.render_metric(name, value, unit.as_deref(), *trend),
            Block::LogEntry {
                message,
                level,
                timestamp,
                source,
            } => self.render_log_entry(message, level, timestamp.as_deref(), source.as_deref()),
            Block::Table { headers, rows } => self.render_table(headers, rows),
            Block::Trace {
                name,
                duration_ms,
                start_time,
                status,
                metadata,
            } => self.render_trace(name, *duration_ms, start_time, status, metadata),
            Block::Raw(content) => self.render_raw(content),
            Block::Container(blocks) => self.render_blocks(blocks),
        }
    }

    fn render_blocks(&self, blocks: &[Block]) -> Result<String> {
        let mut result = String::new();
        for block in blocks {
            result.push_str(&self.render_block(block)?);
        }
        Ok(result)
    }

    fn render_template(&self, template_data: &TemplateData) -> Result<String>;

    fn render_metrics(&self, metrics: &[Metric]) -> Result<String>;

    fn render_logs(&self, logs: &[LogEntry]) -> Result<String>;

    fn render_traces(&self, traces: &[Trace]) -> Result<String>;
}
