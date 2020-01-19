#[cfg(target_os = "linux")]
mod linux;

#[cfg(target_os = "linux")]
pub use linux::*;

mod cpu_percent;
mod cpu_times_percent;

pub use cpu_percent::*;
pub use cpu_times_percent::*;
