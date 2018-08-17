//! Process monitoring utilities.

#[macro_use]
extern crate lazy_static;
extern crate libc;

#[macro_use]
mod utils;

pub mod disk;
pub mod pidfile;
pub mod process;
pub mod system;
/// Type for process identifiers.
///
/// This should expand to `i32` (signed 32 bit integer).
pub type PID = libc::pid_t;

/// Return the PID of the calling process.
pub fn getpid() -> PID {
    unsafe { libc::getpid() }
}

/// Return the PID of the parent process.
pub fn getppid() -> PID {
    unsafe { libc::getppid() }
}

/// Type for user identifiers.
pub type UID = libc::uid_t;

/// Type for group identifiers.
pub type GID = libc::gid_t;
