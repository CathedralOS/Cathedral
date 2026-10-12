//! Exhaust a nested frame envelope while the controller exercises an outside sibling.
use super::*;

pub(super) fn exhaust(image: Executable) {
    let before = resources::physical_budget().unwrap();
    let quotas = resources::budget().unwrap();
    let slots = task::available_slots().unwrap();
    let input = Handle::bootstrap(0).unwrap();
    let output = Handle::bootstrap(1).unwrap();
    let available = before.reserved - before.charged;

    // Enough process slots and runtime quotas remain; only physical funding runs out.
    let hog = image
        .reserve_funded(limits(0, 0), 0, 256, available)
        .unwrap();
    let child = hog.spawn(35).unwrap();
    let pair = hog.port().unwrap().connect().unwrap();
    assert_eq!(&receive(pair.1).0[..5], b"ready");
    assert_eq!(raw::allocate(1), Err(Error(abi::NO_MEMORY as i64)));
    assert_eq!(
        image.reserve_funded(limits(1, 1), 0, 256, 1).unwrap_err(),
        Error(abi::BUSY as i64)
    );
    assert_eq!(resources::budget().unwrap(), quotas);
    assert_eq!(task::available_slots().unwrap(), slots - 1);
    assert_eq!(hog.assign_memory(0), Err(Error(abi::BUSY as i64)));
    output.send(b"exhausted").unwrap();
    assert_eq!(&receive(input).0[..7], b"recover");

    child.cancel().unwrap();
    child.wait().unwrap();
    // Teardown refunds use, but reservations stay owned until explicitly returned.
    assert_eq!(raw::allocate(1), Err(Error(abi::NO_MEMORY as i64)));
    hog.release().unwrap();
    assert_eq!(resources::physical_budget().unwrap(), before);

    // Failure comes from real reserved funding, independently of the 256-frame load ceiling.
    let tiny = image.reserve_funded(limits(1, 1), 0, 256, 1).unwrap();
    for _ in 0..2 {
        assert_eq!(tiny.spawn(35).unwrap_err(), Error(abi::NO_MEMORY as i64));
        assert_eq!(
            resources::physical_budget().unwrap().charged,
            before.charged
        );
    }
    tiny.assign_memory(256).unwrap();
    let recovered = tiny.spawn(35).unwrap();
    assert_eq!(
        &receive(tiny.port().unwrap().connect().unwrap().1).0[..5],
        b"ready"
    );
    recovered.cancel().unwrap();
    recovered.wait().unwrap();
    tiny.release().unwrap();
    assert_eq!(task::available_slots().unwrap(), slots);
    assert_eq!(resources::physical_budget().unwrap(), before);
    assert_eq!(resources::budget().unwrap(), quotas);
    let object = raw::allocate(1).unwrap();
    // SAFETY: No mapped Rust references, aliases or offers were created.
    unsafe { raw::release(object) }.unwrap();
    assert_eq!(resources::physical_budget().unwrap(), before);
    cathedral_user_runtime::write(b"Cathedral physical memory: nested exhaustion, sibling allocation, funded load rollback and recovery passed\n").unwrap();
}
