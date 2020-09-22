#[cfg(target_os = "linux")]
use std::fs;
use std::io;
#[cfg(target_os = "linux")]
use std::path::Path;
use std::path::PathBuf;

#[cfg(target_os = "windows")]
use std::string::FromUtf16Error;
#[cfg(target_os = "windows")]
use winapi::{shared::ntdef::NTSTATUS, um::errhandlingapi::GetLastError};

#[cfg(target_os = "linux")]
#[cfg(feature = "sensors")]
use glob::glob as other_glob;
#[cfg(target_os = "linux")]
use snafu::ResultExt;
use snafu::Snafu;

#[cfg(target_os = "windows")]
#[derive(Debug, Snafu)]
#[snafu(visibility(pub(crate)))]
pub enum WindowsOsError {
	#[snafu(display("Windows call to \"{}\" failed with status code {:#x}", call, code))]
	Win32Error { call: &'static str, code: u32 },

	#[snafu(display("{} failed with status code {:#x}", call, status))]
	NtError {
		call: &'static str,
		status: NTSTATUS,
	},
}

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
	#[cfg(target_family = "unix")]
	NixError { source: nix::Error },

	/// macOS only.
	#[snafu(display("OS error: {}", source))]
	OsError { source: io::Error },

	/// Windows only
	#[cfg(target_os = "windows")]
	#[snafu(display("Windows error: {}", source))]
	WindowsError { source: WindowsOsError },

	/// Windows only
	#[cfg(target_os = "windows")]
	#[snafu(display("Failed to convert from UTF-16 : {}", source))]
	FromUtf16ConvertError { source: FromUtf16Error },

	/// Windows only
	#[cfg(target_os = "windows")]
	#[snafu(display("{}", message))]
	OtherError { message: String },
}

#[cfg(target_family = "unix")]
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

#[cfg(target_os = "windows")]
impl From<WindowsOsError> for Error {
	fn from(error: WindowsOsError) -> Self {
		Error::WindowsError { source: error }
	}
}

#[cfg(target_os = "windows")]
impl From<FromUtf16Error> for Error {
	fn from(error: FromUtf16Error) -> Self {
		Error::FromUtf16ConvertError { source: error }
	}
}

#[cfg(target_os = "windows")]
impl WindowsOsError {
	pub(crate) fn last_win32_error(call: &'static str) -> WindowsOsError {
		Self::Win32Error {
			call,
			code: unsafe { GetLastError() },
		}
	}
	pub(crate) fn from_code(code: u32, call: &'static str) -> WindowsOsError {
		Self::Win32Error { call, code }
	}
	pub(crate) fn last_win32_error_code() -> u32 {
		unsafe { GetLastError() as u32 }
	}
	pub(crate) fn nt_error(call: &'static str, status: NTSTATUS) -> WindowsOsError {
		Self::NtError { call, status }
	}
}

#[cfg(target_os = "linux")]
pub(crate) fn read_file<P>(path: P) -> Result<String>
where
	P: AsRef<Path>,
{
	fs::read_to_string(&path).context(ReadFile {
		path: path.as_ref(),
	})
}

#[cfg(target_os = "linux")]
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

#[cfg(target_os = "linux")]
pub(crate) fn read_link<P>(path: P) -> Result<PathBuf>
where
	P: AsRef<Path>,
{
	fs::read_link(&path).context(ReadFile {
		path: path.as_ref(),
	})
}

#[cfg(feature = "sensors")]
#[cfg(target_os = "linux")]
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
