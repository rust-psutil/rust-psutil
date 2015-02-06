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

// This should be moved to a better named module
#[unstable]
pub mod errno {
    use libc::types::os::arch::c95::c_int;

    extern {
        fn __errno_location() -> *const c_int;
    }

    pub fn errno() -> i32 {
        unsafe { *__errno_location() as i32 }
    }
}
