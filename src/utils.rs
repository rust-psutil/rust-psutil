//! Utility methods, mostly for dealing with IO.

use std::fs::File;
use std::io::{Read, Result};
use std::path::Path;
use std::env;

pub fn read_file(path: &Path) -> Result<String> {
    let mut buffer = String::new();
    let mut file = try!(File::open(path));
    try!(file.read_to_string(&mut buffer));
    Ok(buffer)
}


pub fn read_proc_file(subpath: &str) -> Result<String> {
    let default_procdir = "/proc".to_string();
    let procdir = env::var("PSUTIL_PROCDIR").unwrap_or(default_procdir);
    let path = Path::new(&procdir).join(subpath);
    read_file(&path)
}
