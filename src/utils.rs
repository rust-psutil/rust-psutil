//! Utility methods, mostly for dealing with IO.

macro_rules! try_parse {
    ($field:expr) => {
        try_parse!($field, FromStr::from_str)
    };
    ($field:expr, $from_str:path) => {
        match $from_str($field) {
            Ok(result) => Ok(result),
            Err(_) => Err(Error::new(
                ErrorKind::InvalidInput,
                format!("Could not parse {:?}", $field),
            )),
        }?
    };
}
