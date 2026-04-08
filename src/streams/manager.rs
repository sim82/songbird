//! Main stream manager coordinating multiple concurrent streams.

use crate::streams::{StreamConfig, StreamState};
use crate::samples::SampleCache;

/// Manages multiple concurrent streams with probabilistic scheduling.
pub struct StreamManager {
    /// Active streams and their configurations.
    streams: Vec<(StreamConfig, StreamState)>,
    /// Reference to the sample cache.
    sample_cache: SampleCache,
}

impl StreamManager {
    /// Create a new stream manager.
    pub fn new(sample_cache: SampleCache) -> Self {
        Self {
            streams: Vec::new(),
            sample_cache,
        }
    }

    /// Add a stream to the manager.
    pub fn add_stream(&mut self, config: StreamConfig) {
        let state = StreamState::new(config.id.clone());
        self.streams.push((config, state));
    }

    /// Get the number of active streams.
    pub fn stream_count(&self) -> usize {
        self.streams.len()
    }

    /// Get a mutable reference to a stream by ID.
    pub fn get_stream_mut(&mut self, id: &str) -> Option<&mut (StreamConfig, StreamState)> {
        self.streams.iter_mut().find(|(cfg, _)| cfg.id == id)
    }

    /// Get a reference to a stream by ID.
    pub fn get_stream(&self, id: &str) -> Option<&(StreamConfig, StreamState)> {
        self.streams.iter().find(|(cfg, _)| cfg.id == id)
    }

    /// Get reference to the sample cache.
    pub fn sample_cache(&self) -> &SampleCache {
        &self.sample_cache
    }

    /// Get mutable reference to the sample cache.
    /// Returns stereo output (left, right) for this frame.
    pub fn process_frame(&mut self) -> (f32, f32) {
        let left_out = 0.0;
        let right_out = 0.0;

        // For now, stub implementation
        // Will be fully implemented with schedulers in this phase
        (left_out, right_out)
    }

    /// Reset all streams to initial state.
    pub fn reset(&mut self) {
        for (_, state) in &mut self.streams {
            state.reset();
        }
    }

    /// Start all streams.
    pub fn start_all(&mut self) {
        for (_, state) in &mut self.streams {
            state.is_active = true;
        }
    }

    /// Stop all streams.
    pub fn stop_all(&mut self) {
        for (_, state) in &mut self.streams {
            state.is_active = false;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::streams::config::StreamMode;

    #[test]
    fn test_manager_creation() {
        let cache = SampleCache::new();
        let manager = StreamManager::new(cache);
        assert_eq!(manager.stream_count(), 0);
    }

    #[test]
    fn test_add_stream() {
        let cache = SampleCache::new();
        let mut manager = StreamManager::new(cache);
        let config = StreamConfig::new("test".to_string(), StreamMode::Continuous);
        manager.add_stream(config);
        assert_eq!(manager.stream_count(), 1);
    }

    #[test]
    fn test_get_stream() {
        let cache = SampleCache::new();
        let mut manager = StreamManager::new(cache);
        let config = StreamConfig::new("test".to_string(), StreamMode::Continuous);
        manager.add_stream(config);

        let stream = manager.get_stream("test");
        assert!(stream.is_some());
        let (cfg, _) = stream.unwrap();
        assert_eq!(cfg.id, "test");
    }

    #[test]
    fn test_start_stop_all() {
        let cache = SampleCache::new();
        let mut manager = StreamManager::new(cache);
        let config = StreamConfig::new("test".to_string(), StreamMode::Continuous);
        manager.add_stream(config);

        manager.start_all();
        let (_, state) = manager.get_stream("test").unwrap();
        assert!(state.is_active);

        manager.stop_all();
        let (_, state) = manager.get_stream("test").unwrap();
        assert!(!state.is_active);
    }
}
