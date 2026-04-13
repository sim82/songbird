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

    /// Decide whether to trigger an event during a single sample tick.
    ///
    /// `probability` is interpreted as a per-second probability (e.g. 0.15 = 15% per second).
    /// This method converts that to a per-sample probability using the provided sample_rate.
    pub fn should_trigger(&self, rng: &mut impl Rng, sample_rate: usize) -> bool {
        if self.probability <= 0.0 {
            return false;
        }
        if (self.probability - 1.0).abs() < std::f32::EPSILON {
            return true;
        }
        // Convert per-second probability to per-sample probability:
        // 1 - (1 - p_per_sample)^(sample_rate) = p_per_second
        // => p_per_sample = 1 - (1 - p_per_second)^(1 / sample_rate)
        let p_sec = self.probability.clamp(0.0, 1.0);
        let p_sample = 1.0 - (1.0 - p_sec).powf(1.0 / sample_rate as f32);
        rng.gen_range(0.0..1.0) < p_sample
    }

    /// Schedule next discrete event.
    ///
    /// `sample_rate` is required so triggering probability can be evaluated per-sample.
    pub fn schedule_event(
        &self,
        pool_size: usize,
        rng: &mut impl Rng,
        sample_rate: usize,
    ) -> Option<DiscreteEvent> {
        if !self.should_trigger(rng, sample_rate) {
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
        // Default: 10% per second, 0.5 to 2 seconds delay (in samples at 44.1kHz)
        Self::new(0.1, 22050, 88200)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_discrete_trigger_probability() {
        let sample_rate = 44100usize;
        let scheduler = DiscreteScheduler::new(1.0, 100, 200); // Always trigger (per-second = 1.0)
        let mut rng = rand::thread_rng();
        assert!(scheduler.should_trigger(&mut rng, sample_rate));

        let scheduler = DiscreteScheduler::new(0.0, 100, 200); // Never trigger
        assert!(!scheduler.should_trigger(&mut rng, sample_rate));
    }

    #[test]
    fn test_discrete_schedule_event() {
        let sample_rate = 44100usize;
        let scheduler = DiscreteScheduler::new(1.0, 1000, 5000);
        let mut rng = rand::thread_rng();

        let event = scheduler.schedule_event(2, &mut rng, sample_rate);
        assert!(event.is_some());
        let ev = event.unwrap();
        assert!(ev.sample_index < 2);
        assert!(ev.delay_samples >= 1000);
        assert!(ev.delay_samples <= 5000);
    }
}
