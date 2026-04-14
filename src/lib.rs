//! Songbird: Ambient Sound Synthesis Framework
//!
//! A flexible framework for synthesizing ambient sounds by mixing multiple
//! audio voices, with deterministic continuous playback and probabilistic
//! discrete event triggering, stereo panning, and hot-reloadable configuration.

pub mod audio;
pub mod config;
pub mod samples;
pub mod synthesis;
pub mod voices;

pub use synthesis::SynthesisEngine;
