//! Read information from the /proc filesystem

#![feature(collections)]
#![feature(core)]
#![feature(io)]
#![feature(libc)]
#![feature(path)]

extern crate libc;

pub mod process;
pub mod system;

/// Type for process identifiers
#[stable]
pub type PID = i32;

/// Get the PID of the current process
#[stable]
pub fn getpid() -> PID { unsafe { libc::getpid() } }
