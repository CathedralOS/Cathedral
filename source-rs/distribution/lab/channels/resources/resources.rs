// Real nested reservation subdivision, exhaustion, cancellation and return.
use super::*;
use cathedral_user_runtime::{
    memory::raw,
    resources::{self, Limits},
    task::{Launch, Outcome, Port},
};
fn limits(pages: u64, channels: u64) -> Limits {
    Limits {
        private_pages: pages,
        reader_pages: pages,
        channels,
    }
}
pub fn run() {
    let before = resources::budget().unwrap();
    let launch = Launch::at(1).unwrap();
    resources::assign(launch, limits(8, 4)).unwrap();
    probes::check(launch);
    let loan = Launch::at(2).unwrap();
    let child = launch.spawn_tree(20, &[], &[loan]).unwrap();
    let (_, events) = Port::at(1).unwrap().connect().unwrap();
    assert_eq!(&receive(events).0[..5], b"ready");
    assert_eq!(
        resources::assign(launch, limits(8, 4)),
        Err(Error(abi::BUSY as i64))
    );
    child.cancel().unwrap();
    assert_eq!(child.wait().unwrap(), Outcome::Cancelled);
    // Returned accounts retain their reservations; explicitly reclaim only unused capacity.
    resources::assign(Launch::at(1).unwrap(), limits(0, 4)).unwrap();
    resources::assign(Launch::at(2).unwrap(), limits(0, 4)).unwrap();
    assert_eq!(resources::budget().unwrap(), before);
    cathedral_user_runtime::write(b"Cathedral resources: nested page/channel subdivision, exhaustion and subtree reclamation passed\n").unwrap();
}
pub fn worker(mode: u64) -> u64 {
    if mode == 20 {
        resources::assign(Launch::at(0).unwrap(), limits(4, 4)).unwrap();
        let _child = Launch::at(0).unwrap().spawn(21).unwrap();
        let (_, events) = Port::at(0).unwrap().connect().unwrap();
        assert_eq!(&receive(events).0[..5], b"ready");
    }
    let budget = resources::budget().unwrap();
    assert_eq!(budget.limits, limits(4, 4));
    let _pages = raw::allocate(4).unwrap();
    assert_eq!(raw::allocate(1), Err(Error(abi::NO_MEMORY as i64)));
    for _ in 0..4 {
        ipc::channel().unwrap();
    }
    assert_eq!(ipc::channel().unwrap_err(), Error(abi::NO_MEMORY as i64));
    assert_eq!(resources::budget().unwrap().used.private_pages, 4);
    assert_eq!(resources::budget().unwrap().used.channels, 4);
    Handle::bootstrap(1).unwrap().send(b"ready").unwrap();
    Handle::bootstrap(0).unwrap().receive(&mut [0; 64]).unwrap();
    0
}
