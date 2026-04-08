//! Main synthesis engine orchestrating streams, mixing, and output.

use crate::audio::AudioBuffer;
use crate::samples::SampleCache;
use crate::streams::{StreamConfig, StreamState};

/// Main synthesis engine coordinating audio playback.
#[derive(Debug)]
pub struct Engine {
    /// Sample cache.
    pub sample_cache: SampleCache,
    /// Active streams.
    pub streams: Vec<StreamState>,
    /// Stream configurations.
    pub stream_configs: Vec<StreamConfig>,
    /// Sample rate in Hz.
    pub sample_rate: u32,
    /// Is the engine running?
    pub is_running: bool,
}

impl Engine {
    /// Create a new synthesis engine.
    pub fn new(sample_rate: u32) -> Self {
        Self {
            sample_cache: SampleCache::new(),
            streams: Vec::new(),
            stream_configs: Vec::new(),
            sample_rate,
            is_running: false,
        }
    }

    /// Add a stream configuration to the engine.
    pub fn add_stream(&mut self, config: StreamConfig) {
        let state = StreamState::new(config.id.clone());
        self.stream_configs.push(config);
        self.streams.push(state);
    }

    /// Start the synthesis engine.
    pub fn start(&mut self) {
        self.is_running = true;
    }

    /// Stop the synthesis engine.
    pub fn stop(&mut self) {
        self.is_running = false;
    }

    /// Process one frame of audio (stub for now).
    pub fn process_frame(&mut self) -> (f32, f32) {
        // TODO: Implement actual synthesis logic
        (0.0, 0.0)
    }

    /// Load a sample into the cache.
    pub fn load_sample(&mut self, id: String, buffer: AudioBuffer) {
        let mut cache = self.sample_cache.clone();
        cache.add(id, buffer);
        self.sample_cache = cache;
    }

    /// Get a sample from the cache.
    pub fn get_sample(&self, id: &str) -> Option<AudioBuffer> {
        self.sample_cache.get(id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::streams::config::StreamMode;

    #[test]
    fn test_engine_creation() {
        let engine = Engine::new(44100);
        assert_eq!(engine.sample_rate, 44100);
        assert!(!engine.is_running);
        assert!(engine.streams.is_empty());
    }

    #[test]
    fn test_add_stream() {
        let mut engine = Engine::new(44100);
        let config = StreamConfig::new("test".to_string(), StreamMode::Continuous);
        engine.add_stream(config);

        assert_eq!(engine.streams.len(), 1);
        assert_eq!(engine.stream_configs.len(), 1);
    }

    #[test]
    fn test_start_stop() {
        let mut engine = Engine::new(44100);
        assert!(!engine.is_running);

        engine.start();
        assert!(engine.is_running);

        engine.stop();
        assert!(!engine.is_running);
    }
}
