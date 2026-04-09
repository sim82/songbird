//! OS audio output layer abstraction.
//!
//! Provides trait-based abstraction for platform-specific audio output with
//! real-time buffer management for synthesis integration.

/// Audio output errors.
#[derive(Debug, Clone)]
pub enum AudioError {
    /// Device initialization failed.
    DeviceInitError(String),
    /// Audio write operation failed.
    WriteError(String),
    /// Invalid configuration.
    InvalidConfig(String),
}

impl std::fmt::Display for AudioError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            AudioError::DeviceInitError(msg) => write!(f, "Device init failed: {}", msg),
            AudioError::WriteError(msg) => write!(f, "Write failed: {}", msg),
            AudioError::InvalidConfig(msg) => write!(f, "Invalid config: {}", msg),
        }
    }
}

impl std::error::Error for AudioError {}

/// Audio output format specification.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AudioFormat {
    /// Sample rate in Hz (e.g., 44100, 48000).
    pub sample_rate: u32,
    /// Number of channels (e.g., 2 for stereo).
    pub channels: u32,
}

impl AudioFormat {
    /// Create audio format with common stereo sample rate.
    pub fn new(sample_rate: u32) -> Self {
        Self {
            sample_rate,
            channels: 2, // stereo
        }
    }

    /// Check validity of audio format.
    pub fn is_valid(&self) -> bool {
        self.sample_rate > 0 && self.sample_rate <= 192_000 && self.channels > 0
    }
}

/// Trait for audio output backends.
///
/// Implementations handle platform-specific audio device management.
pub trait AudioDevice: Send + Sync {
    /// Start audio playback.
    fn start(&mut self) -> Result<(), AudioError>;

    /// Stop audio playback.
    fn stop(&mut self) -> Result<(), AudioError>;

    /// Write stereo samples to output device.
    /// Returns the number of frames actually written.
    fn write(&mut self, left: &[f32], right: &[f32]) -> Result<usize, AudioError>;

    /// Get audio format.
    fn format(&self) -> AudioFormat;

    /// Get current latency in milliseconds (approximate).
    fn latency_ms(&self) -> u32 {
        0
    }
}

/// Stub audio device for testing and platforms without audio support.
#[derive(Debug)]
pub struct StubAudioDevice {
    format: AudioFormat,
    running: bool,
}

impl StubAudioDevice {
    /// Create new stub audio device.
    pub fn new(format: AudioFormat) -> Result<Self, AudioError> {
        if !format.is_valid() {
            return Err(AudioError::InvalidConfig(
                "Invalid audio format".to_string(),
            ));
        }
        Ok(Self {
            format,
            running: false,
        })
    }
}

impl AudioDevice for StubAudioDevice {
    fn start(&mut self) -> Result<(), AudioError> {
        self.running = true;
        Ok(())
    }

    fn stop(&mut self) -> Result<(), AudioError> {
        self.running = false;
        Ok(())
    }

    fn write(&mut self, left: &[f32], right: &[f32]) -> Result<usize, AudioError> {
        if !self.running {
            return Err(AudioError::WriteError("Device not running".to_string()));
        }
        if left.len() != right.len() {
            return Err(AudioError::WriteError(
                "Channel length mismatch".to_string(),
            ));
        }
        Ok(left.len())
    }

    fn format(&self) -> AudioFormat {
        self.format
    }
}

/// Audio output manager wrapping a device backend.
///
/// Provides unified interface for starting playback, writing frames, and monitoring.
pub struct AudioOutput {
    device: Box<dyn AudioDevice>,
    buffer_left: Vec<f32>,
    buffer_right: Vec<f32>,
}

impl std::fmt::Debug for AudioOutput {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        f.debug_struct("AudioOutput")
            .field("device", &"<trait object>")
            .field("buffer_size", &self.buffer_left.len())
            .finish()
    }
}

impl AudioOutput {
    /// Create audio output with device backend.
    pub fn with_device(device: Box<dyn AudioDevice>) -> Self {
        Self {
            device,
            buffer_left: Vec::new(),
            buffer_right: Vec::new(),
        }
    }

    /// Create default stub audio output (for testing).
    pub fn stub(format: AudioFormat) -> Result<Self, AudioError> {
        let device = Box::new(StubAudioDevice::new(format)?);
        Ok(Self {
            device,
            buffer_left: Vec::new(),
            buffer_right: Vec::new(),
        })
    }

    /// Allocate internal buffers with given frame capacity.
    pub fn allocate_buffers(&mut self, frames: usize) {
        self.buffer_left.resize(frames, 0.0);
        self.buffer_right.resize(frames, 0.0);
    }

    /// Get mutable references to internal buffers for synthesis.
    pub fn buffers_mut(&mut self) -> (&mut [f32], &mut [f32]) {
        (&mut self.buffer_left, &mut self.buffer_right)
    }

    /// Get immutable references to internal buffers.
    pub fn buffers(&self) -> (&[f32], &[f32]) {
        (&self.buffer_left, &self.buffer_right)
    }

    /// Start playback.
    pub fn start(&mut self) -> Result<(), AudioError> {
        self.device.start()
    }

    /// Stop playback.
    pub fn stop(&mut self) -> Result<(), AudioError> {
        self.device.stop()
    }

    /// Write audio data directly.
    pub fn write(&mut self, left: &[f32], right: &[f32]) -> Result<usize, AudioError> {
        self.device.write(left, right)
    }

    /// Get audio format.
    pub fn format(&self) -> AudioFormat {
        self.device.format()
    }

    /// Get device latency in milliseconds.
    pub fn latency_ms(&self) -> u32 {
        self.device.latency_ms()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_audio_format_validity() {
        let valid = AudioFormat::new(44100);
        assert!(valid.is_valid());
        assert_eq!(valid.sample_rate, 44100);
        assert_eq!(valid.channels, 2);

        let invalid = AudioFormat {
            sample_rate: 0,
            channels: 2,
        };
        assert!(!invalid.is_valid());
    }

    #[test]
    fn test_audio_format_high_rate() {
        let fmt = AudioFormat::new(192_000);
        assert!(fmt.is_valid());
    }

    #[test]
    fn test_audio_format_too_high() {
        let invalid = AudioFormat {
            sample_rate: 200_000,
            channels: 2,
        };
        assert!(!invalid.is_valid());
    }

    #[test]
    fn test_stub_audio_device() {
        let format = AudioFormat::new(48000);
        let mut device = StubAudioDevice::new(format).unwrap();

        let result = device.write(&[0.1], &[0.2]);
        assert!(result.is_err()); // Not started

        device.start().unwrap();
        let result = device.write(&[0.1], &[0.2]);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 1);

        device.stop().unwrap();
        let result = device.write(&[0.1], &[0.2]);
        assert!(result.is_err()); // Stopped
    }

    #[test]
    fn test_stub_device_channel_mismatch() {
        let format = AudioFormat::new(48000);
        let mut device = StubAudioDevice::new(format).unwrap();
        device.start().unwrap();

        let result = device.write(&[0.1, 0.2], &[0.3]);
        assert!(result.is_err()); // Mismatched channels
    }

    #[test]
    fn test_audio_output_manager() {
        let format = AudioFormat::new(44100);
        let mut output = AudioOutput::stub(format).unwrap();

        output.allocate_buffers(1024);
        {
            let (left, right) = output.buffers_mut();
            left[0] = 0.1;
            right[0] = 0.2;
        }

        output.start().unwrap();
        // Copy data before calling write to avoid borrow conflict
        let data_left = vec![0.1];
        let data_right = vec![0.2];
        let _ = output.write(&data_left, &data_right);
        output.stop().unwrap();
    }

    #[test]
    fn test_audio_output_buffer_allocation() {
        let format = AudioFormat::new(44100);
        let mut output = AudioOutput::stub(format).unwrap();

        assert_eq!(output.buffer_left.len(), 0);
        output.allocate_buffers(512);
        assert_eq!(output.buffer_left.len(), 512);
        assert_eq!(output.buffer_right.len(), 512);
    }

    #[test]
    fn test_audio_error_display() {
        let err = AudioError::DeviceInitError("test".to_string());
        assert_eq!(err.to_string(), "Device init failed: test");
    }
}
