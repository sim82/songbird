//! Stream configuration types.

/// Stream playback mode.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StreamMode {
    /// Continuous overlapping samples.
    Continuous,
    /// Event-driven, non-overlapping bird mode.
    Bird,
}

/// Configuration for a single stream.
#[derive(Debug, Clone)]
pub struct StreamConfig {
    /// Unique identifier for the stream.
    pub id: String,
    /// Stream playback mode.
    pub mode: StreamMode,
    /// Pan position (-1.0 left to 1.0 right).
    pub pan: f32,
    /// Probability of triggering (0.0 to 1.0).
    pub probability: f32,
    /// Sample pool: list of sample identifiers.
    pub sample_pool: Vec<String>,
    /// Minimum delay between events (milliseconds, for bird mode).
    pub min_delay_ms: u32,
    /// Maximum delay between events (milliseconds, for bird mode).
    pub max_delay_ms: u32,
}

impl StreamConfig {
    /// Create a new stream configuration.
    pub fn new(id: String, mode: StreamMode) -> Self {
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
