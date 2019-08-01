//! Utility methods, mostly for dealing with IO.

use std::io;

macro_rules! try_parse {
    ($field:expr) => {
        try_parse!($field, std::str::FromStr::from_str)
    };
    ($field:expr, $from_str:path) => {
        $from_str($field).map_err(|_| {
            std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                format!("Could not parse {:?}", $field),
            )
        })?
    };
}

pub fn not_found(key: &str) -> io::Error {
    io::Error::new(io::ErrorKind::NotFound, format!("{} not found", key))
}

pub fn invalid_data(message: &str) -> io::Error {
    io::Error::new(io::ErrorKind::InvalidData, message)
}
