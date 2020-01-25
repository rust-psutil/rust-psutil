use std::path::PathBuf;

use crate::Fd;

pub struct OpenFile {
	pub path: PathBuf,
	pub fd: Option<Fd>,
}
