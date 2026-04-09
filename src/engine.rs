//! Main synthesis engine orchestrating voices, mixing, and output.

use crate::audio::AudioBuffer;
use crate::samples::SampleCache;
use crate::voices::{VoiceConfig, VoiceState};

/// Main synthesis engine coordinating audio playback.
#[derive(Debug)]
pub struct Engine {
    /// Sample cache.
    pub sample_cache: SampleCache,
    /// Active voices.
    pub voices: Vec<VoiceState>,
    /// Voice configurations.
    pub voice_configs: Vec<VoiceConfig>,
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
            voices: Vec::new(),
            voice_configs: Vec::new(),
            sample_rate,
            is_running: false,
        }
    }

    /// Add a voice configuration to the engine.
    pub fn add_voice(&mut self, config: VoiceConfig) {
        let state = VoiceState::new(config.id.clone());
        self.voice_configs.push(config);
        self.voices.push(state);
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

    #[test]
    fn test_engine_creation() {
        let engine = Engine::new(44100);
        assert_eq!(engine.sample_rate, 44100);
        assert!(!engine.is_running);
        assert!(engine.voices.is_empty());
    }

    #[test]
    fn test_add_voice() {
        let mut engine = Engine::new(44100);
        let config = VoiceConfig::new_continuous("test".to_string(), 500);
        engine.add_voice(config);

        assert_eq!(engine.voices.len(), 1);
        assert_eq!(engine.voice_configs.len(), 1);
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
