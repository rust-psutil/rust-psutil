use psutil::cpu;

fn main() {
    let logical_cores = cpu::cpu_count();
    let physical_cores = cpu::cpu_count_physical();

    println!(
        "This system has {} actual CPUs and {} hyperthreads",
        physical_cores, logical_cores
    );
}
