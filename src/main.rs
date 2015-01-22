//! Example psutil executable

extern crate psutil;

fn main() {
    let processes = psutil::Process::all();

    println!("Hello world");
    println!("Read {n} pids", n=processes.len());

    for process in processes.iter() {
        println!("{} $ {}", process.pid, process.cmdline_str().unwrap());
    }
}
