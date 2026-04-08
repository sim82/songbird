//! YAML configuration parser.

use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

/// Parsed configuration structure.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    /// Sample rate in Hz.
    pub sample_rate: u32,
    /// List of streams.
    pub streams: Vec<StreamConfigYaml>,
    /// Sample directory.
    pub sample_dir: String,
}

/// Stream configuration from YAML.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StreamConfigYaml {
    /// Stream identifier.
    pub id: String,
    /// Stream mode: "continuous" or "bird".
    pub mode: String,
    /// Stereo pan: -1.0 (left) to 1.0 (right).
    pub pan: Option<f32>,
    /// Trigger probability (0.0 to 1.0).
    pub probability: Option<f32>,
    /// Pool of sample identifiers to use.
    pub samples: Option<Vec<String>>,
    /// Minimum delay between events (ms).
    pub min_delay_ms: Option<u32>,
    /// Maximum delay between events (ms).
    pub max_delay_ms: Option<u32>,
}

/// Parses YAML configuration files.
pub struct ConfigParser;

impl ConfigParser {
    /// Parse a YAML configuration file.
    pub fn parse<P: AsRef<Path>>(path: P) -> Result<Config, String> {
        let path = path.as_ref();
        let content =
            fs::read_to_string(path).map_err(|e| format!("Failed to read config file: {}", e))?;

        serde_yaml::from_str(&content).map_err(|e| format!("Failed to parse YAML: {}", e))
    }

    /// Write a configuration to a YAML file.
    pub fn write<P: AsRef<Path>>(path: P, config: &Config) -> Result<(), String> {
        let yaml = serde_yaml::to_string(config)
            .map_err(|e| format!("Failed to serialize config: {}", e))?;

        fs::write(path, yaml).map_err(|e| format!("Failed to write config file: {}", e))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_parsing() {
        let yaml = r#"
sample_rate: 44100
sample_dir: "./samples"
streams:
  - id: water
    mode: continuous
    pan: 0.0
    probability: 0.8
    samples:
      - water1
      - water2
"#;
        let config: Config = serde_yaml::from_str(yaml).expect("Failed to parse YAML");
        assert_eq!(config.sample_rate, 44100);
        assert_eq!(config.streams.len(), 1);
        assert_eq!(config.streams[0].id, "water");
    }
}
