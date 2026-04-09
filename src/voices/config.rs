//! Voice configuration types.

/// Voice playback mode with mode-specific configuration.
#[derive(Debug, Clone)]
pub enum VoiceMode {
    /// Continuous overlapping samples.
    /// Plays samples completely, **always** selecting new random samples when current finishes.
    /// Transitions use fixed-duration crossfading for smooth blending.
    Continuous {
        /// Crossfade/overlap duration in milliseconds.
        overlap_ms: u32,
    },
    /// Discrete (event-driven), non-overlapping samples.
    /// Individual samples triggered probabilistically at random intervals.
    Discrete {
        /// Probability of triggering a new event (0.0 to 1.0).
        probability: f32,
        /// Minimum delay between events in milliseconds.
        min_delay_ms: u32,
        /// Maximum delay between events in milliseconds.
        max_delay_ms: u32,
    },
}

impl VoiceMode {
    /// Create a continuous mode with default overlap duration.
    pub fn continuous(overlap_ms: u32) -> Self {
        VoiceMode::Continuous { overlap_ms }
    }

    /// Create a discrete mode with trigger probability and delay range.
    pub fn discrete(probability: f32, min_delay_ms: u32, max_delay_ms: u32) -> Self {
        VoiceMode::Discrete {
            probability: probability.clamp(0.0, 1.0),
            min_delay_ms,
            max_delay_ms,
        }
    }

    /// Check if this is continuous mode.
    pub fn is_continuous(&self) -> bool {
        matches!(self, VoiceMode::Continuous { .. })
    }

    /// Check if this is discrete mode.
    pub fn is_discrete(&self) -> bool {
        matches!(self, VoiceMode::Discrete { .. })
    }
}

impl PartialEq for VoiceMode {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (VoiceMode::Continuous { overlap_ms: a }, VoiceMode::Continuous { overlap_ms: b }) => {
                a == b
            }
            (
                VoiceMode::Discrete {
                    probability: p1,
                    min_delay_ms: m1,
                    max_delay_ms: x1,
                },
                VoiceMode::Discrete {
                    probability: p2,
                    min_delay_ms: m2,
                    max_delay_ms: x2,
                },
            ) => (p1 - p2).abs() < f32::EPSILON && m1 == m2 && x1 == x2,
            _ => false,
        }
    }
}

impl Eq for VoiceMode {}

/// Configuration for a single voice.
#[derive(Debug, Clone, PartialEq)]
pub struct VoiceConfig {
    /// Unique identifier for the voice.
    pub id: String,
    /// Voice playback mode with mode-specific configuration.
    pub mode: VoiceMode,
    /// Pan position (-1.0 left to 1.0 right).
    pub pan: f32,
    /// Sample pool: list of sample identifiers.
    pub sample_pool: Vec<String>,
}

impl VoiceConfig {
    /// Create a new continuous voice configuration.
    pub fn new_continuous(id: String, overlap_ms: u32) -> Self {
        Self {
            id,
            mode: VoiceMode::continuous(overlap_ms),
            pan: 0.0,
            sample_pool: vec![],
        }
    }

    /// Create a new discrete voice configuration.
    pub fn new_discrete(
        id: String,
        probability: f32,
        min_delay_ms: u32,
        max_delay_ms: u32,
    ) -> Self {
        Self {
            id,
            mode: VoiceMode::discrete(probability, min_delay_ms, max_delay_ms),
            pan: 0.0,
            sample_pool: vec![],
        }
    }

    /// Generic constructor (for compatibility).
    pub fn new(id: String, mode: VoiceMode) -> Self {
        Self {
            id,
            mode,
            pan: 0.0,
            sample_pool: vec![],
        }
    }
}
