//! Audio buffer primitives and audio processing infrastructure.

pub mod buffer;
pub mod mixer;
pub mod output;

pub use buffer::{AudioBuffer, PlaybackCursor};
pub use mixer::StereoMixer;
pub use output::{AudioDevice, AudioError, AudioFormat, AudioOutput, StubAudioDevice};
