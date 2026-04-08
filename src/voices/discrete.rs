//! Discrete-mode voice scheduler.
//!
//! Handles event-driven, non-overlapping sample playback.

use rand::Rng;

/// Represents a scheduled discrete event for sample playback.
#[derive(Debug, Clone)]
pub struct DiscreteEvent {
    /// Sample index to play (from pool).
    pub sample_index: usize,
    /// Delay before playback starts (in samples).
    pub delay_samples: usize,
}

/// Discrete-mode scheduler for event-driven, non-overlapping samples.
#[derive(Debug)]
pub struct DiscreteScheduler {
    /// Probability of triggering an event (0.0 to 1.0).
    pub probability: f32,
    /// Minimum delay between events (samples).
    pub min_delay: usize,
    /// Maximum delay between events (samples).
    pub max_delay: usize,
}

impl DiscreteScheduler {
    /// Create a new discrete scheduler.
    pub fn new(probability: f32, min_delay: usize, max_delay: usize) -> Self {
        Self {
            probability: probability.clamp(0.0, 1.0),
            min_delay,
            max_delay,
        }
    }

    /// Decide whether to trigger an event.
    pub fn should_trigger(&self, rng: &mut impl Rng) -> bool {
        rng.gen_range(0.0..1.0) < self.probability
    }

    /// Schedule next discrete event.
    pub fn schedule_event(&self, pool_size: usize, rng: &mut impl Rng) -> Option<DiscreteEvent> {
        if !self.should_trigger(rng) {
            return None;
        }

        Some(DiscreteEvent {
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

    /// Calculate delay until next event (samples).
    pub fn next_delay(&self, rng: &mut impl Rng) -> usize {
        rng.gen_range(self.min_delay..=self.max_delay)
    }
}

impl Default for DiscreteScheduler {
    fn default() -> Self {
        Self::new(0.1, 22050, 88200) // 0.5 to 2 seconds at 44.1 kHz
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_discrete_trigger_probability() {
        let scheduler = DiscreteScheduler::new(1.0, 100, 200); // Always trigger
        let mut rng = rand::thread_rng();
        assert!(scheduler.should_trigger(&mut rng));

        let scheduler = DiscreteScheduler::new(0.0, 100, 200); // Never trigger
        assert!(!scheduler.should_trigger(&mut rng));
    }

    #[test]
    fn test_discrete_schedule_event() {
        let scheduler = DiscreteScheduler::new(1.0, 1000, 5000);
        let mut rng = rand::thread_rng();

        let event = scheduler.schedule_event(2, &mut rng);
        assert!(event.is_some());
        let ev = event.unwrap();
        assert!(ev.sample_index < 2);
        assert!(ev.delay_samples >= 1000);
        assert!(ev.delay_samples <= 5000);
    }
}
