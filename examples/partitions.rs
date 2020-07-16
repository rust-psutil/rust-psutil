use psutil::disk;

#[cfg(target_os = "windows")]
fn windows_main() {
	use psutil::disk::os::windows::PartitionExt;

	let partitions = disk::partitions_physical().unwrap();
	println!(
		"{:>54} {:>30} {:>8} {:>22} {:>50}",
		"Device", "Name", "Root", "File System", "Flags"
	);
	for p in partitions {
		println!(
			"{:>54} {:>30} {:>8} {:>22} {:>50}",
			p.device(),
			p.name().unwrap_or("<none>"),
			p.mountpoint().to_str().unwrap(),
			p.filesystem().as_str(),
			p.mount_options()
		);
	}
}

fn main() {
	#[cfg(target_os = "windows")]
	windows_main();
	#[cfg(not(target_os = "windows"))]
	unimplemented!("currently only on windows")
}
