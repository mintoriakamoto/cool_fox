//! Shared plumbing for every VIctory crate and tool.
//!
//! * [`config`]: where the user's files live (never inside the repository).
//! * [`policy`]: the runtime half of rule #1, refusing to write
//!   game-derived data into the source tree.
//! * [`logging`]: one tracing setup for all binaries.
//! * [`profiler_client`]: launcher-side profiler reader for real-time monitoring.

pub mod config;
pub mod logging;
pub mod policy;
pub mod profiler_client;
pub mod game_process;
pub mod session_persistence;

#[cfg(test)]
mod integration_test;

pub use config::Config;
pub use policy::{PolicyError, ensure_outside_repo};
pub use profiler_client::{ProfilerClient, ProfileFrame, SessionStats};
pub use game_process::{GameSession, LaunchConfig, GameMetadata};
