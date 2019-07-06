fn main() {
    let logical_cores = psutil::cpu::cpu_count(true).unwrap();
    let physical_cores = psutil::cpu::cpu_count(false).unwrap();

    println!("This system has {} actual CPUs and {} hyperthreads", physical_cores, logical_cores);
}
