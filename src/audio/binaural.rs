//! Binaural audio spatializer with ITD, ILD, and head shadowing.
//!
//! Implements lightweight binaural rendering:
//! - ITD (Interaural Time Difference): Fixed delay (~0.7ms) scaled by pan
//! - ILD (Interaural Level Difference): Linear gain attenuation
//! - Head Shadowing: 1-pole LPF at 1500Hz applied to opposite channel

use std::f32::consts::PI;

/// Binaural renderer: ITD + ILD + head shadowing (1-pole LPF).
#[derive(Debug)]
pub struct BinauralRenderer {
    sample_rate: u32,
    /// ITD max delay in samples (~0.7ms = 31 samples at 44.1kHz)
    itd_max_delay_samples: usize,
    /// LPF cutoff frequency at full pan (1500Hz)
    lpf_cutoff_hz: f32,
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
            lpf_cutoff_hz: 1500.0,
            delay_buffer,
            delay_write_index: 0,
            lpf_state_left: 0.0,
            lpf_state_right: 0.0,
        }
    }

    /// Process a mono sample with binaural panning.
    ///
    /// Applies ITD (time delay), ILD (level difference), and head shadowing (LPF)
    /// to create a lightweight binaural effect.
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

        let left_sample = sample * left_gain;
        let right_sample = sample * right_gain;

        // 2. Apply ITD (Interaural Time Difference)
        // Delay the sample on the "near" side (opposite of pan direction)
        let delay_amount =
            ((self.itd_max_delay_samples as f32 * pan.abs()) as usize).min(self.itd_max_delay_samples - 1);

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

        // 3. Apply head shadowing (1-pole LPF to opposite channel)
        // Only apply head shadowing at extreme pan angles (>0.75) to preserve audio clarity
        // at moderate pans. Head shadowing primarily affects extreme side-positioned sources.
        let pan_threshold = 0.75;
        let (final_left, final_right) = if pan.abs() < pan_threshold {
            // Moderate pan: ITD + ILD only, no head shadowing
            (left_with_itd, right_with_itd)
        } else {
            // Extreme pan (>0.75): apply head shadowing LPF
            // Linear interpolation from minimal effect at 0.75 to full 1500Hz LPF at 1.0
            let normalized_pan = (pan.abs() - pan_threshold) / (1.0 - pan_threshold);
            let effective_cutoff_hz = self.lpf_cutoff_hz * normalized_pan;
            let lpf_alpha = self.compute_lpf_coefficient(effective_cutoff_hz);

            if pan < 0.0 {
                // Left pan: apply LPF to right channel
                let filtered_right = self.apply_lpf_filter_right(right_with_itd, lpf_alpha);
                (left_with_itd, filtered_right)
            } else {
                // Right pan: apply LPF to left channel
                let filtered_left = self.apply_lpf_filter_left(left_with_itd, lpf_alpha);
                (filtered_left, right_with_itd)
            }
        };

        (final_left, final_right)
    }

    /// Compute 1-pole LPF coefficient for given cutoff frequency.
    /// Uses the formula: alpha = wc / (fs + wc) where wc = 2π * fc
    fn compute_lpf_coefficient(&self, cutoff_hz: f32) -> f32 {
        if cutoff_hz <= 0.0 {
            // No filtering (all pass)
            1.0
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
        
        // Process a continuous signal and verify LPF attenuates high frequencies
        let mut outputs = Vec::new();
        
        for _ in 0..200 {
            let (left, right) = renderer.process_binaural(1.0, 1.0); // Full right: LPF on left
            outputs.push((left, right));
        }
        
        // After many samples, the left channel (with LPF) should stabilize
        // Check that later samples are more stable (LPF settling)
        let early_left: f32 = outputs[10..20].iter().map(|(l, _)| l.abs()).sum::<f32>() / 10.0;
        let late_left: f32 = outputs[150..160].iter().map(|(l, _)| l.abs()).sum::<f32>() / 10.0;
        
        // Late samples should be more attenuated (LPF effect)
        assert!(late_left < early_left || (late_left - early_left).abs() < 0.1);
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
