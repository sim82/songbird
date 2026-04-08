//! Main voice manager coordinating multiple concurrent voices.

use crate::samples::SampleCache;
use crate::voices::{VoiceConfig, VoiceState};

/// Manages multiple concurrent voices with probabilistic scheduling.
pub struct VoiceManager {
    /// Active voices and their configurations.
    voices: Vec<(VoiceConfig, VoiceState)>,
    /// Reference to the sample cache.
    sample_cache: SampleCache,
}

impl VoiceManager {
    /// Create a new voice manager.
    pub fn new(sample_cache: SampleCache) -> Self {
        Self {
            voices: Vec::new(),
            sample_cache,
        }
    }

    /// Add a voice to the manager.
    pub fn add_voice(&mut self, config: VoiceConfig) {
        let state = VoiceState::new(config.id.clone());
        self.voices.push((config, state));
    }

    /// Get the number of active voices.
    pub fn voice_count(&self) -> usize {
        self.voices.len()
    }

    /// Get a mutable reference to a voice by ID.
    pub fn get_voice_mut(&mut self, id: &str) -> Option<&mut (VoiceConfig, VoiceState)> {
        self.voices.iter_mut().find(|(cfg, _)| cfg.id == id)
    }

    /// Get a reference to a voice by ID.
    pub fn get_voice(&self, id: &str) -> Option<&(VoiceConfig, VoiceState)> {
        self.voices.iter().find(|(cfg, _)| cfg.id == id)
    }

    /// Process all voices for one sample frame.
    /// Returns stereo output (left, right) for this frame.
    pub fn process_frame(&mut self) -> (f32, f32) {
        let left_out = 0.0;
        let right_out = 0.0;

        // For now, stub implementation
        // Will be fully implemented with schedulers in Phase 4
        (left_out, right_out)
    }

    /// Reset all voices to initial state.
    pub fn reset(&mut self) {
        for (_, state) in &mut self.voices {
            state.reset();
        }
    }

    /// Start all voices.
    pub fn start_all(&mut self) {
        for (_, state) in &mut self.voices {
            state.is_active = true;
        }
    }

    /// Stop all voices.
    pub fn stop_all(&mut self) {
        for (_, state) in &mut self.voices {
            state.is_active = false;
        }
    }

    /// Get reference to the sample cache.
    pub fn sample_cache(&self) -> &SampleCache {
        &self.sample_cache
    }

    /// Get mutable reference to the sample cache.
    pub fn sample_cache_mut(&mut self) -> &mut SampleCache {
        &mut self.sample_cache
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::voices::config::VoiceMode;

    #[test]
    fn test_manager_creation() {
        let cache = SampleCache::new();
        let manager = VoiceManager::new(cache);
        assert_eq!(manager.voice_count(), 0);
    }

    #[test]
    fn test_add_voice() {
        let cache = SampleCache::new();
        let mut manager = VoiceManager::new(cache);
        let config = VoiceConfig::new("test".to_string(), VoiceMode::Continuous);
        manager.add_voice(config);
        assert_eq!(manager.voice_count(), 1);
    }

    #[test]
    fn test_get_voice() {
        let cache = SampleCache::new();
        let mut manager = VoiceManager::new(cache);
        let config = VoiceConfig::new("test".to_string(), VoiceMode::Continuous);
        manager.add_voice(config);

        let voice = manager.get_voice("test");
        assert!(voice.is_some());
        let (cfg, _) = voice.unwrap();
        assert_eq!(cfg.id, "test");
    }

    #[test]
    fn test_start_stop_all() {
        let cache = SampleCache::new();
        let mut manager = VoiceManager::new(cache);
        let config = VoiceConfig::new("test".to_string(), VoiceMode::Continuous);
        manager.add_voice(config);

        manager.start_all();
        let (_, state) = manager.get_voice("test").unwrap();
        assert!(state.is_active);

        manager.stop_all();
        let (_, state) = manager.get_voice("test").unwrap();
        assert!(!state.is_active);
    }
}
