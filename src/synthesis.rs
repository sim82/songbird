//! Audio synthesis orchestration.
//!
//! Combines voice scheduling, mixing, and panning to produce final audio output.
//! Handles:
//! - Continuous mode: Sample playback with crossfading transitions
//! - Discrete mode: Event-driven scheduling with probabilistic triggers
//! - Voice mixing: Combines all active voices with panning

use crate::audio::StereoMixer;
use crate::samples::SampleCache;
use crate::voices::{ContinuousScheduler, DiscreteScheduler, VoiceConfig, VoiceMode, VoiceState};
use rand::thread_rng;

/// Per-voice crossfade state for continuous mode transitions.
#[derive(Debug, Clone)]
struct CrossfadeState {
    /// Currently fading-out sample index.
    old_sample_idx: usize,
    /// Currently fading-in sample index (new sample).
    new_sample_idx: usize,
    /// Total overlap duration in samples.
    overlap_duration: usize,
    /// Current position in the overlap (0 to overlap_duration).
    overlap_position: usize,
}

/// Synthesis state per voice, augmenting VoiceState with mode-specific data.
#[derive(Debug)]
struct VoiceSynthesisState {
    /// Base playback state.
    pub state: VoiceState,
    /// For continuous mode: crossfade state if transitioning between samples.
    pub crossfade: Option<CrossfadeState>,
}

impl VoiceSynthesisState {
    fn new(id: String) -> Self {
        Self {
            state: VoiceState::new(id),
            crossfade: None,
        }
    }
}

/// Main synthesis orchestrator combining voices, mixing, and scheduling.
pub struct SynthesisEngine {
    /// Sample cache.
    sample_cache: SampleCache,
    /// Per-voice synthesis state.
    voices: Vec<(VoiceConfig, VoiceSynthesisState)>,
    /// Sample rate in Hz.
    sample_rate: u32,
    /// Is synthesis running?
    is_running: bool,
}

impl SynthesisEngine {
    /// Create a new synthesis engine.
    pub fn new(sample_rate: u32) -> Self {
        Self {
            sample_cache: SampleCache::new(),
            voices: Vec::new(),
            sample_rate,
            is_running: false,
        }
    }

    /// Add a voice to the engine.
    pub fn add_voice(&mut self, config: VoiceConfig) {
        let synthesis_state = VoiceSynthesisState::new(config.id.clone());
        self.voices.push((config, synthesis_state));
    }

    /// Start synthesis.
    pub fn start(&mut self) {
        self.is_running = true;
        // Activate all voices when engine starts
        for (_, state) in &mut self.voices {
            state.state.is_active = true;
        }
    }

    /// Stop synthesis.
    pub fn stop(&mut self) {
        self.is_running = false;
        // Deactivate all voices when engine stops
        for (_, state) in &mut self.voices {
            state.state.is_active = false;
        }
    }

    /// Get reference to sample cache.
    pub fn sample_cache(&self) -> &SampleCache {
        &self.sample_cache
    }

    /// Get mutable reference to sample cache.
    pub fn sample_cache_mut(&mut self) -> &mut SampleCache {
        &mut self.sample_cache
    }

    /// Process one audio frame, returning stereo output (left, right).
    ///
    /// For each voice:
    /// 1. If continuous mode: play current sample, manage crossfading
    /// 2. If discrete mode: check if event should trigger, play if needed
    /// 3. Mix all voices with their respective pan settings
    pub fn process_frame(&mut self) -> (f32, f32) {
        if !self.is_running || self.voices.is_empty() {
            return (0.0, 0.0);
        }

        let mut output_samples = Vec::new();

        // Process each voice (need separate scope to avoid borrowing conflicts)
        for i in 0..self.voices.len() {
            let (config, voice_state) = &mut self.voices[i];

            if !voice_state.state.is_active {
                continue;
            }

            let sample_cache = &self.sample_cache.clone();
            let sample_rate = self.sample_rate;

            let stereo_sample = match &config.mode {
                VoiceMode::Continuous { overlap_ms } => Self::process_continuous_voice(
                    config,
                    voice_state,
                    sample_cache,
                    sample_rate,
                    *overlap_ms,
                ),
                VoiceMode::Discrete {
                    probability,
                    min_delay_ms,
                    max_delay_ms,
                } => Self::process_discrete_voice(
                    config,
                    voice_state,
                    sample_cache,
                    sample_rate,
                    *probability,
                    *min_delay_ms,
                    *max_delay_ms,
                ),
            };

            output_samples.push(stereo_sample);
        }

        // Mix all voice outputs
        StereoMixer::mix_samples(&output_samples)
    }

    /// Process a continuous-mode voice for one frame.
    fn process_continuous_voice(
        config: &VoiceConfig,
        voice_state: &mut VoiceSynthesisState,
        sample_cache: &SampleCache,
        sample_rate: u32,
        overlap_ms: u32,
    ) -> (f32, f32) {
        let sample_idx = voice_state.state.current_sample_index;
        if sample_idx >= config.sample_pool.len() {
            return (0.0, 0.0);
        }

        let sample_id = &config.sample_pool[sample_idx];
        let buffer = match sample_cache.get(sample_id) {
            Some(buf) => buf,
            None => return (0.0, 0.0),
        };

        let mut sample_value = 0.0;

        // If we're in a crossfade transition, blend both samples
        if let Some(ref mut crossfade) = voice_state.crossfade {
            let progress = crossfade.overlap_position as f32 / crossfade.overlap_duration as f32;

            // Old sample fades out
            let old_gain = 1.0 - progress;
            if crossfade.old_sample_idx < config.sample_pool.len()
                && let Some(old_buf) =
                    sample_cache.get(&config.sample_pool[crossfade.old_sample_idx])
            {
                sample_value += old_buf.sample_left(voice_state.state.playback_position) * old_gain;
            }

            // New sample fades in
            let new_gain = progress;
            if let Some(new_buf) = sample_cache.get(&config.sample_pool[crossfade.new_sample_idx]) {
                sample_value += new_buf.sample_left(voice_state.state.playback_position) * new_gain;
            }

            crossfade.overlap_position += 1;
            if crossfade.overlap_position >= crossfade.overlap_duration {
                // Crossfade complete, switch to new sample
                voice_state.state.current_sample_index = crossfade.new_sample_idx;
                voice_state.crossfade = None;
            }
        } else {
            // Normal playback (not in crossfade)
            sample_value = buffer.sample_left(voice_state.state.playback_position);
        }

        // Advance playback position
        voice_state.state.playback_position += 1;

        // Check if current sample finished
        if voice_state.state.playback_position >= buffer.length && voice_state.crossfade.is_none() {
            // Schedule next sample
            let mut rng = thread_rng();
            let scheduler = ContinuousScheduler::new(500, 5000);
            let next_event = scheduler.schedule_event(config.sample_pool.len(), &mut rng);
            println!("next event: {next_event:?}");

            // Start crossfade
            let overlap_samples = (sample_rate as f32 * overlap_ms as f32 / 1000.0) as usize;
            voice_state.crossfade = Some(CrossfadeState {
                old_sample_idx: voice_state.state.current_sample_index,
                new_sample_idx: next_event.sample_index,
                overlap_duration: next_event.overlap_samples.min(overlap_samples),
                overlap_position: 0,
            });
            voice_state.state.playback_position = 0;
        }

        // Apply panning and return stereo output
        let mut mixer = StereoMixer::new();
        mixer.set_pan(config.pan);
        mixer.apply_pan(sample_value)
    }

    /// Process a discrete-mode voice for one frame.
    fn process_discrete_voice(
        config: &VoiceConfig,
        voice_state: &mut VoiceSynthesisState,
        sample_cache: &SampleCache,
        sample_rate: u32,
        probability: f32,
        min_delay_ms: u32,
        max_delay_ms: u32,
    ) -> (f32, f32) {
        let mut sample_value = 0.0;

        // If waiting for next event, count down
        if voice_state.state.next_event_countdown > 0 {
            voice_state.state.next_event_countdown -= 1;
        } else {
            // Time to trigger a new event
            let mut rng = thread_rng();
            // Convert ms to samples
            let min_delay_samples = (sample_rate as f32 * min_delay_ms as f32 / 1000.0) as usize;
            let max_delay_samples = (sample_rate as f32 * max_delay_ms as f32 / 1000.0) as usize;
            let scheduler =
                DiscreteScheduler::new(probability, min_delay_samples, max_delay_samples);

            if let Some(event) = scheduler.schedule_event(config.sample_pool.len(), &mut rng, sample_rate as usize) {
                println!("discrete event: {event:?}");
                // Start playing the triggered sample
                voice_state.state.current_sample_index = event.sample_index;
                voice_state.state.playback_position = 0;
                // Schedule next event trigger after this one finishes plus delay
                voice_state.state.next_event_countdown = event.delay_samples;
            }
        }

        // If currently playing a sample, output its current frame
        let sample_idx = voice_state.state.current_sample_index;
        if sample_idx < config.sample_pool.len() {
            if let Some(buffer) = sample_cache.get(&config.sample_pool[sample_idx]) {
                if voice_state.state.playback_position < buffer.length {
                    sample_value = buffer.sample_left(voice_state.state.playback_position);
                    voice_state.state.playback_position += 1;
                } else {
                    // Sample finished, stop playback
                    voice_state.state.current_sample_index = config.sample_pool.len();
                }
            }
        }

        // Apply panning
        let mut mixer = StereoMixer::new();
        mixer.set_pan(config.pan);
        mixer.apply_pan(sample_value)
    }

    /// Get number of voices.
    pub fn voice_count(&self) -> usize {
        self.voices.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_synthesis_engine_creation() {
        let engine = SynthesisEngine::new(44100);
        assert_eq!(engine.voice_count(), 0);
        assert!(!engine.is_running);
    }

    #[test]
    fn test_add_voice() {
        let mut engine = SynthesisEngine::new(44100);
        let config = VoiceConfig::new_continuous("test".to_string(), 500);
        engine.add_voice(config);
        assert_eq!(engine.voice_count(), 1);
    }

    #[test]
    fn test_start_stop() {
        let mut engine = SynthesisEngine::new(44100);
        assert!(!engine.is_running);
        engine.start();
        assert!(engine.is_running);
        engine.stop();
        assert!(!engine.is_running);
    }

    #[test]
    fn test_process_frame_no_voices() {
        let mut engine = SynthesisEngine::new(44100);
        engine.start();
        let (left, right) = engine.process_frame();
        assert_eq!(left, 0.0);
        assert_eq!(right, 0.0);
    }

    #[test]
    fn test_process_frame_inactive_voice() {
        let mut engine = SynthesisEngine::new(44100);
        let config = VoiceConfig::new_continuous("test".to_string(), 500);
        engine.add_voice(config);
        engine.start();

        let (left, right) = engine.process_frame();
        assert_eq!(left, 0.0);
        assert_eq!(right, 0.0);
    }
}
