//! Voice configuration types.

/// Voice playback mode.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VoiceMode {
    /// Continuous overlapping samples.
    Continuous,
    /// Discrete (event-driven), non-overlapping samples.
    Discrete,
}

/// Configuration for a single voice.
#[derive(Debug, Clone)]
pub struct VoiceConfig {
    /// Unique identifier for the voice.
    pub id: String,
    /// Voice playback mode.
    pub mode: VoiceMode,
    /// Pan position (-1.0 left to 1.0 right).
    pub pan: f32,
    /// Probability of triggering (0.0 to 1.0).
    pub probability: f32,
    /// Sample pool: list of sample identifiers.
    pub sample_pool: Vec<String>,
    /// Minimum delay between events (milliseconds, for discrete mode).
    pub min_delay_ms: u32,
    /// Maximum delay between events (milliseconds, for discrete mode).
    pub max_delay_ms: u32,
}

impl VoiceConfig {
    /// Create a new voice configuration.
    pub fn new(id: String, mode: VoiceMode) -> Self {
        Self {
            id,
            mode,
            pan: 0.0,
            probability: 1.0,
            sample_pool: vec![],
            min_delay_ms: 100,
            max_delay_ms: 1000,
        }
    }
}
