//! Stereo audio mixer with panning support.

/// Stereo mixer for combining multiple audio sources with panning.
#[derive(Debug, Clone, Copy)]
pub struct StereoMixer {
    /// Pan position: -1.0 (left) to 1.0 (right), 0.0 (center).
    pub pan: f32,
}

impl StereoMixer {
    /// Create a new mixer with center panning.
    pub fn new() -> Self {
        Self { pan: 0.0 }
    }

    /// Set pan position (-1.0 left, 0.0 center, 1.0 right).
    pub fn set_pan(&mut self, pan: f32) {
        self.pan = pan.clamp(-1.0, 1.0);
    }

    /// Apply panning to a mono sample, returning (left, right) stereo output.
    pub fn apply_pan(&self, sample: f32) -> (f32, f32) {
        // Linear panning: as we pan right, left decreases and right increases
        let left_gain = (1.0 - self.pan) / 2.0;
        let right_gain = (1.0 + self.pan) / 2.0;

        (sample * left_gain, sample * right_gain)
    }

    /// Mix multiple samples additively.
    pub fn mix_samples(samples: &[(f32, f32)]) -> (f32, f32) {
        let mut left = 0.0;
        let mut right = 0.0;

        for (l, r) in samples {
            left += l;
            right += r;
        }

        (left, right)
    }
}

impl Default for StereoMixer {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_center_pan() {
        let mixer = StereoMixer::new();
        let (left, right) = mixer.apply_pan(1.0);
        assert!((left - 0.5).abs() < 0.001);
        assert!((right - 0.5).abs() < 0.001);
    }

    #[test]
    fn test_left_pan() {
        let mixer = StereoMixer { pan: -1.0 };
        let (left, right) = mixer.apply_pan(1.0);
        assert!(left > right);
    }

    #[test]
    fn test_right_pan() {
        let mixer = StereoMixer { pan: 1.0 };
        let (left, right) = mixer.apply_pan(1.0);
        assert!(right > left);
    }
}
