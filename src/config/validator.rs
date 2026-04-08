//! Configuration validation.

use crate::voices::config::VoiceMode;

/// Validates configuration for correctness.
pub struct ConfigValidator;

impl ConfigValidator {
    /// Validate sample rate.
    pub fn validate_sample_rate(rate: u32) -> Result<(), String> {
        if !(8000..=192000).contains(&rate) {
            return Err(format!(
                "Sample rate must be between 8000 and 192000 Hz, got {}",
                rate
            ));
        }
        Ok(())
    }

    /// Validate voice mode string.
    pub fn validate_voice_mode(mode: &str) -> Result<VoiceMode, String> {
        match mode {
            "continuous" => Ok(VoiceMode::Continuous),
            "discrete" => Ok(VoiceMode::Discrete),
            _ => Err(format!(
                "Invalid voice mode '{}'. Must be 'continuous' or 'discrete'",
                mode
            )),
        }
    }

    /// Validate pan value.
    pub fn validate_pan(pan: f32) -> Result<(), String> {
        if !(-1.0..=1.0).contains(&pan) {
            return Err(format!("Pan must be between -1.0 and 1.0, got {}", pan));
        }
        Ok(())
    }

    /// Validate probability value.
    pub fn validate_probability(prob: f32) -> Result<(), String> {
        if !(0.0..=1.0).contains(&prob) {
            return Err(format!(
                "Probability must be between 0.0 and 1.0, got {}",
                prob
            ));
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_sample_rate() {
        assert!(ConfigValidator::validate_sample_rate(44100).is_ok());
        assert!(ConfigValidator::validate_sample_rate(4400).is_err());
        assert!(ConfigValidator::validate_sample_rate(192000).is_ok());
        assert!(ConfigValidator::validate_sample_rate(193000).is_err());
    }

    #[test]
    fn test_validate_voice_mode() {
        assert_eq!(
            ConfigValidator::validate_voice_mode("continuous").unwrap(),
            VoiceMode::Continuous
        );
        assert_eq!(
            ConfigValidator::validate_voice_mode("discrete").unwrap(),
            VoiceMode::Discrete
        );
        assert!(ConfigValidator::validate_voice_mode("invalid").is_err());
    }
}
