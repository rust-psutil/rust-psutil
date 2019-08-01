// https://github.com/heim-rs/heim/blob/master/heim-host/src/platform.rs
// Not found in python psutil.

use std::io;

use platforms::target::{Arch, OS};

#[derive(Clone, Debug)]
pub struct Info {}

impl Info {
    pub fn operating_system(&self) -> OS {
        todo!()
    }

    pub fn release(&self) -> &str {
        todo!()
    }

    pub fn version(&self) -> &str {
        todo!()
    }

    pub fn hostname(&self) -> &str {
        todo!()
    }

    pub fn architecture(&self) -> Arch {
        todo!()
    }
}

pub fn info() -> io::Result<Info> {
    todo!()
}
