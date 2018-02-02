extern crate psutil;

use std::path::PathBuf;
use std::env;

#[test]
fn uptime() {
    let mut d = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    d.push("resources/test/fakeproc");
    env::set_var("PSUTIL_PROCDIR", d.to_str().unwrap());
    assert!(psutil::system::uptime() == 16);
}
