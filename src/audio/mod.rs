//! Audio buffer primitives and audio processing infrastructure.

pub mod buffer;
pub mod mixer;
pub mod output;

#[cfg(feature = "alsa")]
pub mod alsa;

pub use buffer::{AudioBuffer, PlaybackCursor};
pub use mixer::StereoMixer;
pub use output::{AudioDevice, AudioError, AudioFormat, AudioOutput, StubAudioDevice};

#[cfg(feature = "alsa")]
pub use alsa::create_alsa_device;
