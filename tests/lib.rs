extern crate psutil;

#[test] fn getpid() { assert!(psutil::getpid() != 0) }
#[test] fn getppid() { assert!(psutil::getppid() != 0) }
