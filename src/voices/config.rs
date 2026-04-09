//! Voice configuration types.

/// Voice playback mode.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VoiceMode {
    /// Continuous overlapping samples.
    /// Plays samples completely, **always** selecting new random samples when current finishes.
    /// Transitions use fixed-duration crossfading for smooth blending.
    /// No probability involved—deterministic sample succession.
    Continuous,
    /// Discrete (event-driven), non-overlapping samples.
    /// Individual samples triggered probabilistically at random intervals.
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
    /// Probability of triggering discrete events (0.0 to 1.0).
    /// **Only used in Discrete mode.** Controls probabilistic event triggering.
    /// For Continuous mode, this field is ignored (all sample transitions occur deterministically).
    pub probability: f32,
    /// Sample pool: list of sample identifiers.
    pub sample_pool: Vec<String>,
    /// For Continuous mode: crossfade/overlap duration in milliseconds (when transitioning between samples).
    /// For Discrete mode: minimum delay between events in milliseconds.
    pub min_delay_ms: u32,
    /// For Continuous mode: reserved for future use (symmetric crossfade).
    /// For Discrete mode: maximum delay between events in milliseconds.
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
