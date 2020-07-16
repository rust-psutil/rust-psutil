cfg_if::cfg_if! {
	if #[cfg(target_os = "linux")] {
		mod linux;
		pub use linux::*;
	} else if #[cfg(target_os = "macos")] {
		mod macos;
		#[allow(unused_imports)]
		pub use macos::*;
	} else if #[cfg(target_os = "windows")] {
		mod windows;
		pub use windows::*;
	}
}

cfg_if::cfg_if! {
	if #[cfg(target_family = "unix")] {
		mod unix;
		pub use unix::*;
	}
}
