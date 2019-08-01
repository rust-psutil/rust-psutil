// https://github.com/heim-rs/heim/blob/master/heim-common/src/sys/unix.rs

use std::io;

use libc;
use once_cell::sync::Lazy;

use crate::{Bytes, FloatCount};

fn ticks_per_second() -> io::Result<FloatCount> {
    let result = unsafe { libc::sysconf(libc::_SC_CLK_TCK) };

    if result > 0 {
        Ok(result as FloatCount)
    } else {
        Err(io::Error::last_os_error())
    }
}

fn page_size() -> io::Result<Bytes> {
    let result = unsafe { libc::sysconf(libc::_SC_PAGESIZE) };

    if result > 0 {
        Ok(result as Bytes)
    } else {
        Err(io::Error::last_os_error())
    }
}

pub(crate) static TICKS_PER_SECOND: Lazy<FloatCount> =
    Lazy::new(|| ticks_per_second().expect("Unable to determine CPU number of ticks per second"));

pub(crate) static PAGE_SIZE: Lazy<Bytes> =
    Lazy::new(|| page_size().expect("Unable to determine page size"));
