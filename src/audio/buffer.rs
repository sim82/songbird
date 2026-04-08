//! Core audio buffer abstraction.
//!
//! Represents a stereo or mono audio buffer with f32 samples.

use std::sync::Arc;

/// A stereo audio buffer containing left and right channel samples.
#[derive(Clone, Debug)]
pub struct AudioBuffer {
    /// Left channel samples (f32).
    pub left: Arc<Vec<f32>>,
    /// Right channel samples (f32).
    pub right: Arc<Vec<f32>>,
    /// Sample rate in Hz.
    pub sample_rate: u32,
    /// Current playback position (in samples).
    pub position: usize,
    /// Total number of samples in the buffer.
    pub length: usize,
}

/// Represents a playback cursor in a sample with looping support.
#[derive(Clone, Debug)]
pub struct PlaybackCursor {
    /// Current position in samples.
    pub position: usize,
    /// Whether playback has completed.
    pub finished: bool,
}

impl PlaybackCursor {
    /// Create a new cursor at position 0.
    pub fn new() -> Self {
        Self {
            position: 0,
            finished: false,
        }
    }

    /// Advance cursor by one sample.
    pub fn advance(&mut self, buffer_length: usize) {
        self.position += 1;
        if self.position >= buffer_length {
            self.finished = true;
        }
    }

    /// Advance cursor by n samples.
    pub fn advance_by(&mut self, n: usize, buffer_length: usize) {
        self.position += n;
        if self.position >= buffer_length {
            self.finished = true;
        }
    }

    /// Reset cursor to position 0.
    pub fn reset(&mut self) {
        self.position = 0;
        self.finished = false;
    }
}

impl Default for PlaybackCursor {
    fn default() -> Self {
        Self::new()
    }
}

impl AudioBuffer {
    /// Create a new stereo audio buffer.
    pub fn new_stereo(left: Vec<f32>, right: Vec<f32>, sample_rate: u32) -> Self {
        let length = left.len();
        assert_eq!(
            left.len(),
            right.len(),
            "Left and right channels must have same length"
        );

        Self {
            left: Arc::new(left),
            right: Arc::new(right),
            sample_rate,
            position: 0,
            length,
        }
    }

    /// Create a mono audio buffer (duplicated to stereo).
    pub fn new_mono(mono: Vec<f32>, sample_rate: u32) -> Self {
        let length = mono.len();
        let stereo = Arc::new(mono.clone());

        Self {
            left: stereo.clone(),
            right: stereo,
            sample_rate,
            position: 0,
            length,
        }
    }

    /// Get a sample from the left channel at the given position.
    pub fn sample_left(&self, pos: usize) -> f32 {
        if pos < self.left.len() {
            self.left[pos]
        } else {
            0.0
        }
    }

    /// Get a sample from the right channel at the given position.
    pub fn sample_right(&self, pos: usize) -> f32 {
        if pos < self.right.len() {
            self.right[pos]
        } else {
            0.0
        }
    }

    /// Get a stereo sample pair at the given position.
    pub fn sample_pair(&self, pos: usize) -> (f32, f32) {
        (self.sample_left(pos), self.sample_right(pos))
    }

    /// Check if playback has reached the end.
    pub fn is_finished(&self) -> bool {
        self.position >= self.length
    }

    /// Reset playback position to the beginning.
    pub fn reset(&mut self) {
        self.position = 0;
    }

    /// Advance playback position by one sample.
    pub fn advance(&mut self) {
        self.position += 1;
    }

    /// Advance playback position by n samples.
    pub fn advance_by(&mut self, n: usize) {
        self.position = std::cmp::min(self.position + n, self.length);
    }

    /// Get duration in seconds.
    pub fn duration_seconds(&self) -> f32 {
        self.length as f32 / self.sample_rate as f32
    }

    /// Create a new cursor at the beginning of this buffer.
    pub fn cursor(&self) -> PlaybackCursor {
        PlaybackCursor {
            position: 0,
            finished: false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mono_buffer_creation() {
        let samples = vec![0.1, 0.2, 0.3];
        let buffer = AudioBuffer::new_mono(samples, 44100);
        assert_eq!(buffer.length, 3);
        assert_eq!(buffer.sample_rate, 44100);
        assert_eq!(buffer.sample_left(0), 0.1);
        assert_eq!(buffer.sample_right(0), 0.1);
    }

    #[test]
    fn test_stereo_buffer_creation() {
        let left = vec![0.1, 0.2, 0.3];
        let right = vec![0.4, 0.5, 0.6];
        let buffer = AudioBuffer::new_stereo(left, right, 48000);
        assert_eq!(buffer.length, 3);
        assert_eq!(buffer.sample_left(1), 0.2);
        assert_eq!(buffer.sample_right(1), 0.5);
    }

    #[test]
    fn test_playback_position() {
        let buffer = AudioBuffer::new_mono(vec![0.1, 0.2, 0.3], 44100);
        let mut buf = buffer;
        assert!(!buf.is_finished());
        buf.advance();
        buf.advance();
        buf.advance();
        assert!(buf.is_finished());
    }

    #[test]
    fn test_sample_pair() {
        let left = vec![0.1, 0.2];
        let right = vec![0.4, 0.5];
        let buffer = AudioBuffer::new_stereo(left, right, 44100);
        let (l, r) = buffer.sample_pair(0);
        assert_eq!(l, 0.1);
        assert_eq!(r, 0.4);
    }

    #[test]
    fn test_duration_seconds() {
        let buffer = AudioBuffer::new_mono(vec![0.0; 44100], 44100);
        assert!((buffer.duration_seconds() - 1.0).abs() < 0.001);
    }

    #[test]
    fn test_cursor_creation() {
        let buffer = AudioBuffer::new_mono(vec![0.0; 100], 44100);
        let cursor = buffer.cursor();
        assert_eq!(cursor.position, 0);
        assert!(!cursor.finished);
    }
}
