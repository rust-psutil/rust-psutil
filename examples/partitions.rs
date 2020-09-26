use psutil::disk;
#[cfg(target_os = "windows")]
use psutil::disk::os::windows::PartitionExt;

fn main() {
	let partitions = disk::partitions_physical().unwrap();
	#[cfg(target_os = "windows")]
	println!(
		"{:>54} {:>30} {:>8} {:>22} {:>50}",
		"Device", "Name", "Root", "File System", "Flags"
	);
	#[cfg(not(target_os = "windows"))]
	println!(
		"{:>54} {:>8} {:>22} {:>50}",
		"Device", "Root", "File System", "Flags"
	);
	for p in partitions {
		#[cfg(target_os = "windows")]
		println!(
			"{:>54} {:>30} {:>8} {:>22} {:>50}",
			p.device(),
			p.name().unwrap_or("<none>"),
			p.mountpoint().to_str().unwrap(),
			p.filesystem().as_str(),
			p.mount_options()
		);
		#[cfg(not(target_os = "windows"))]
		println!(
			"{:>54} {:>8} {:>22} {:>50}",
			p.device(),
			p.mountpoint().to_str().unwrap(),
			p.filesystem().as_str(),
			p.mount_options()
		)
	}
}
