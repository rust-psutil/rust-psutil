use std::fs;
use std::io;
use std::path::{Path, PathBuf};

#[cfg(feature = "sensors")]
use glob::glob as other_glob;
use nix;
use snafu::{ResultExt, Snafu};

#[derive(Debug, Snafu)]
#[snafu(visibility(pub(crate)))]
pub enum ParseStatusError {
	/// Linux only.
	#[snafu(display("Length is not 1. Contents: '{}'", contents))]
	IncorrectLength { contents: String },

	/// Linux and macOS.
	#[snafu(display("Incorrect char. Contents: '{}'", contents))]
	IncorrectChar { contents: String },
}

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, Snafu)]
#[snafu(visibility(pub(crate)))]
pub enum Error {
	/// Linux only.
	#[snafu(display("Failed to read file '{}': {}", path.display(), source))]
	ReadFile { path: PathBuf, source: io::Error },

	/// Linux only.
	#[snafu(display("File '{}' is missing data. Contents: '{}'", path.display(), contents))]
	MissingData { path: PathBuf, contents: String },

	/// Linux only.
	#[snafu(display("Parse error for file '{}'. Contents: '{}'. {}", path.display(), contents, source))]
	ParseInt {
		path: PathBuf,
		contents: String,
		source: std::num::ParseIntError,
	},

	/// Linux only.
	#[snafu(display("Parse error for file '{}'. Contents: '{}'. {}", path.display(), contents, source))]
	ParseFloat {
		path: PathBuf,
		contents: String,
		source: std::num::ParseFloatError,
	},

	/// Linux and macOS.
	#[snafu(display("Failed to parse status. {}", source))]
	ParseStatus { source: ParseStatusError },

	// Unix only.
	#[snafu(display("nix error: {}", source))]
	NixError { source: nix::Error },

	/// macOS only.
	#[snafu(display("OS error: {}", source))]
	OsError { source: io::Error },
}

impl From<nix::Error> for Error {
	fn from(error: nix::Error) -> Self {
		Error::NixError { source: error }
	}
}

impl From<io::Error> for Error {
	fn from(error: io::Error) -> Self {
		Error::OsError { source: error }
	}
}

impl From<ParseStatusError> for Error {
	fn from(error: ParseStatusError) -> Self {
		Error::ParseStatus { source: error }
	}
}

pub(crate) fn read_file<P>(path: P) -> Result<String>
where
	P: AsRef<Path>,
{
	fs::read_to_string(&path).context(ReadFile {
		path: path.as_ref(),
	})
}

pub(crate) fn read_dir<P>(path: P) -> Result<Vec<fs::DirEntry>>
where
	P: AsRef<Path>,
{
	fs::read_dir(&path)
		.context(ReadFile {
			path: path.as_ref(),
		})?
		.map(|entry| {
			entry.context(ReadFile {
				path: path.as_ref(),
			})
		})
		.collect()
}

pub(crate) fn read_link<P>(path: P) -> Result<PathBuf>
where
	P: AsRef<Path>,
{
	fs::read_link(&path).context(ReadFile {
		path: path.as_ref(),
	})
}

#[cfg(feature = "sensors")]
pub(crate) fn glob(path: &str) -> Vec<Result<PathBuf>> {
	other_glob(path)
		.unwrap() // only errors on invalid pattern
		.map(|result| {
			result
				.map_err(|e| e.into_error())
				.context(ReadFile { path })
		})
		.collect()
}
