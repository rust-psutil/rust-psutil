//! Example psutil executable

extern crate psutil;

fn main() {
    let processes = psutil::Process::all();

    println!("Hello world");
    println!("Read {n} pids", n=processes.len());

    for process in processes.iter() {
        let cmdline = process.cmdline().unwrap();
        if cmdline.len() > 0 {
            println!("{} $ {}", process.pid, cmdline[0]);
        }
    }
}
