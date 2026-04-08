//! Per-stream playback state.

use std::collections::HashMap;

/// Playback state for a single stream.
#[derive(Debug, Clone)]
pub struct StreamState {
    /// Stream identifier.
    pub id: String,
    /// Currently playing sample index (in the sample pool).
    pub current_sample_index: usize,
    /// Playback position within the current sample (in samples).
    pub playback_position: usize,
    /// Remaining time until next event trigger (in samples).
    pub next_event_countdown: usize,
    /// Is the stream currently active?
    pub is_active: bool,
    /// Metadata for continuous mode scheduling.
    pub metadata: HashMap<String, f32>,
}

impl StreamState {
    /// Create a new stream state.
    pub fn new(id: String) -> Self {
        Self {
            id,
            current_sample_index: 0,
            playback_position: 0,
            next_event_countdown: 0,
            is_active: false,
            metadata: HashMap::new(),
        }
    }

    /// Reset to initial state.
    pub fn reset(&mut self) {
        self.current_sample_index = 0;
        self.playback_position = 0;
        self.next_event_countdown = 0;
        self.is_active = false;
        self.metadata.clear();
    }
}
