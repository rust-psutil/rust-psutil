//! Process monitoring utilities

#![feature(collections)]
#![feature(core)]
#![feature(io)]
#![feature(libc)]
#![feature(path)]

extern crate libc;

pub mod pidfile;
pub mod process;
pub mod system;

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
