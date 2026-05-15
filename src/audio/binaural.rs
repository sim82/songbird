//! Binaural audio spatializer with ITD, ILD, and head shadowing.
//!
//! Implements lightweight binaural rendering:
//! - ITD (Interaural Time Difference): Fixed delay (~0.7ms) scaled by pan
//! - ILD (Interaural Level Difference): Linear gain attenuation for amplitude balance
//! - Head Shadowing: Frequency-dependent filtering on each channel
//!   - Near-side ear: high cutoff (~22kHz, minimal filtering)
//!   - Far-side ear: gradually lowers from 22kHz to 1500Hz as pan increases

use std::f32::consts::PI;

/// Binaural renderer: ITD + ILD + head shadowing (frequency-dependent filtering).
#[derive(Debug)]
pub struct BinauralRenderer {
    sample_rate: u32,
    /// ITD max delay in samples (~0.7ms = 31 samples at 44.1kHz)
    itd_max_delay_samples: usize,
    /// LPF min cutoff frequency on far-side ear (1500Hz)
    lpf_min_cutoff_hz: f32,
    /// LPF max cutoff frequency on near-side ear (22kHz, effectively no filtering)
    lpf_max_cutoff_hz: f32,
    /// Circular delay buffer for ITD (stores one channel's delayed history)
    delay_buffer: Vec<f32>,
    /// Current write position in delay buffer
    delay_write_index: usize,
    /// 1-pole LPF state for left channel
    lpf_state_left: f32,
    /// 1-pole LPF state for right channel
    lpf_state_right: f32,
}

impl BinauralRenderer {
    /// Create a new binaural renderer for the given sample rate.
    pub fn new(sample_rate: u32) -> Self {
        // ITD: ~0.7ms typical head width = 0.7 / 1000 * sample_rate samples
        let itd_max_delay_samples = ((sample_rate as f32 * 0.0007) as usize).max(1);

        // Allocate delay buffer for max ITD delay
        let delay_buffer = vec![0.0; itd_max_delay_samples];

        Self {
            sample_rate,
            itd_max_delay_samples,
            lpf_min_cutoff_hz: 1500.0,  // Far-side ear (shadowed)
            lpf_max_cutoff_hz: 22000.0, // Near-side ear (direct sound)
            delay_buffer,
            delay_write_index: 0,
            lpf_state_left: 0.0,
            lpf_state_right: 0.0,
        }
    }

    /// Process a mono sample with binaural panning.
    ///
    /// Applies ITD (time delay), ILD (level difference), and head shadowing (frequency-dependent LPF)
    /// to create a lightweight binaural effect.
    ///
    /// Head shadowing model:
    /// - Each channel (ear) has its own LPF cutoff based on whether it's the near or far side
    /// - Near-side ear: cutoff ~ 22kHz (minimal filtering, direct sound path)
    /// - Far-side ear: cutoff gradually lowers from 22kHz to 1500Hz as pan magnitude increases
    /// - ILD (panning gain) is independent of filtering
    ///
    /// - `sample`: mono input sample
    /// - `pan`: pan position (-1.0 left to 1.0 right, 0.0 center)
    ///
    /// Returns stereo output (left, right).
    pub fn process_binaural(&mut self, sample: f32, pan: f32) -> (f32, f32) {
        let pan = pan.clamp(-1.0, 1.0);

        // 1. Apply ILD (Interaural Level Difference)
        // Linear panning: left_gain = (1 - pan) / 2, right_gain = (1 + pan) / 2
        let left_gain = (1.0 - pan) / 2.0;
        let right_gain = (1.0 + pan) / 2.0;

        let left_gain = 1.0;
        let right_gain = 1.0;

        let left_sample = sample * left_gain;
        let right_sample = sample * right_gain;

        // 2. Apply ITD (Interaural Time Difference)
        // Delay the sample on the "near" side (opposite of pan direction)
        let delay_amount = ((self.itd_max_delay_samples as f32 * pan.abs()) as usize)
            .min(self.itd_max_delay_samples - 1);

        let delayed_sample = if delay_amount > 0 {
            let read_index = (self.delay_write_index + self.itd_max_delay_samples - delay_amount)
                % self.itd_max_delay_samples;
            self.delay_buffer[read_index]
        } else {
            sample
        };

        // Write current sample to delay buffer for future reads
        self.delay_buffer[self.delay_write_index] = sample;
        self.delay_write_index = (self.delay_write_index + 1) % self.itd_max_delay_samples;

        // Choose which channel gets delayed based on pan direction
        let (left_with_itd, right_with_itd) = if pan < 0.0 {
            // Left pan: delay right channel
            (left_sample, delayed_sample * right_gain)
        } else if pan > 0.0 {
            // Right pan: delay left channel
            (delayed_sample * left_gain, right_sample)
        } else {
            // Center: no ITD delay
            (left_sample, right_sample)
        };

        // 3. Apply head shadowing: frequency-dependent filtering on each channel
        // Each channel gets a cutoff frequency based on whether it's near-side or far-side
        // Left channel cutoff:
        //   if pan < 0 (left pan): left is near-side -> high cutoff (22kHz)
        //   if pan > 0 (right pan): left is far-side -> cutoff scales with |pan| from 22kHz to 1500Hz
        // Right channel cutoff:
        //   if pan > 0 (right pan): right is near-side -> high cutoff (22kHz)
        //   if pan < 0 (left pan): right is far-side -> cutoff scales with |pan| from 22kHz to 1500Hz
        let abs_pan = pan.abs();

        // Left channel: high cutoff if left is near-side (pan <= 0), low if far-side (pan > 0)
        let left_cutoff_hz = if pan <= 0.0 {
            self.lpf_max_cutoff_hz // Near-side: 22kHz
        } else {
            // Far-side: interpolate from 22kHz at pan=0 to 1500Hz at pan=1.0
            self.lpf_max_cutoff_hz - (self.lpf_max_cutoff_hz - self.lpf_min_cutoff_hz) * abs_pan
        };

        // Right channel: high cutoff if right is near-side (pan >= 0), low if far-side (pan < 0)
        let right_cutoff_hz = if pan >= 0.0 {
            self.lpf_max_cutoff_hz // Near-side: 22kHz
        } else {
            // Far-side: interpolate from 22kHz at pan=0 to 1500Hz at pan=-1.0
            self.lpf_max_cutoff_hz - (self.lpf_max_cutoff_hz - self.lpf_min_cutoff_hz) * abs_pan
        };

        // Apply LPF to each channel with its respective cutoff
        let left_lpf_alpha = self.compute_lpf_coefficient(left_cutoff_hz);
        let right_lpf_alpha = self.compute_lpf_coefficient(right_cutoff_hz);

        let final_left = self.apply_lpf_filter_left(left_with_itd, left_lpf_alpha);
        let final_right = self.apply_lpf_filter_right(right_with_itd, right_lpf_alpha);

        (final_left, final_right)
    }

    /// Compute 1-pole LPF coefficient for given cutoff frequency.
    /// Uses the formula: alpha = wc / (fs + wc) where wc = 2π * fc
    /// High cutoff values (e.g., 22kHz) result in alpha ≈ 1.0, meaning minimal filtering.
    /// Low cutoff values (e.g., 1500Hz) result in alpha < 1.0, causing significant attenuation.
    fn compute_lpf_coefficient(&self, cutoff_hz: f32) -> f32 {
        if cutoff_hz <= 0.0 {
            1.0 // Infinite cutoff: no filtering (passthrough)
        } else {
            let wc = 2.0 * PI * cutoff_hz;
            wc / (self.sample_rate as f32 + wc)
        }
    }

    /// Apply 1-pole LPF to left channel: y = alpha * x + (1 - alpha) * y_prev
    fn apply_lpf_filter_left(&mut self, input: f32, alpha: f32) -> f32 {
        let output = alpha * input + (1.0 - alpha) * self.lpf_state_left;
        self.lpf_state_left = output;
        output
    }

    /// Apply 1-pole LPF to right channel: y = alpha * x + (1 - alpha) * y_prev
    fn apply_lpf_filter_right(&mut self, input: f32, alpha: f32) -> f32 {
        let output = alpha * input + (1.0 - alpha) * self.lpf_state_right;
        self.lpf_state_right = output;
        output
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_binaural_creation() {
        let renderer = BinauralRenderer::new(44100);
        assert_eq!(renderer.sample_rate, 44100);
        // ITD: ~0.7ms at 44.1kHz = approximately 30-31 samples
        assert!(renderer.delay_buffer.len() >= 30);
    }

    #[test]
    fn test_center_pan_no_effect() {
        let mut renderer = BinauralRenderer::new(44100);
        let (left, right) = renderer.process_binaural(1.0, 0.0);
        // Center pan: both channels equal, no ITD, no LPF
        assert!((left - 0.5).abs() < 0.001);
        assert!((right - 0.5).abs() < 0.001);
    }

    #[test]
    fn test_full_left_pan() {
        let mut renderer = BinauralRenderer::new(44100);
        let (left, right) = renderer.process_binaural(1.0, -1.0);
        // Full left: left louder than right
        assert!(left > right);
        // Left should be closer to 1.0, right closer to 0.0
        assert!(left > 0.8);
        assert!(right < 0.2);
    }

    #[test]
    fn test_full_right_pan() {
        let mut renderer = BinauralRenderer::new(44100);
        let (left, right) = renderer.process_binaural(1.0, 1.0);
        // Full right: right louder than left
        assert!(right > left);
        // Right should be closer to 1.0, left closer to 0.0
        assert!(right > 0.8);
        assert!(left < 0.2);
    }

    #[test]
    fn test_itd_delay_application() {
        let mut renderer = BinauralRenderer::new(44100);
        // Process several samples to fill delay buffer
        for _ in 0..100 {
            let _ = renderer.process_binaural(1.0, 0.5);
        }

        // After filling buffer, check that delay is working
        // (exact verification is complex, just ensure no panic)
        let (left, right) = renderer.process_binaural(0.5, 0.5);
        assert!(left >= 0.0 && left <= 1.0);
        assert!(right >= 0.0 && right <= 1.0);
    }

    #[test]
    fn test_lpf_attenuation() {
        let mut renderer = BinauralRenderer::new(44100);

        // At full right pan (pan=1.0):
        //   Left channel (far-side): low cutoff (1500Hz), will be filtered
        //   Right channel (near-side): high cutoff (22kHz), minimal filtering
        // The high-frequency content should be preserved in right channel
        let mut left_outputs = Vec::new();
        let mut right_outputs = Vec::new();

        for _ in 0..200 {
            let (left, right) = renderer.process_binaural(1.0, 1.0); // Full right pan
            left_outputs.push(left);
            right_outputs.push(right);
        }

        // Right channel should stabilize around 0.5 (with minimal LPF impact)
        let right_mid: f32 = right_outputs[100..110].iter().sum::<f32>() / 10.0;
        assert!(right_mid > 0.4); // Should remain close to 0.5

        // Left channel should be attenuated due to LPF
        let left_mid: f32 = left_outputs[100..110].iter().sum::<f32>() / 10.0;
        assert!(left_mid < right_mid); // Filtered channel should be lower
    }

    #[test]
    fn test_pan_sweep() {
        let mut renderer = BinauralRenderer::new(44100);

        // Sweep pan from -1 to 1 and verify smooth transitions
        for pan_int in -100..=100 {
            let pan = pan_int as f32 / 100.0;
            let (left, right) = renderer.process_binaural(1.0, pan);

            // Basic sanity checks
            assert!(left >= 0.0);
            assert!(right >= 0.0);
            assert!(left <= 1.0);
            assert!(right <= 1.0);

            // Verify total power is roughly preserved (with some headroom for filtering)
            let power = left * left + right * right;
            assert!(power > 0.2); // Relaxed: filtering and ITD can affect energy slightly
        }
    }

    #[test]
    fn test_lpf_coefficient_bounds() {
        let renderer = BinauralRenderer::new(44100);

        // Test coefficient computation at various frequencies
        let alpha_zero = renderer.compute_lpf_coefficient(0.0);
        assert_eq!(alpha_zero, 1.0); // No filtering

        let alpha_1500 = renderer.compute_lpf_coefficient(1500.0);
        assert!(alpha_1500 > 0.0 && alpha_1500 < 1.0);
    }
}
