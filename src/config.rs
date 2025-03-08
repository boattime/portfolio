use crate::error::{Error, Result};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    #[serde(default = "default_source_dir")]
    pub source_dir: PathBuf,

    #[serde(default = "default_output_dir")]
    pub output_dir: PathBuf,

    #[serde(default = "default_templates_dir")]
    pub templates_dir: PathBuf,

    #[serde(default = "default_interval")]
    pub interval_seconds: u64,

    #[serde(default = "default_workers")]
    pub workers: usize,

    #[serde(default = "default_verbose")]
    pub verbose: bool,
}

fn default_source_dir() -> PathBuf {
    PathBuf::from("./content")
}

fn default_output_dir() -> PathBuf {
    PathBuf::from("./public")
}

fn default_templates_dir() -> PathBuf {
    PathBuf::from("./templates")
}

fn default_interval() -> u64 {
    30
}

fn default_workers() -> usize {
    num_cpus::get().max(1)
}

fn default_verbose() -> bool {
    false
}

impl Config {
    pub fn from_env() -> Result<Self> {
        match envy::from_env::<Config>() {
            Ok(config) => Ok(config),
            Err(e) => {
                if let envy::Error::MissingValue(field) = &e {
                    println!(
                        "Warning: Missing environment variable for '{}', using default",
                        field
                    );
                    Ok(Config::default())
                } else {
                    Err(Error::EnvyError(e))
                }
            }
        }
    }

    pub fn default() -> Self {
        Self {
            source_dir: default_source_dir(),
            output_dir: default_output_dir(),
            templates_dir: default_templates_dir(),
            interval_seconds: default_interval(),
            workers: default_workers(),
            verbose: default_verbose(),
        }
    }

    pub fn validate(&self) -> Result<()> {
        if !self.source_dir.exists() {
            println!("Creating source directory: {:?}", self.source_dir);
            std::fs::create_dir_all(&self.source_dir).map_err(|e| {
                Error::ConfigError(format!("Failed to create source directory: {}", e))
            })?;
        }

        if !self.templates_dir.exists() {
            println!("Creating templates directory: {:?}", self.templates_dir);
            std::fs::create_dir_all(&self.templates_dir).map_err(|e| {
                Error::ConfigError(format!("Failed to create templates directory: {}", e))
            })?;
        }

        if !self.output_dir.exists() {
            println!("Creating output directory: {:?}", self.output_dir);
            std::fs::create_dir_all(&self.output_dir).map_err(|e| {
                Error::ConfigError(format!("Failed to create output directory: {}", e))
            })?;
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_default_config() {
        let config = Config::default();
        assert_eq!(config.interval_seconds, 30);
        assert!(config.workers >= 1);
    }

    #[test]
    fn test_validate_config() {
        let temp_dir = TempDir::new().unwrap();
        let temp_path = temp_dir.path();

        let source_dir = temp_path.join("content");
        let templates_dir = temp_path.join("templates");
        let output_dir = temp_path.join("public");

        let config = Config {
            source_dir,
            templates_dir,
            output_dir,
            interval_seconds: 30,
            workers: 2,
            verbose: false,
        };

        assert!(config.validate().is_ok());
        assert!(config.source_dir.exists());
        assert!(config.templates_dir.exists());
        assert!(config.output_dir.exists());
    }
}
