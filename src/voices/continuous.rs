//! Continuous-mode voice scheduler.
//!
//! Continuous mode plays samples completely (not looped), selecting new samples
//! **deterministically** when the current sample finishes. Crossfading occurs over
//! a fixed overlap period, where the new sample fades in while the old sample fades out
//! for smooth blending.
//!
//! Key behavior:
//! - Each sample plays to completion
//! - When a sample finishes, **always** select a new random sample from the pool
//! - New sample begins fading in while old sample fades out
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
/// a new sample is **deterministically selected** from the pool (no probability).
/// The new sample and old sample overlap for a fixed crossfade period, creating
/// smooth blended transitions.
///
/// Note: There is NO probability involved in sample selection for continuous mode.
/// Probability only applies to discrete mode (event triggering). Continuous voices
/// always play new samples in succession without gaps.
#[derive(Debug)]
pub struct ContinuousScheduler {
    /// Minimum crossfade duration in samples.
    pub min_overlap: usize,
    /// Maximum crossfade duration in samples.
    pub max_overlap: usize,
}

impl ContinuousScheduler {
    /// Create a new continuous scheduler.
    pub fn new(min_overlap: usize, max_overlap: usize) -> Self {
        Self {
            min_overlap,
            max_overlap,
        }
    }

    /// Schedule the next sample event (always triggers).
    ///
    /// Called when the current sample finishes. Always returns a new event
    /// with a randomly selected sample from the pool and a random crossfade duration.
    /// There is no probability check—continuous voices **always** select a new sample.
    pub fn schedule_event(&self, pool_size: usize, rng: &mut impl Rng) -> ScheduleEvent {
        ScheduleEvent {
            sample_index: self.select_sample(pool_size, rng),
            overlap_samples: self.next_overlap(rng),
        }
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
        Self::new(1000, 5000)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_schedule_event_always_returns() {
        let scheduler = ContinuousScheduler::new(1000, 5000);
        let mut rng = rand::thread_rng();

        let event = scheduler.schedule_event(3, &mut rng);
        assert!(event.sample_index < 3);
        assert!(event.overlap_samples >= 1000);
        assert!(event.overlap_samples <= 5000);
    }

    #[test]
    fn test_select_sample() {
        let scheduler = ContinuousScheduler::default();
        let mut rng = rand::thread_rng();

        let sample_idx = scheduler.select_sample(5, &mut rng);
        assert!(sample_idx < 5);
    }

    #[test]
    fn test_select_sample_empty_pool() {
        let scheduler = ContinuousScheduler::default();
        let mut rng = rand::thread_rng();

        let sample_idx = scheduler.select_sample(0, &mut rng);
        assert_eq!(sample_idx, 0);
    }

    #[test]
    fn test_next_overlap_in_range() {
        let scheduler = ContinuousScheduler::new(2000, 8000);
        let mut rng = rand::thread_rng();

        for _ in 0..10 {
            let overlap = scheduler.next_overlap(&mut rng);
            assert!(overlap >= 2000);
            assert!(overlap <= 8000);
        }
    }
}
