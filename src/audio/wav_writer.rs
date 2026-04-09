//! WAV file writer audio output backend.
//!
//! Writes audio synthesis output directly to a WAV file on disk.
//! Useful for rendering long ambient pieces or batch processing without
//! real-time playback.

use crate::audio::{AudioDevice, AudioError, AudioFormat};
use std::fs::File;
use std::io::{BufWriter, Seek, SeekFrom, Write};
use std::path::Path;

/// WAV file writer for audio output.
///
/// Implements AudioDevice to write stereo samples to a WAV file.
/// Automatically finalizes the WAV header on drop.
pub struct WavWriter {
    writer: Option<BufWriter<File>>,
    format: AudioFormat,
    frames_written: u32,
    file_path: String,
}

impl WavWriter {
    /// Create a new WAV writer that will write to the specified file.
    pub fn new<P: AsRef<Path>>(path: P, format: AudioFormat) -> Result<Self, AudioError> {
        if !format.is_valid() {
            return Err(AudioError::InvalidConfig(
                "Invalid audio format".to_string(),
            ));
        }

        let file_path = path
            .as_ref()
            .to_str()
            .ok_or_else(|| AudioError::DeviceInitError("Invalid file path".to_string()))?
            .to_string();

        let file = File::create(&file_path).map_err(|e| {
            AudioError::DeviceInitError(format!("Failed to create WAV file: {}", e))
        })?;

        let writer = BufWriter::new(file);

        Ok(Self {
            writer: Some(writer),
            format,
            frames_written: 0,
            file_path,
        })
    }

    /// Get the path of the output WAV file.
    pub fn file_path(&self) -> &str {
        &self.file_path
    }

    /// Get the number of frames written so far.
    pub fn frames_written(&self) -> u32 {
        self.frames_written
    }

    /// Finalize the WAV file and close it.
    fn finalize(&mut self) -> Result<(), AudioError> {
        if let Some(writer) = self.writer.take() {
            let mut file = writer
                .into_inner()
                .map_err(|e| AudioError::WriteError(format!("Failed to flush WAV file: {}", e)))?;

            let data_size = self.frames_written * self.format.channels * 2; // 16-bit = 2 bytes
            let file_size = 36 + data_size; // 44-byte header - 8 for "RIFF" and size field

            // Seek to position 4 (file size in RIFF header)
            file.seek(SeekFrom::Start(4))
                .map_err(|e| AudioError::WriteError(format!("Seek failed: {}", e)))?;

            file.write_all(&file_size.to_le_bytes())
                .map_err(|e| AudioError::WriteError(format!("Failed to write file size: {}", e)))?;

            // Seek to position 40 (data chunk size)
            file.seek(SeekFrom::Start(40))
                .map_err(|e| AudioError::WriteError(format!("Seek failed: {}", e)))?;

            file.write_all(&data_size.to_le_bytes())
                .map_err(|e| AudioError::WriteError(format!("Failed to write data size: {}", e)))?;

            file.sync_all()
                .map_err(|e| AudioError::WriteError(format!("Failed to sync file: {}", e)))?;
        }

        Ok(())
    }

    /// Write WAV header with placeholder sizes.
    fn write_header(&mut self) -> Result<(), AudioError> {
        if let Some(writer) = &mut self.writer {
            // RIFF header
            writer
                .write_all(b"RIFF")
                .map_err(|e| AudioError::WriteError(format!("Failed to write RIFF: {}", e)))?;

            // File size placeholder (will update on finalize)
            writer
                .write_all(&0u32.to_le_bytes())
                .map_err(|e| AudioError::WriteError(format!("Failed to write size: {}", e)))?;

            // WAVE format
            writer
                .write_all(b"WAVE")
                .map_err(|e| AudioError::WriteError(format!("Failed to write WAVE: {}", e)))?;

            // fmt subchunk
            writer
                .write_all(b"fmt ")
                .map_err(|e| AudioError::WriteError(format!("Failed to write fmt: {}", e)))?;

            // Subchunk1Size (16 for PCM)
            writer.write_all(&16u32.to_le_bytes()).map_err(|e| {
                AudioError::WriteError(format!("Failed to write subchunk size: {}", e))
            })?;

            // Audio format (1 = PCM)
            writer.write_all(&1u16.to_le_bytes()).map_err(|e| {
                AudioError::WriteError(format!("Failed to write audio format: {}", e))
            })?;

            // Number of channels (2 for stereo)
            writer
                .write_all(&(self.format.channels as u16).to_le_bytes())
                .map_err(|e| AudioError::WriteError(format!("Failed to write channels: {}", e)))?;

            // Sample rate
            writer
                .write_all(&self.format.sample_rate.to_le_bytes())
                .map_err(|e| {
                    AudioError::WriteError(format!("Failed to write sample rate: {}", e))
                })?;

            // Bytes per second (sample_rate * channels * 2)
            let byte_rate = self.format.sample_rate * self.format.channels * 2;
            writer
                .write_all(&byte_rate.to_le_bytes())
                .map_err(|e| AudioError::WriteError(format!("Failed to write byte rate: {}", e)))?;

            // Block align (channels * 2)
            let block_align = (self.format.channels * 2) as u16;
            writer.write_all(&block_align.to_le_bytes()).map_err(|e| {
                AudioError::WriteError(format!("Failed to write block align: {}", e))
            })?;

            // Bits per sample (16)
            writer.write_all(&16u16.to_le_bytes()).map_err(|e| {
                AudioError::WriteError(format!("Failed to write bits per sample: {}", e))
            })?;

            // data subchunk
            writer
                .write_all(b"data")
                .map_err(|e| AudioError::WriteError(format!("Failed to write data: {}", e)))?;

            // Data size placeholder (will update on finalize)
            writer
                .write_all(&0u32.to_le_bytes())
                .map_err(|e| AudioError::WriteError(format!("Failed to write data size: {}", e)))?;

            writer.flush().map_err(|e| {
                AudioError::WriteError(format!("Failed to flush WAV header: {}", e))
            })?;
        }

        Ok(())
    }
}

impl AudioDevice for WavWriter {
    fn start(&mut self) -> Result<(), AudioError> {
        self.write_header()
    }

    fn stop(&mut self) -> Result<(), AudioError> {
        self.finalize()
    }

    fn write(&mut self, left: &[f32], right: &[f32]) -> Result<usize, AudioError> {
        if left.len() != right.len() {
            return Err(AudioError::WriteError(
                "Channel length mismatch".to_string(),
            ));
        }

        let frames_to_write = left.len();
        if frames_to_write == 0 {
            return Ok(0);
        }

        if let Some(writer) = &mut self.writer {
            let mut interleaved = Vec::with_capacity(frames_to_write * 2 * 2); // 2 channels, 2 bytes per sample

            // Interleave left and right channels and convert f32 to i16
            for i in 0..frames_to_write {
                let left_i16 = (left[i].clamp(-1.0, 1.0) * 32767.0) as i16;
                let right_i16 = (right[i].clamp(-1.0, 1.0) * 32767.0) as i16;

                interleaved.extend_from_slice(&left_i16.to_le_bytes());
                interleaved.extend_from_slice(&right_i16.to_le_bytes());
            }

            writer
                .write_all(&interleaved)
                .map_err(|e| AudioError::WriteError(format!("Failed to write samples: {}", e)))?;

            self.frames_written += frames_to_write as u32;
            Ok(frames_to_write)
        } else {
            Err(AudioError::WriteError(
                "WAV writer not initialized".to_string(),
            ))
        }
    }

    fn format(&self) -> AudioFormat {
        self.format
    }

    fn latency_ms(&self) -> u32 {
        0
    }
}

impl Drop for WavWriter {
    fn drop(&mut self) {
        // Attempt to finalize on drop
        let _ = self.finalize();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_wav_writer_creation() {
        let temp_dir = std::env::temp_dir();
        let path = temp_dir.join("test_songbird_wav.wav");

        let format = AudioFormat::new(44100);
        let result = WavWriter::new(&path, format);

        assert!(result.is_ok());
        let writer = result.unwrap();
        assert_eq!(writer.file_path(), path.to_str().unwrap());
        assert_eq!(writer.frames_written(), 0);

        // Cleanup
        let _ = std::fs::remove_file(&path);
    }

    #[test]
    fn test_wav_writer_invalid_path() {
        let format = AudioFormat::new(44100);
        let result = WavWriter::new("/nonexistent/path/that/does/not/exist/file.wav", format);

        assert!(result.is_err());
    }

    #[test]
    fn test_wav_writer_invalid_format() {
        let temp_dir = std::env::temp_dir();
        let path = temp_dir.join("test_invalid_format.wav");

        let invalid_format = AudioFormat {
            sample_rate: 0,
            channels: 2,
        };
        let result = WavWriter::new(&path, invalid_format);

        assert!(result.is_err());
    }

    #[test]
    fn test_wav_writer_finalization() {
        let temp_dir = std::env::temp_dir();
        let path = temp_dir.join("test_wav_finalization.wav");

        let format = AudioFormat::new(44100);
        let mut writer = WavWriter::new(&path, format).expect("Failed to create writer");

        // Start writing
        writer.start().expect("Failed to start");

        // Write some test data
        let left = vec![0.5; 1000];
        let right = vec![0.3; 1000];
        let written = writer.write(&left, &right).expect("Failed to write");
        assert_eq!(written, 1000);

        // Stop (finalize)
        writer.stop().expect("Failed to stop");

        // Check file was written and header updated
        let file_size = std::fs::metadata(&path).expect("Failed to stat file").len();

        let mut file = std::fs::File::open(&path).expect("Failed to open");
        use std::io::Read;
        let mut header = [0u8; 44];
        file.read_exact(&mut header).expect("Failed to read header");

        let riff_size = u32::from_le_bytes([header[4], header[5], header[6], header[7]]);
        let data_size = u32::from_le_bytes([header[40], header[41], header[42], header[43]]);

        assert!(riff_size > 0, "RIFF size should be non-zero");
        assert!(data_size > 0, "data size should be non-zero");
        assert_eq!(riff_size as u64, file_size - 8);
        assert_eq!(data_size as u64, file_size - 44);

        // Cleanup
        let _ = std::fs::remove_file(&path);
    }
}
