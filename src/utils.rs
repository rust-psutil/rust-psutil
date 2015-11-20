//! Utility methods, mostly for dealing with IO.

use std::fs::File;
use std::io::{Read, Result};
use std::path::Path;

pub fn read_file(path: &Path) -> Result<String> {
    let mut buffer = String::new();
    let mut file = try!(File::open(path));
    try!(file.read_to_string(&mut buffer));
    return Ok(buffer);
}
