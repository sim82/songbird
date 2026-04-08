//! Audio buffer primitives and audio processing infrastructure.

pub mod buffer;
pub mod mixer;
pub mod output;

pub use buffer::{AudioBuffer, PlaybackCursor};
