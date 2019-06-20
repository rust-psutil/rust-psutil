//! Process monitoring utilities.

#[macro_use]
mod utils;

#[cfg(target_os = "linux")]
mod linux;
#[cfg(target_os = "linux")]
pub use linux::*;

/// Type for process identifiers.
///
/// This should expand to `i32` (signed 32 bit integer).
pub type PID = libc::pid_t;

/// Type for user identifiers.
pub type UID = libc::uid_t;

/// Type for group identifiers.
pub type GID = libc::gid_t;
