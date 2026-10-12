//! Executable authority chooses code; funded unbound accounts supply process capacity.
use super::*;
use cathedral_user_runtime::{
    memory::raw,
    resources::{self, Limits},
    task::{self, Executable, Launch, Outcome},
};
fn limits(pages: u64, channels: u64) -> Limits {
    Limits {
        private_pages: pages,
        reader_pages: pages,
        channels,
    }
}
pub fn run() -> u64 {
    let baseline = resources::budget().unwrap();
    assert_eq!(task::available_slots().unwrap(), 5);
    let image = Launch::at(0).unwrap().executable().unwrap();
    let other_image = Launch::at(1).unwrap().executable().unwrap();
    super::probes::admission(image);
    assert_eq!(
        Executable::from_raw(0)
            .reserve(limits(0, 0), 0, 256)
            .unwrap_err(),
        Error(abi::BAD_HANDLE as i64)
    );
    for _ in 0..4 {
        let nested = image.reserve(limits(8, 4), 2, 256).unwrap();
        assert_eq!(
            nested.allow_executable(Executable::from_raw(0)),
            Err(Error(abi::BAD_HANDLE as i64))
        );
        nested.allow_executable(other_image).unwrap();
        assert_eq!(
            nested.assign_cpu(abi::CPU_REFILL_PERIOD_US + 1),
            Err(Error(abi::INVALID_ARGUMENT as i64))
        );
        let sibling = other_image.reserve(limits(4, 2), 0, 256).unwrap();
        let spare = image.reserve(limits(0, 0), 0, 256).unwrap();
        assert_eq!(task::available_slots().unwrap(), 0);
        assert_eq!(
            image.reserve(limits(0, 0), 0, 256).unwrap_err(),
            Error(abi::NO_MEMORY as i64)
        );
        let outside = sibling.spawn(33).unwrap();
        let outside_pair = sibling.port().unwrap().connect().unwrap();
        assert_eq!(&receive(outside_pair.1).0[..5], b"ready");
        let child = nested.spawn(30).unwrap();
        assert_eq!(
            nested.allow_executable(other_image),
            Err(Error(abi::BUSY as i64))
        );
        assert_eq!(nested.assign_cpu(20000), Err(Error(abi::BUSY as i64)));
        let pair = nested.port().unwrap().connect().unwrap();
        assert_eq!(&receive(pair.1).0[..5], b"ready");
        assert_eq!(nested.release(), Err(Error(abi::BUSY as i64)));
        child.cancel().unwrap();
        assert_eq!(child.wait().unwrap(), Outcome::Cancelled);
        outside_pair.0.send(b"alive").unwrap();
        assert_eq!(&receive(outside_pair.1).0[..5], b"alive");
        outside.cancel().unwrap();
        outside.wait().unwrap();
        // Creator death returns descendant reservations as fresh owned launch
        // grants. Release them explicitly after their retained charges are gone.
        while let Ok(launch) = Launch::at(2) {
            launch.release().unwrap();
        }
        for stale in [nested, sibling, spare] {
            assert_eq!(stale.spawn(0).unwrap_err(), Error(abi::BAD_HANDLE as i64));
        }
        assert_eq!(task::available_slots().unwrap(), 5);
        assert_eq!(resources::budget().unwrap(), baseline);
    }
    let pinned = image.reserve(limits(0, 1), 0, 256).unwrap();
    let child = pinned.spawn(34).unwrap();
    let (control, reply) = pinned.port().unwrap().connect().unwrap();
    let retained = receive(reply).1.unwrap();
    control.send(b"accepted").unwrap();
    assert_eq!(child.wait().unwrap(), Outcome::Returned(34));
    assert_eq!(pinned.release(), Err(Error(abi::BUSY as i64)));
    assert_eq!(task::available_slots().unwrap(), 4);
    retained.close().unwrap();
    pinned.release().unwrap();
    assert_eq!(task::available_slots().unwrap(), 5);
    assert_eq!(resources::budget().unwrap(), baseline);
    cathedral_user_runtime::write(b"Cathedral admission: executable capabilities, recursive funded pools, load rollback, stale grants and sibling survival passed\n").unwrap();
    let (_, idle) = ipc::channel().unwrap();
    idle.receive(&mut [0; 64]).unwrap();
    0
}
pub fn worker(role: u64, mode: u64) -> u64 {
    assert_eq!(role, if mode == 33 { 9 } else { 1 });
    assert!(cathedral_user_runtime::display::mapping().is_err());
    assert!(cathedral_user_runtime::keyboard::read(false).is_err());
    let image = Executable::at(0).unwrap();
    if mode == 34 {
        let (_, receive) = ipc::channel().unwrap();
        Handle::bootstrap(1)
            .unwrap()
            .send_with(b"retained", receive)
            .unwrap();
        Handle::bootstrap(0).unwrap().receive(&mut [0; 64]).unwrap();
        return 34;
    }
    if mode == 30 {
        assert!(Executable::at(1).is_ok());
        assert!(Executable::at(2).is_err());
        let slots = task::available_slots().unwrap();
        let budget = resources::budget().unwrap();
        let rejected = image.reserve(limits(1, 1), 0, 1).unwrap();
        assert_eq!(
            rejected.spawn(32).unwrap_err(),
            Error(abi::NO_MEMORY as i64)
        );
        rejected.release().unwrap();
        assert_eq!(task::available_slots().unwrap(), slots);
        assert_eq!(resources::budget().unwrap(), budget);
    }
    if mode == 31 || mode == 32 {
        assert!(Executable::at(1).is_err());
    }
    if mode < 32 {
        let child = image
            .reserve(
                limits(
                    if mode == 30 { 4 } else { 2 },
                    if mode == 30 { 2 } else { 1 },
                ),
                if mode == 30 { 1 } else { 0 },
                256,
            )
            .unwrap();
        child.spawn(mode + 1).unwrap();
        assert_eq!(
            &receive(child.port().unwrap().connect().unwrap().1).0[..5],
            b"ready"
        );
    }
    let budget = resources::budget().unwrap();
    raw::allocate(budget.limits.private_pages as usize).unwrap();
    assert_eq!(raw::allocate(1), Err(Error(abi::NO_MEMORY as i64)));
    Handle::bootstrap(1).unwrap().send(b"ready").unwrap();
    loop {
        let mut bytes = [0; 64];
        let len = Handle::bootstrap(0).unwrap().receive(&mut bytes).unwrap();
        Handle::bootstrap(1).unwrap().send(&bytes[..len]).unwrap();
    }
}
