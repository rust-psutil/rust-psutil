use std::io;

use crate::Mhz;

pub struct CpuFreq {}

impl CpuFreq {
    pub fn current(&self) -> Mhz {
        todo!()
    }

    pub fn min(&self) -> Mhz {
        todo!()
    }

    pub fn max(&self) -> Mhz {
        todo!()
    }
}

pub fn cpu_freq() -> io::Result<CpuFreq> {
    todo!()
}

pub fn cpu_freq_percpu() -> io::Result<Vec<CpuFreq>> {
    todo!()
}
