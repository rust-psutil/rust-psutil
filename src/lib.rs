//! Process monitoring utilities

extern crate libc;

pub mod pidfile;
pub mod process;
pub mod system;
mod utils;

/// Type for process identifiers
///
/// This should expand to `i32` (signed 32 bit integer).
pub type PID = libc::types::os::arch::posix88::pid_t;

/// Return the PID of the calling process
pub fn getpid() -> PID {
    unsafe { libc::funcs::posix88::unistd::getpid() }
}

/// Return the PID of the parent process
pub fn getppid() -> PID {
    unsafe { libc::funcs::posix88::unistd::getppid() }
}

/// Type for user identifiers
pub type UID = libc::types::os::arch::posix88::uid_t;

/// Type for group identifiers
pub type GID = libc::types::os::arch::posix88::gid_t;
