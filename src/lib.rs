//! Songbird: Ambient Sound Synthesis Framework
//!
//! A flexible framework for synthesizing ambient sounds by mixing multiple
//! audio streams, with probabilistic sample scheduling, stereo panning,
//! and hot-reloadable configuration.

pub mod audio;
pub mod config;
pub mod engine;
pub mod samples;
pub mod voices;

pub use engine::Engine;
