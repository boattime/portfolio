use crate::error::{Error, Result};
use crate::templating::renderer::{Block, TemplateData};
use std::collections::HashMap;
use std::fs;
use std::path::Path;

#[derive(Debug, Clone)]
pub struct Template {
    pub name: String,

    pub content: String,

    pub blocks: Vec<Block>,

    pub variables: HashMap<String, String>,
}

impl Template {
    pub fn from_file<P: AsRef<Path>>(path: P) -> Result<Self> {
        let path_ref = path.as_ref();
        let content = fs::read_to_string(path_ref).map_err(|e| {
            Error::TemplateError(format!("Failed to read template file: {}", e).into())
        })?;

        let name = path_ref
            .file_stem()
            .and_then(|os_str| os_str.to_str())
            .unwrap_or("unnamed")
            .to_string();

        Self::from_string(name, content)
    }

    pub fn from_string<S: Into<String>>(name: S, content: S) -> Result<Self> {
        let name = name.into();
        let content = content.into();

        let mut parser = TemplateParser::new(&content);
        let blocks = parser.parse()?;

        Ok(Self {
            name,
            content,
            blocks,
            variables: HashMap::new(),
        })
    }

    pub fn to_template_data(&self) -> TemplateData {
        TemplateData {
            blocks: self.blocks.clone(),
            variables: self.variables.clone(),
            template_name: self.name.clone(),
        }
    }

    pub fn set_variable<S: Into<String>>(&mut self, name: S, value: S) {
        self.variables.insert(name.into(), value.into());
    }

    pub fn set_variables(&mut self, variables: HashMap<String, String>) {
        self.variables.extend(variables);
    }
}

struct TemplateParser<'a> {
    content: &'a str,
    position: usize,
    line: usize,
    column: usize,
}

impl<'a> TemplateParser<'a> {
    fn new(content: &'a str) -> Self {
        Self {
            content,
            position: 0,
            line: 1,
            column: 1,
        }
    }

    fn parse(&mut self) -> Result<Vec<Block>> {
        let mut blocks = Vec::new();

        while let Some(block) = self.parse_block()? {
            blocks.push(block);
        }

        Ok(blocks)
    }

    fn parse_block(&mut self) -> Result<Option<Block>> {
        self.skip_whitespace();

        if self.is_at_end() {
            return Ok(None);
        }

        if self.match_char('@') {
            if self.match_string("metrics") {
                return Ok(Some(Block::Raw("@metrics".to_string())));
            } else if self.match_string("logs") {
                return Ok(Some(Block::Raw("@logs".to_string())));
            } else if self.match_string("traces") {
                return Ok(Some(Block::Raw("@traces".to_string())));
            } else if self.match_string("var") {
                self.expect_char('{')?;
                let var_name = self.parse_until('}')?;
                self.expect_char('}')?;
                return Ok(Some(Block::Raw(format!("@var{{{}}}", var_name))));
            } else {
                return self.parse_directive();
            }
        }

        let text = self.parse_text();
        if text.is_empty() {
            Ok(None)
        } else {
            Ok(Some(Block::Paragraph(text)))
        }
    }

    fn parse_directive(&mut self) -> Result<Option<Block>> {
        let directive = self.parse_identifier();

        match directive.as_str() {
            "heading" => self.parse_heading_directive(),
            "paragraph" => self.parse_paragraph_directive(),
            "command" => self.parse_command_directive(),
            "output" => self.parse_output_directive(),
            "frame" => self.parse_frame_directive(),
            "metric" => self.parse_metric_directive(),
            "log" => self.parse_log_directive(),
            "table" => self.parse_table_directive(),
            "trace" => self.parse_trace_directive(),
            "raw" => self.parse_raw_directive(),
            _ => Err(Error::TemplateError(
                format!(
                    "Unknown directive @{} at line {}, column {}",
                    directive, self.line, self.column
                )
                .into(),
            )),
        }
    }

    fn parse_heading_directive(&mut self) -> Result<Option<Block>> {
        self.expect_char('{')?;
        let level_str = self.parse_until('}')?;
        self.expect_char('}')?;

        let level = level_str.parse::<usize>().map_err(|_| {
            Error::TemplateError(
                format!(
                    "Invalid heading level '{}' at line {}, column {}",
                    level_str, self.line, self.column
                )
                .into(),
            )
        })?;

        self.expect_char('{')?;
        let text = self.parse_until('}')?;
        self.expect_char('}')?;

        Ok(Some(Block::Heading { level, text }))
    }

    fn parse_paragraph_directive(&mut self) -> Result<Option<Block>> {
        self.expect_char('{')?;
        let text = self.parse_until('}')?;
        self.expect_char('}')?;

        Ok(Some(Block::Paragraph(text)))
    }

    fn parse_command_directive(&mut self) -> Result<Option<Block>> {
        self.expect_char('{')?;
        let command = self.parse_until('}')?;
        self.expect_char('}')?;

        Ok(Some(Block::CommandPrompt(command)))
    }

    fn parse_output_directive(&mut self) -> Result<Option<Block>> {
        self.expect_char('{')?;

        let start_pos = self.position;
        let mut depth = 1;

        while depth > 0 && !self.is_at_end() {
            let c = self.advance();
            if c == '{' {
                depth += 1;
            } else if c == '}' {
                depth -= 1;
            }
        }

        if depth > 0 {
            return Err(Error::TemplateError(
                format!(
                    "Unclosed output block at line {}, column {}",
                    self.line, self.column
                )
                .into(),
            ));
        }

        let content = &self.content[start_pos..(self.position - 1)];

        let mut nested_parser = TemplateParser::new(content);
        let nested_blocks = nested_parser.parse()?;

        Ok(Some(Block::Output(nested_blocks)))
    }

    fn parse_frame_directive(&mut self) -> Result<Option<Block>> {
        let title = if self.peek() == '{' {
            self.expect_char('{')?;
            let title = self.parse_until('}')?;
            self.expect_char('}')?;
            Some(title)
        } else {
            None
        };

        self.expect_char('{')?;

        let start_pos = self.position;
        let mut depth = 1;

        while depth > 0 && !self.is_at_end() {
            let c = self.advance();
            if c == '{' {
                depth += 1;
            } else if c == '}' {
                depth -= 1;
            }
        }

        if depth > 0 {
            return Err(Error::TemplateError(
                format!(
                    "Unclosed frame block at line {}, column {}",
                    self.line, self.column
                )
                .into(),
            ));
        }

        let content = &self.content[start_pos..(self.position - 1)];

        let mut nested_parser = TemplateParser::new(content);
        let nested_blocks = nested_parser.parse()?;

        Ok(Some(Block::Frame {
            title,
            content: nested_blocks,
        }))
    }

    fn parse_metric_directive(&mut self) -> Result<Option<Block>> {
        self.expect_char('{')?;
        let name = self.parse_until('}')?;
        self.expect_char('}')?;

        self.expect_char('{')?;
        let value = self.parse_until('}')?;
        self.expect_char('}')?;

        let unit = if self.peek() == '{' {
            self.expect_char('{')?;
            let unit = self.parse_until('}')?;
            self.expect_char('}')?;
            Some(unit)
        } else {
            None
        };

        let trend = if self.peek() == '{' {
            self.expect_char('{')?;
            let trend_str = self.parse_until('}')?;
            self.expect_char('}')?;

            Some(trend_str.parse::<f64>().map_err(|_| {
                Error::TemplateError(
                    format!(
                        "Invalid trend value '{}' at line {}, column {}",
                        trend_str, self.line, self.column
                    )
                    .into(),
                )
            })?)
        } else {
            None
        };

        Ok(Some(Block::Metric {
            name,
            value,
            unit,
            trend,
        }))
    }

    fn parse_log_directive(&mut self) -> Result<Option<Block>> {
        self.expect_char('{')?;
        let message = self.parse_until('}')?;
        self.expect_char('}')?;

        self.expect_char('{')?;
        let level = self.parse_until('}')?;
        self.expect_char('}')?;

        let timestamp = if self.peek() == '{' {
            self.expect_char('{')?;
            let timestamp = self.parse_until('}')?;
            self.expect_char('}')?;
            Some(timestamp)
        } else {
            None
        };

        let source = if self.peek() == '{' {
            self.expect_char('{')?;
            let source = self.parse_until('}')?;
            self.expect_char('}')?;
            Some(source)
        } else {
            None
        };

        Ok(Some(Block::LogEntry {
            message,
            level,
            timestamp,
            source,
        }))
    }

    fn parse_table_directive(&mut self) -> Result<Option<Block>> {
        self.expect_char('{')?;

        self.skip_whitespace();

        let mut headers = Vec::new();
        if self.match_string("@headers") {
            self.expect_char('{')?;
            let headers_str = self.parse_until('}')?;
            self.expect_char('}')?;

            headers = headers_str
                .split('|')
                .map(|s| s.trim().to_string())
                .collect();

            self.skip_whitespace();
        }

        let mut rows = Vec::new();
        while self.match_string("@row") {
            self.expect_char('{')?;
            let row_str = self.parse_until('}')?;
            self.expect_char('}')?;

            let row: Vec<String> = row_str.split('|').map(|s| s.trim().to_string()).collect();
            rows.push(row);

            self.skip_whitespace();
        }

        self.expect_char('}')?;

        Ok(Some(Block::Table { headers, rows }))
    }

    fn parse_trace_directive(&mut self) -> Result<Option<Block>> {
        self.expect_char('{')?;
        let name = self.parse_until('}')?;
        self.expect_char('}')?;

        self.expect_char('{')?;
        let duration_str = self.parse_until('}')?;
        self.expect_char('}')?;

        let duration_ms = duration_str.parse::<u64>().map_err(|_| {
            Error::TemplateError(
                format!(
                    "Invalid duration '{}' at line {}, column {}",
                    duration_str, self.line, self.column
                )
                .into(),
            )
        })?;

        self.expect_char('{')?;
        let start_time = self.parse_until('}')?;
        self.expect_char('}')?;

        self.expect_char('{')?;
        let status = self.parse_until('}')?;
        self.expect_char('}')?;

        let mut metadata = HashMap::new();
        if self.peek() == '{' {
            self.expect_char('{')?;

            while self.match_string("@meta") {
                self.expect_char('{')?;
                let key = self.parse_until('}')?;
                self.expect_char('}')?;

                self.expect_char('{')?;
                let value = self.parse_until('}')?;
                self.expect_char('}')?;

                metadata.insert(key, value);
            }

            self.expect_char('}')?;
        }

        Ok(Some(Block::Trace {
            name,
            duration_ms,
            start_time,
            status,
            metadata,
        }))
    }

    fn parse_raw_directive(&mut self) -> Result<Option<Block>> {
        self.expect_char('{')?;
        let content = self.parse_until('}')?;
        self.expect_char('}')?;

        Ok(Some(Block::Raw(content)))
    }

    fn parse_text(&mut self) -> String {
        let start_pos = self.position;

        while !self.is_at_end() && self.peek() != '@' {
            self.advance();
        }

        self.content[start_pos..self.position].to_string()
    }

    fn parse_identifier(&mut self) -> String {
        let start_pos = self.position;

        while !self.is_at_end() && (self.peek().is_alphanumeric() || self.peek() == '_') {
            self.advance();
        }

        self.content[start_pos..self.position].to_string()
    }

    fn parse_until(&mut self, end_char: char) -> Result<String> {
        let start_pos = self.position;

        while !self.is_at_end() && self.peek() != end_char {
            if self.peek() == '\\' && self.peek_next() == end_char {
                self.advance();
            }
            self.advance();
        }

        if self.is_at_end() {
            return Err(Error::TemplateError(
                format!(
                    "Expected '{}' but reached end of input at line {}, column {}",
                    end_char, self.line, self.column
                )
                .into(),
            ));
        }

        Ok(self.content[start_pos..self.position].to_string())
    }

    fn expect_char(&mut self, expected: char) -> Result<()> {
        if self.is_at_end() {
            return Err(Error::TemplateError(
                format!(
                    "Expected '{}' but reached end of input at line {}, column {}",
                    expected, self.line, self.column
                )
                .into(),
            ));
        }

        if self.peek() != expected {
            return Err(Error::TemplateError(
                format!(
                    "Expected '{}' but found '{}' at line {}, column {}",
                    expected,
                    self.peek(),
                    self.line,
                    self.column
                )
                .into(),
            ));
        }

        self.advance();
        Ok(())
    }

    fn match_char(&mut self, expected: char) -> bool {
        if self.is_at_end() || self.peek() != expected {
            return false;
        }

        self.advance();
        true
    }

    fn match_string(&mut self, expected: &str) -> bool {
        let end_pos = self.position + expected.len();
        if end_pos > self.content.len() {
            return false;
        }

        if &self.content[self.position..end_pos] != expected {
            return false;
        }

        for _ in 0..expected.len() {
            self.advance();
        }

        true
    }

    fn skip_whitespace(&mut self) {
        while !self.is_at_end() && self.peek().is_whitespace() {
            self.advance();
        }
    }

    fn advance(&mut self) -> char {
        let c = self.content[self.position..].chars().next().unwrap();
        self.position += c.len_utf8();

        if c == '\n' {
            self.line += 1;
            self.column = 1;
        } else {
            self.column += 1;
        }

        c
    }

    fn peek(&self) -> char {
        if self.is_at_end() {
            '\0'
        } else {
            self.content[self.position..].chars().next().unwrap()
        }
    }

    fn peek_next(&self) -> char {
        if self.is_at_end() {
            return '\0';
        }

        let current_char = self.content[self.position..].chars().next().unwrap();
        let current_len = current_char.len_utf8();

        if self.position + current_len >= self.content.len() {
            '\0'
        } else {
            self.content[self.position + current_len..]
                .chars()
                .next()
                .unwrap()
        }
    }

    fn is_at_end(&self) -> bool {
        self.position >= self.content.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_template_from_string() {
        let template_content = "@heading{1}{Test Template}\n@paragraph{This is a test.}";
        let template = Template::from_string("test", template_content).unwrap();

        assert_eq!(template.name, "test");
        assert_eq!(template.blocks.len(), 2);

        match &template.blocks[0] {
            Block::Heading { level, text } => {
                assert_eq!(*level, 1);
                assert_eq!(text, "Test Template");
            }
            _ => panic!("Expected heading block"),
        }

        match &template.blocks[1] {
            Block::Paragraph(text) => {
                assert_eq!(text, "This is a test.");
            }
            _ => panic!("Expected paragraph block"),
        }
    }

    #[test]
    fn test_template_from_file() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "@heading{{1}}{{Test Template}}").unwrap();
        writeln!(temp_file, "@paragraph{{This is a test.}}").unwrap();

        let template = Template::from_file(temp_file.path()).unwrap();

        assert_eq!(template.blocks.len(), 2);

        match &template.blocks[0] {
            Block::Heading { level, text } => {
                assert_eq!(*level, 1);
                assert_eq!(text, "Test Template");
            }
            _ => panic!("Expected heading block"),
        }
    }

    #[test]
    fn test_parse_directive() {
        let template_content = "@heading{1}{Title}\n@paragraph{Text}\n@command{ls -la}\n";
        let template = Template::from_string("test", template_content).unwrap();

        assert_eq!(template.blocks.len(), 3);

        match &template.blocks[0] {
            Block::Heading { level, text } => {
                assert_eq!(*level, 1);
                assert_eq!(text, "Title");
            }
            _ => panic!("Expected heading block"),
        }

        match &template.blocks[1] {
            Block::Paragraph(text) => {
                assert_eq!(text, "Text");
            }
            _ => panic!("Expected paragraph block"),
        }

        match &template.blocks[2] {
            Block::CommandPrompt(cmd) => {
                assert_eq!(cmd, "ls -la");
            }
            _ => panic!("Expected command block"),
        }
    }

    #[test]
    fn test_parse_nested_blocks() {
        let template_content =
            "@frame{Frame Title}{\n@heading{2}{Nested Heading}\n@paragraph{Nested paragraph.}\n}";
        let template = Template::from_string("test", template_content).unwrap();

        assert_eq!(template.blocks.len(), 1);

        match &template.blocks[0] {
            Block::Frame { title, content } => {
                assert_eq!(title.as_ref().unwrap(), "Frame Title");
                assert_eq!(content.len(), 2);

                match &content[0] {
                    Block::Heading { level, text } => {
                        assert_eq!(*level, 2);
                        assert_eq!(text, "Nested Heading");
                    }
                    _ => panic!("Expected heading block"),
                }

                match &content[1] {
                    Block::Paragraph(text) => {
                        assert_eq!(text, "Nested paragraph.");
                    }
                    _ => panic!("Expected paragraph block"),
                }
            }
            _ => panic!("Expected frame block"),
        }
    }

    #[test]
    fn test_parse_metric() {
        let template_content = "@metric{CPU Usage}{78.5}{%}{+2.3}";
        let template = Template::from_string("test", template_content).unwrap();

        assert_eq!(template.blocks.len(), 1);

        match &template.blocks[0] {
            Block::Metric {
                name,
                value,
                unit,
                trend,
            } => {
                assert_eq!(name, "CPU Usage");
                assert_eq!(value, "78.5");
                assert_eq!(unit.as_ref().unwrap(), "%");
                assert_eq!(trend.unwrap(), 2.3);
            }
            _ => panic!("Expected metric block"),
        }
    }

    #[test]
    fn test_parse_table() {
        let template_content = "@table{\n@headers{Name|Value|Status}\n@row{Server 1|10.5|OK}\n@row{Server 2|8.3|Warning}\n}";
        let template = Template::from_string("test", template_content).unwrap();

        assert_eq!(template.blocks.len(), 1);

        match &template.blocks[0] {
            Block::Table { headers, rows } => {
                assert_eq!(headers.len(), 3);
                assert_eq!(headers[0], "Name");
                assert_eq!(headers[1], "Value");
                assert_eq!(headers[2], "Status");

                assert_eq!(rows.len(), 2);
                assert_eq!(rows[0][0], "Server 1");
                assert_eq!(rows[0][1], "10.5");
                assert_eq!(rows[0][2], "OK");

                assert_eq!(rows[1][0], "Server 2");
                assert_eq!(rows[1][1], "8.3");
                assert_eq!(rows[1][2], "Warning");
            }
            _ => panic!("Expected table block"),
        }
    }

    #[test]
    fn test_parse_plain_text() {
        let template_content = "This is plain text.\n@heading{1}{Title}\nMore plain text.";
        let template = Template::from_string("test", template_content).unwrap();

        assert_eq!(template.blocks.len(), 3);

        match &template.blocks[0] {
            Block::Paragraph(text) => {
                assert_eq!(text, "This is plain text.\n");
            }
            _ => panic!("Expected paragraph block"),
        }

        match &template.blocks[1] {
            Block::Heading { level, text } => {
                assert_eq!(*level, 1);
                assert_eq!(text, "Title");
            }
            _ => panic!("Expected heading block"),
        }

        match &template.blocks[2] {
            Block::Paragraph(text) => {
                assert_eq!(text, "More plain text.");
            }
            _ => panic!("Expected paragraph block"),
        }
    }

    #[test]
    fn test_template_error_handling() {
        let template_content = "@heading{1{Title}";
        let result = Template::from_string("test", template_content);
        assert!(result.is_err());

        let template_content = "@unknown{Something}";
        let result = Template::from_string("test", template_content);
        assert!(result.is_err());

        let template_content = "@heading{not_a_number}{Title}";
        let result = Template::from_string("test", template_content);
        assert!(result.is_err());
    }

    #[test]
    fn test_template_variables() {
        let mut template = Template::from_string("test", "@heading{1}{Hello @var{name}}").unwrap();

        template.set_variable("name", "World");

        let data = template.to_template_data();
        assert_eq!(data.variables.get("name").unwrap(), "World");
    }
}
