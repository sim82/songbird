//! OS audio output layer abstraction.
//!
//! Placeholder for platform-specific audio output implementation.

/// Abstract audio output device.
#[derive(Debug)]
pub struct AudioOutput {
    /// Sample rate in Hz.
    pub sample_rate: u32,
}

impl AudioOutput {
    /// Create a new audio output device.
    pub fn new(sample_rate: u32) -> Self {
        Self { sample_rate }
    }

    /// Start playback (stub).
    pub fn start(&mut self) {
        // TODO: Implement platform-specific audio initialization
    }

    /// Stop playback (stub).
    pub fn stop(&mut self) {
        // TODO: Implement platform-specific audio teardown
    }

    /// Write audio data to the output (stub).
    pub fn write(&mut self, _left: &[f32], _right: &[f32]) {
        // TODO: Implement platform-specific audio write
    }
}
