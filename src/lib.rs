//! A process and system monitoring library for Rust, heavily inspired by the [`psutil`](https://psutil.readthedocs.io/en/latest/#) module for Python.
//!
//! # Minimum versions supported
//!
//! - Linux: 2.6.0
//!
//! # Note about the API
//!
//! `rust-psutil` implements the same API as `psutil` with some exceptions:
//!
//! - some things have been slightly renamed
//! - the crate is namespaced based on subsystem, e.g. `cpu::cpu_percent()`
//!     - users can opt into which subsystems to use based on cargo feature flags
//! - some functions have been refactored
//!     - e.g. `cpu_count(bool)` into `cpu_count()` and `cpu_count_physical()`
//! - functions that need to persist data between calls are implemented as methods on 'collectors'
//!     - e.g. `cpu_percent()` -> `CpuPercentCollector::cpu_percent()`
//! - platform specific functionality is hidden behind traits that need to be imported before used
//!     - e.g. import `cpu::os::linux::ProcessExt` to use Linux specific process functionality
//! - some types are different, for example:
//!     - structs instead of named tuples
//!     - `std::time::Duration` instead of float for seconds
//!     - enums instead of constants
//! - most struct fields have been replaced with getter methods to better enable platform based extensions

#[macro_use]
mod utils;

#[cfg(feature = "cpu")]
pub mod cpu;

#[cfg(feature = "disk")]
pub mod disk;

#[cfg(feature = "host")]
pub mod host;

#[cfg(feature = "memory")]
pub mod memory;

#[cfg(feature = "network")]
pub mod network;

#[cfg(feature = "process")]
pub mod process;

#[cfg(feature = "sensors")]
pub mod sensors;

#[cfg(target_family = "unix")]
mod unix;

#[cfg(target_family = "unix")]
use unix::*;

mod types;

pub use types::*;

pub enum TcpConnectionStatus {
    Established,
    SynSent,
    SynRecv,
    FinWait1,
    FinWait2,
    TimeWait,
    Close,
    CloseWait,
    LastAck,
    Listen,
    Closing,
    None,
    /// Windows only
    DeleteTcb,
    /// Solaris only
    Idle,
    /// Solaris only
    Bound,
}
