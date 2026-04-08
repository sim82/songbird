//! Stream definitions and scheduling logic.
//!
//! Streams represent independent audio sources that can operate in two modes:
//! - Continuous: Overlapping samples from a pool
//! - Bird: Event-driven, non-overlapping samples

pub mod bird;
pub mod config;
pub mod continuous;
pub mod manager;
pub mod state;

pub use bird::{BirdEvent, BirdScheduler};
pub use config::StreamConfig;
pub use continuous::{ContinuousScheduler, ScheduleEvent};
pub use manager::StreamManager;
pub use state::StreamState;
