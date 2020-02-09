use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use glob::glob as other_glob;
use nix;
use snafu::{ResultExt, Snafu};

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, Snafu)]
#[snafu(visibility(pub(crate)))]
pub enum Error {
	#[snafu(display("Failed to read file '{}': {}", path.display(), source))]
	ReadFile { path: PathBuf, source: io::Error },

	#[snafu(display("File '{}' is missing data. Contents: '{}'", path.display(), contents))]
	MissingData { path: PathBuf, contents: String },

	#[snafu(display("Parse error for file '{}'. Contents: '{}'. {}", path.display(), contents, source))]
	ParseInt {
		path: PathBuf,
		contents: String,
		source: std::num::ParseIntError,
	},

	#[snafu(display("Parse error for file '{}'. Contents: '{}'. {}", path.display(), contents, source))]
	ParseFloat {
		path: PathBuf,
		contents: String,
		source: std::num::ParseFloatError,
	},

	#[snafu(display("Error while parsing status. Contents: '{}'.", contents))]
	ParseStatus { contents: String },

	#[snafu(display("nix error: {}", source))]
	NixError { source: nix::Error },
}

impl From<nix::Error> for Error {
	fn from(error: nix::Error) -> Self {
		Error::NixError { source: error }
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
