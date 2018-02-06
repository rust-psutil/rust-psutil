extern crate psutil;

#[test]
fn uptime() {
    assert!(psutil::system::uptime() > 0);
}
