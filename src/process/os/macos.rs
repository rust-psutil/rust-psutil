use std::collections::HashMap;
use std::io;

use crate::process::Process;

pub trait ProcessExt {
    fn environ(&self) -> io::Result<HashMap<String, String>>;
}

impl ProcessExt for Process {
    fn environ(&self) -> io::Result<HashMap<String, String>> {
        todo!()
    }
}
