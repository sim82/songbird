//! ALSA audio output backend for Linux.
//!
//! Provides real-time PCM playback via ALSA (Advanced Linux Sound Architecture).
//! Requires libalsa-dev on the system.

#[cfg(feature = "alsa")]
mod alsa_impl {
    use crate::audio::{AudioDevice, AudioError, AudioFormat};
    use alsa::Direction;
    use alsa::pcm::{Access, Format, HwParams, PCM};
    use std::sync::Mutex;

    /// Wrapper to make ALSA PCM Send + Sync for use in AudioDevice trait.
    struct AlsaPcmWrapper(Mutex<PCM>);

    impl AlsaPcmWrapper {
        fn new(pcm: PCM) -> Self {
            Self(Mutex::new(pcm))
        }

        fn with_pcm<F, R>(&self, f: F) -> Result<R, String>
        where
            F: FnOnce(&PCM) -> Result<R, String>,
        {
            let pcm = self.0.lock().unwrap();
            f(&pcm)
        }
    }

    // ALSA handles synchronization internally; it's safe to mark as Send + Sync
    unsafe impl Send for AlsaPcmWrapper {}
    unsafe impl Sync for AlsaPcmWrapper {}

    /// ALSA PCM audio device.
    pub struct AlsaDevice {
        pcm: AlsaPcmWrapper,
        format: AudioFormat,
        buffer_size: u32,
    }

    impl AlsaDevice {
        /// Create new ALSA device.
        pub fn new(device_name: &str, format: AudioFormat) -> Result<Self, AudioError> {
            if !format.is_valid() {
                return Err(AudioError::InvalidConfig(
                    "Invalid audio format".to_string(),
                ));
            }

            // Open PCM device for playback
            let pcm = PCM::new(device_name, Direction::Playback, false).map_err(|e| {
                AudioError::DeviceInitError(format!("Failed to open PCM device: {}", e))
            })?;

            // Set up hardware parameters
            {
                let hwp = HwParams::any(&pcm).map_err(|e| {
                    AudioError::DeviceInitError(format!("Failed to create hw params: {}", e))
                })?;

                hwp.set_access(Access::RWInterleaved).map_err(|e| {
                    AudioError::DeviceInitError(format!("Failed to set access: {}", e))
                })?;

                hwp.set_format(Format::S16LE).map_err(|e| {
                    AudioError::DeviceInitError(format!("Failed to set format: {}", e))
                })?;

                hwp.set_channels(format.channels).map_err(|e| {
                    AudioError::DeviceInitError(format!("Failed to set channels: {}", e))
                })?;

                hwp.set_rate(format.sample_rate, alsa::ValueOr::Nearest)
                    .map_err(|e| {
                        AudioError::DeviceInitError(format!("Failed to set sample rate: {}", e))
                    })?;

                // Set buffer and period sizes
                let buffer_size = 4096u32;
                let period_size = 1024u32;

                hwp.set_buffer_size(buffer_size as alsa::pcm::Frames)
                    .map_err(|e| {
                        AudioError::DeviceInitError(format!("Failed to set buffer size: {}", e))
                    })?;

                hwp.set_period_size(period_size as alsa::pcm::Frames, alsa::ValueOr::Nearest)
                    .map_err(|e| {
                        AudioError::DeviceInitError(format!("Failed to set period size: {}", e))
                    })?;

                pcm.hw_params(&hwp).map_err(|e| {
                    AudioError::DeviceInitError(format!("Failed to apply hw params: {}", e))
                })?;
            } // hwp dropped here

            // Prepare for playback
            pcm.prepare().map_err(|e| {
                AudioError::DeviceInitError(format!("Failed to prepare PCM: {}", e))
            })?;

            Ok(Self {
                pcm: AlsaPcmWrapper::new(pcm),
                format,
                buffer_size: 4096,
            })
        }

        /// Get default ALSA device name.
        pub fn default_device() -> &'static str {
            "default"
        }
    }

    impl AudioDevice for AlsaDevice {
        fn start(&mut self) -> Result<(), AudioError> {
            self.pcm
                .with_pcm(|pcm| {
                    // Start PCM in non-blocking mode
                    pcm.start()
                        .map_err(|e| format!("Failed to start PCM: {}", e))
                })
                .map_err(AudioError::DeviceInitError)?;
            Ok(())
        }

        fn stop(&mut self) -> Result<(), AudioError> {
            self.pcm
                .with_pcm(|pcm| {
                    pcm.drain()
                        .map_err(|e| format!("Failed to drain PCM: {}", e))
                })
                .map_err(AudioError::WriteError)?;
            Ok(())
        }

        fn write(&mut self, left: &[f32], right: &[f32]) -> Result<usize, AudioError> {
            if left.len() != right.len() {
                return Err(AudioError::WriteError(
                    "Channel length mismatch".to_string(),
                ));
            }

            let frames_to_write = left.len();
            let mut interleaved = Vec::with_capacity(frames_to_write * 2);

            // Interleave left and right channels and convert f32 to i16
            for i in 0..frames_to_write {
                let left_i16 = (left[i].clamp(-1.0, 1.0) * 32767.0) as i16;
                let right_i16 = (right[i].clamp(-1.0, 1.0) * 32767.0) as i16;
                interleaved.push(left_i16);
                interleaved.push(right_i16);
            }

            // Write interleaved samples
            self.pcm
                .with_pcm(|pcm| {
                    let io = pcm
                        .io_i16()
                        .map_err(|e| format!("Failed to get IO: {}", e))?;
                    io.writei(&interleaved)
                        .map_err(|e| format!("Write failed: {}", e))
                })
                .map_err(AudioError::WriteError)
        }

        fn format(&self) -> AudioFormat {
            self.format
        }

        fn latency_ms(&self) -> u32 {
            // Approximate latency in milliseconds
            let frames = (self.buffer_size as f64 / self.format.sample_rate as f64) * 1000.0;
            frames as u32
        }
    }
}

#[cfg(feature = "alsa")]
pub use alsa_impl::AlsaDevice;

/// Create ALSA audio device with default settings.
#[cfg(feature = "alsa")]
pub fn create_alsa_device(
    format: crate::audio::AudioFormat,
) -> Result<Box<dyn crate::audio::AudioDevice>, crate::audio::AudioError> {
    let device = AlsaDevice::new(AlsaDevice::default_device(), format)?;
    Ok(Box::new(device))
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_default_device_name() {
        #[cfg(feature = "alsa")]
        {
            use super::alsa_impl::AlsaDevice;
            assert_eq!(AlsaDevice::default_device(), "default");
        }
    }
}
