//! Continuous-mode stream scheduler.
//!
//! Handles overlapping, probabilistically-selected samples.

use rand::Rng;

/// Represents a scheduled event for sample playback.
#[derive(Debug, Clone)]
pub struct ScheduleEvent {
    /// Sample index to play (from pool).
    pub sample_index: usize,
    /// Delay before playback starts (in samples).
    pub delay_samples: usize,
}

/// Continuous-mode scheduler for overlapping samples.
#[derive(Debug)]
pub struct ContinuousScheduler {
    /// Probability of selecting a new sample (0.0 to 1.0).
    pub probability: f32,
    /// Minimum overlap duration (samples).
    pub min_overlap: usize,
    /// Maximum overlap duration (samples).
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

    /// Decide whether to trigger next sample.
    pub fn should_trigger(&self, rng: &mut impl Rng) -> bool {
        rng.gen_range(0.0..1.0) < self.probability
    }

    /// Schedule next event with random parameters.
    pub fn schedule_event(&self, pool_size: usize, rng: &mut impl Rng) -> Option<ScheduleEvent> {
        if !self.should_trigger(rng) {
            return None;
        }

        Some(ScheduleEvent {
            sample_index: self.select_sample(pool_size, rng),
            delay_samples: self.next_delay(rng),
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

    /// Calculate next event delay in samples.
    pub fn next_delay(&self, rng: &mut impl Rng) -> usize {
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
        assert!(ev.delay_samples >= scheduler.min_overlap);
        assert!(ev.delay_samples <= scheduler.max_overlap);
    }
}
