//! YAML configuration parser.

use crate::voices::config::{VoiceConfig, VoiceMode};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

/// Parsed configuration structure.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    /// Sample rate in Hz.
    pub sample_rate: u32,
    /// List of voices (previously "streams").
    pub voices: Option<Vec<VoiceConfigYaml>>,
    /// Legacy field: List of streams (for backward compatibility).
    pub streams: Option<Vec<VoiceConfigYaml>>,
    /// Sample directory.
    pub sample_dir: String,
}

/// Voice configuration from YAML.
///
/// Supports both mode-specific and legacy flat structure for backward compatibility.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VoiceConfigYaml {
    /// Voice identifier.
    pub id: String,
    /// Voice mode: "continuous" or "discrete".
    pub mode: String,
    /// Stereo pan: -1.0 (left) to 1.0 (right).
    pub pan: Option<f32>,
    /// For discrete mode: trigger probability (0.0 to 1.0).
    pub probability: Option<f32>,
    /// For continuous mode: overlap duration in ms.
    /// For discrete mode: ignored (use min_delay_ms, max_delay_ms).
    pub overlap_ms: Option<u32>,
    /// Pool of sample identifiers to use.
    pub samples: Option<Vec<String>>,
    /// For discrete mode: minimum delay between events (ms).
    pub min_delay_ms: Option<u32>,
    /// For discrete mode: maximum delay between events (ms).
    pub max_delay_ms: Option<u32>,
}

impl VoiceConfigYaml {
    /// Convert YAML config to VoiceConfig.
    pub fn to_voice_config(&self) -> Result<VoiceConfig, String> {
        let mode = match self.mode.to_lowercase().as_str() {
            "continuous" => {
                let overlap_ms = self.overlap_ms.or(self.min_delay_ms).unwrap_or(500);
                VoiceMode::continuous(overlap_ms)
            }
            "discrete" => {
                let probability = self.probability.unwrap_or(0.5).clamp(0.0, 1.0);
                let min_delay_ms = self.min_delay_ms.unwrap_or(100);
                let max_delay_ms = self.max_delay_ms.unwrap_or(500);
                VoiceMode::discrete(probability, min_delay_ms, max_delay_ms)
            }
            _ => return Err(format!("Invalid mode: {}", self.mode)),
        };

        let mut config = VoiceConfig::new(self.id.clone(), mode);
        if let Some(pan) = self.pan {
            config.pan = pan.clamp(-1.0, 1.0);
        }
        if let Some(samples) = &self.samples {
            config.sample_pool = samples.clone();
        }
        Ok(config)
    }
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

    /// Get voices from config, checking both 'voices' and legacy 'streams' fields.
    pub fn get_voices(config: &Config) -> Result<Vec<VoiceConfig>, String> {
        let yaml_configs = config
            .voices
            .as_ref()
            .or(config.streams.as_ref())
            .ok_or_else(|| "No voices or streams defined in config".to_string())?;

        yaml_configs.iter().map(|v| v.to_voice_config()).collect()
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
voices:
  - id: water
    mode: continuous
    pan: 0.0
    overlap_ms: 500
    samples:
      - water1
      - water2
"#;
        let config: Config = serde_yaml::from_str(yaml).expect("Failed to parse YAML");
        assert_eq!(config.sample_rate, 44100);
        assert_eq!(config.voices.as_ref().unwrap().len(), 1);
        assert_eq!(config.voices.as_ref().unwrap()[0].id, "water");
    }

    #[test]
    fn test_legacy_streams_field() {
        let yaml = r#"
sample_rate: 44100
sample_dir: "./samples"
streams:
  - id: rain
    mode: discrete
    pan: 0.2
    probability: 0.6
    min_delay_ms: 100
    max_delay_ms: 300
    samples:
      - drop1
      - drop2
"#;
        let config: Config = serde_yaml::from_str(yaml).expect("Failed to parse YAML");
        assert_eq!(config.sample_rate, 44100);
        assert_eq!(config.streams.as_ref().unwrap().len(), 1);
    }

    #[test]
    fn test_yaml_to_voice_config_continuous() {
        let yaml_config = VoiceConfigYaml {
            id: "ambience".to_string(),
            mode: "continuous".to_string(),
            pan: Some(0.5),
            probability: None,
            overlap_ms: Some(1000),
            samples: Some(vec!["sample1".to_string()]),
            min_delay_ms: None,
            max_delay_ms: None,
        };

        let config = yaml_config.to_voice_config().expect("Failed to convert");
        assert_eq!(config.id, "ambience");
        assert!(config.mode.is_continuous());
        assert_eq!(config.pan, 0.5);
    }

    #[test]
    fn test_yaml_to_voice_config_discrete() {
        let yaml_config = VoiceConfigYaml {
            id: "birds".to_string(),
            mode: "discrete".to_string(),
            pan: Some(-0.3),
            probability: Some(0.7),
            overlap_ms: None,
            samples: Some(vec!["chirp1".to_string()]),
            min_delay_ms: Some(200),
            max_delay_ms: Some(800),
        };

        let config = yaml_config.to_voice_config().expect("Failed to convert");
        assert_eq!(config.id, "birds");
        assert!(config.mode.is_discrete());
        assert_eq!(config.pan, -0.3);
    }
}
