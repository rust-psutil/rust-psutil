//! Process monitoring utilities

#![feature(collections)]
#![feature(core)]
#![feature(io)]
#![feature(libc)]
#![feature(path)]
#![feature(os)]

extern crate libc;

pub mod pidfile;
pub mod process;
pub mod system;

/// Type for process identifiers
///
/// This should expand to `i32` (signed 32 bit integer).
#[stable]
pub type PID = libc::types::os::arch::posix88::pid_t;

/// Return the PID of the calling process
#[stable]
pub fn getpid() -> PID {
    unsafe { libc::funcs::posix88::unistd::getpid() }
}

/// Return the PID of the parent process
#[stable]
pub fn getppid() -> PID {
    unsafe { libc::funcs::posix88::unistd::getppid() }
}
