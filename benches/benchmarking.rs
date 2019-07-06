#[macro_use]
extern crate bencher;

use bencher::Bencher;

fn uptime_bench(b: &mut Bencher) {
    b.iter(|| psutil::system::uptime());
}

fn mem_bench(b: &mut Bencher) {
    b.iter(|| psutil::system::virtual_memory().unwrap());
}

fn swap_bench(b: &mut Bencher) {
    b.iter(|| psutil::system::swap_memory().unwrap());
}

fn loadavg_bench(b: &mut Bencher) {
    b.iter(|| psutil::system::loadavg().unwrap());
}

fn cpucount_logical_bench(b: &mut Bencher) {
    b.iter(|| psutil::cpu::cpu_count(true).unwrap());
}

fn cpucount_physical_bench(b: &mut Bencher) {
    b.iter(|| psutil::cpu::cpu_count(false).unwrap());
}

benchmark_group!(
    benches,
    uptime_bench,
    mem_bench,
    swap_bench,
    loadavg_bench,
    cpucount_logical_bench,
    cpucount_physical_bench,
);
benchmark_main!(benches);
