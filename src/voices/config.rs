//! Voice configuration types.

/// Voice playback mode.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VoiceMode {
    /// Continuous overlapping samples.
    /// Plays samples completely, selecting new samples randomly when current finishes.
    /// New samples fade in over configurable overlap period for smooth blending.
    Continuous,
    /// Discrete (event-driven), non-overlapping samples.
    /// Individual samples triggered probabilistically with gaps between them.
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
    /// Probability of triggering new sample in continuous mode (0.0 to 1.0).
    /// For continuous mode: probability that a new sample will be selected when current finishes.
    /// For discrete mode: probability of triggering a new event.
    pub probability: f32,
    /// Sample pool: list of sample identifiers.
    pub sample_pool: Vec<String>,
    /// For continuous mode: overlap fade duration in milliseconds (when transitioning between samples).
    /// For discrete mode: minimum delay between events in milliseconds.
    pub min_delay_ms: u32,
    /// For continuous mode: reserved for future use (crossfade symmetric).
    /// For discrete mode: maximum delay between events in milliseconds.
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
