//! Continuous-mode voice scheduler.
//!
//! Continuous mode plays samples completely (not looped), selecting new samples randomly
//! when the current sample finishes. Crossfading occurs over a fixed overlap period,
//! where the new sample fades in while the old sample fades out for smooth blending.
//!
//! Key behavior:
//! - Each sample plays to completion
//! - At sample completion, scheduler decides whether to pick a new sample (based on probability)
//! - If new sample is selected, it begins fading in while old sample fades out
//! - Overlap period is fixed and defined in voice configuration
//! - After overlap, the new sample fully plays until its completion

use rand::Rng;

/// Represents a scheduled event for continuous-mode sample playback.
#[derive(Debug, Clone)]
pub struct ScheduleEvent {
    /// Sample index to play next (from pool).
    pub sample_index: usize,
    /// Overlap/crossfade duration in samples (fade new sample in, old sample out).
    pub overlap_samples: usize,
}

/// Continuous-mode scheduler for overlapping samples.
///
/// Manages the transition between samples with crossfading. When a sample finishes,
/// the scheduler probabilistically selects the next sample. The new sample and old sample
/// overlap for a fixed crossfade period, creating smooth blended transitions.
#[derive(Debug)]
pub struct ContinuousScheduler {
    /// Probability of selecting a new sample when current finishes (0.0 to 1.0).
    pub probability: f32,
    /// Minimum crossfade duration in samples.
    pub min_overlap: usize,
    /// Maximum crossfade duration in samples.
    pub max_overlap: usize,
}

impl ContinuousScheduler {
    /// Create a new continuous scheduler.
    pub fn new(probability: f32) -> Self {
        Self {
            probability: probability.clamp(0.0, 1.0),
            min_overlap: 1000,
            max_overlap: 5000,
        }
    }

    /// Decide whether to trigger next sample based on probability.
    ///
    /// Called when current sample finishes. Returns true if a new sample should start
    /// (with crossfade overlap), false if playback should pause/idle.
    pub fn should_trigger(&self, rng: &mut impl Rng) -> bool {
        rng.gen_range(0.0..1.0) < self.probability
    }

    /// Schedule next sample event with random crossfade parameters.
    ///
    /// Returns a new event if the scheduler decides to trigger (based on probability),
    /// None otherwise. When an event is returned, the new sample and old sample will
    /// overlap and crossfade for the specified `overlap_samples` duration.
    pub fn schedule_event(&self, pool_size: usize, rng: &mut impl Rng) -> Option<ScheduleEvent> {
        if !self.should_trigger(rng) {
            return None;
        }

        Some(ScheduleEvent {
            sample_index: self.select_sample(pool_size, rng),
            overlap_samples: self.next_overlap(rng),
        })
    }

    /// Select a random sample index from the pool.
    pub fn select_sample(&self, pool_size: usize, rng: &mut impl Rng) -> usize {
        if pool_size == 0 {
            0
        } else {
            rng.gen_range(0..pool_size)
        }
    }

    /// Calculate next crossfade duration in samples.
    ///
    /// Called when scheduling a new sample. Returns a random duration within the
    /// configured min/max overlap range for sample transition.
    pub fn next_overlap(&self, rng: &mut impl Rng) -> usize {
        rng.gen_range(self.min_overlap..=self.max_overlap)
    }
}

impl Default for ContinuousScheduler {
    fn default() -> Self {
        Self::new(0.5)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_scheduler_trigger_probability() {
        let scheduler = ContinuousScheduler::new(1.0); // Always trigger
        let mut rng = rand::thread_rng();
        assert!(scheduler.should_trigger(&mut rng));

        let scheduler = ContinuousScheduler::new(0.0); // Never trigger
        assert!(!scheduler.should_trigger(&mut rng));
    }

    #[test]
    fn test_schedule_event() {
        let scheduler = ContinuousScheduler::new(1.0);
        let mut rng = rand::thread_rng();

        let event = scheduler.schedule_event(3, &mut rng);
        assert!(event.is_some());
        let ev = event.unwrap();
        assert!(ev.sample_index < 3);
        assert!(ev.overlap_samples >= scheduler.min_overlap);
        assert!(ev.overlap_samples <= scheduler.max_overlap);
    }
}
