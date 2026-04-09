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

    /// Validate a VoiceMode configuration.
    pub fn validate_voice_mode(mode: &VoiceMode) -> Result<(), String> {
        match mode {
            VoiceMode::Continuous { overlap_ms } => {
                if *overlap_ms == 0 {
                    return Err("Continuous overlap_ms must be greater than 0".to_string());
                }
                Ok(())
            }
            VoiceMode::Discrete {
                probability,
                min_delay_ms,
                max_delay_ms,
            } => {
                if !(0.0..=1.0).contains(probability) {
                    return Err(format!(
                        "Discrete probability must be between 0.0 and 1.0, got {}",
                        probability
                    ));
                }
                if min_delay_ms > max_delay_ms {
                    return Err(format!(
                        "Discrete min_delay_ms ({}) must be <= max_delay_ms ({})",
                        min_delay_ms, max_delay_ms
                    ));
                }
                Ok(())
            }
        }
    }

    /// Validate pan value.
    pub fn validate_pan(pan: f32) -> Result<(), String> {
        if !(-1.0..=1.0).contains(&pan) {
            return Err(format!("Pan must be between -1.0 and 1.0, got {}", pan));
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
    fn test_validate_continuous_mode() {
        let continuous = VoiceMode::continuous(1000);
        assert!(ConfigValidator::validate_voice_mode(&continuous).is_ok());

        let invalid = VoiceMode::continuous(0);
        assert!(ConfigValidator::validate_voice_mode(&invalid).is_err());
    }

    #[test]
    fn test_validate_discrete_mode() {
        let discrete = VoiceMode::discrete(0.5, 100, 500);
        assert!(ConfigValidator::validate_voice_mode(&discrete).is_ok());

        // Test invalid delay range
        let invalid_delays = VoiceMode::discrete(0.5, 500, 100);
        assert!(ConfigValidator::validate_voice_mode(&invalid_delays).is_err());
    }

    #[test]
    fn test_validate_pan() {
        assert!(ConfigValidator::validate_pan(0.0).is_ok());
        assert!(ConfigValidator::validate_pan(-1.0).is_ok());
        assert!(ConfigValidator::validate_pan(1.0).is_ok());
        assert!(ConfigValidator::validate_pan(-1.5).is_err());
        assert!(ConfigValidator::validate_pan(1.5).is_err());
    }
}
