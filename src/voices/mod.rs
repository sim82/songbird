//! Voice definitions and scheduling logic.
//!
//! Voices represent independent audio sources that can operate in two modes:
//! - Continuous: Overlapping samples from a pool
//! - Discrete: Event-driven, non-overlapping samples

pub mod config;
pub mod continuous;
pub mod discrete;
pub mod manager;
pub mod state;

pub use config::VoiceConfig;
pub use continuous::{ContinuousScheduler, ScheduleEvent};
pub use discrete::{DiscreteEvent, DiscreteScheduler};
pub use manager::VoiceManager;
pub use state::VoiceState;
