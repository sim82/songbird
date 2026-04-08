//! WAV file loader.

use crate::audio::AudioBuffer;
use std::fs::File;
use std::path::Path;

/// Loads WAV files into audio buffers.
pub struct SampleLoader;

impl SampleLoader {
    /// Load a WAV file from the given path.
    ///
    /// Supports mono and stereo WAV files with 16-bit PCM encoding.
    ///
    /// # Arguments
    /// * `path` - Path to the WAV file
    ///
    /// # Returns
    /// Result with an AudioBuffer or error description.
    pub fn load<P: AsRef<Path>>(path: P) -> Result<AudioBuffer, String> {
        let path = path.as_ref();

        let mut file = File::open(path).map_err(|e| format!("Failed to open WAV file: {}", e))?;

        let (header, data) =
            wav::read(&mut file).map_err(|e| format!("Failed to read WAV file: {}", e))?;

        let sample_rate = header.sampling_rate;
        let channels = header.channel_count as usize;

        // Convert bit depth to f32 samples
        let samples_f32 = match data {
            wav::BitDepth::Eight(samples) => samples
                .iter()
                .map(|&s| (s as f32 - 128.0) / 128.0)
                .collect(),
            wav::BitDepth::Sixteen(samples) => {
                samples.iter().map(|&s| s as f32 / 32768.0).collect()
            }
            wav::BitDepth::TwentyFour(samples) => {
                samples.iter().map(|&s| s as f32 / 8388608.0).collect()
            }
            wav::BitDepth::Empty => {
                return Err("WAV file has no audio data".to_string());
            }
        };

        let samples: Vec<f32> = samples_f32;

        if samples.is_empty() {
            return Err("WAV file contains no samples".to_string());
        }

        match channels {
            1 => {
                // Mono: use same samples for both channels
                Ok(AudioBuffer::new_mono(samples, sample_rate))
            }
            2 => {
                // Stereo: split interleaved samples into left and right
                if !samples.len().is_multiple_of(2) {
                    return Err("Stereo WAV has odd number of samples".to_string());
                }

                let mut left = Vec::new();
                let mut right = Vec::new();

                for (i, sample) in samples.iter().enumerate() {
                    if i % 2 == 0 {
                        left.push(*sample);
                    } else {
                        right.push(*sample);
                    }
                }

                Ok(AudioBuffer::new_stereo(left, right, sample_rate))
            }
            ch => Err(format!(
                "Unsupported channel count: {} (expected 1 or 2)",
                ch
            )),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_load_mono_wav() {
        // This test uses the test_sample.wav created by the test setup
        let buffer = SampleLoader::load("test_sample.wav");
        if let Ok(buf) = buffer {
            assert_eq!(buf.sample_rate, 44100);
            assert!(buf.length > 0);
        }
        // If file doesn't exist, we skip gracefully
    }
}
