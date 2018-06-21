//! Utility methods, mostly for dealing with IO.

use std::fs::File;
use std::io::{Read, Result};
use std::path::Path;

pub fn read_file(path: &Path) -> Result<String> {
    let mut buffer = String::new();
    let mut file = try!(File::open(path));
    try!(file.read_to_string(&mut buffer));
    Ok(buffer)
}

macro_rules! try_parse {
    ($field:expr) => {
        try_parse!($field, FromStr::from_str)
    };
    ($field:expr, $from_str:path) => {
        try!(match $from_str($field) {
            Ok(result) => Ok(result),
            Err(_) => Err(Error::new(ErrorKind::InvalidInput,
                format!("Could not parse {:?}", $field)))
        })
    };
}
