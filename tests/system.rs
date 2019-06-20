#[test]
fn uptime() {
    assert!(psutil::host::uptime() > 0);
}

#[test]
fn virtual_memory() {
    let vm = psutil::memory::virtual_memory().unwrap();
    // simple sanity checking
    assert!(vm.total != 0);
    assert!(vm.free <= vm.total);
    assert!(vm.percent > 0.0);
}

#[test]
fn swap_memory() {
    let sm = psutil::memory::swap_memory().unwrap();
    // simple sanity checking
    if sm.total != 0 {
        assert!(sm.free <= sm.total);
        assert!(sm.percent >= 0.0);
    }
}

#[test]
fn loadaverage() {
    let load = psutil::host::loadavg().unwrap();
    // shouldn't be negative
    assert!(load.one >= 0.0);
    assert!(load.five >= 0.0);
    assert!(load.fifteen >= 0.0);
}
