//! Example psutil executable

extern crate psutil;

fn main() {
    println!("Hello world");
    println!("Read {n} pids", n=psutil::pids().len());

    for pid in psutil::pids().iter() {
        println!("{} $ {}", pid, psutil::cmdline(*pid).unwrap());
    }
}
