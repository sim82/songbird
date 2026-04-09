//! Audio buffer primitives and audio processing infrastructure.

pub mod buffer;
pub mod mixer;
pub mod output;
pub mod wav_writer;

#[cfg(feature = "alsa")]
pub mod alsa;

pub use buffer::{AudioBuffer, PlaybackCursor};
pub use mixer::StereoMixer;
pub use output::{AudioDevice, AudioError, AudioFormat, AudioOutput, StubAudioDevice};
pub use wav_writer::WavWriter;

#[cfg(feature = "alsa")]
pub use alsa::create_alsa_device;
