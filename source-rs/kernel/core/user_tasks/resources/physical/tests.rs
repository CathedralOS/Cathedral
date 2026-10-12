use super::*;

#[test]
fn nested_reservations_conserve_capacity_and_keep_siblings_funded() {
    let mut funding = Funding::new([100, 0, 0, 0, 0, 0, 0, 0]);
    funding.charge(0, 10).unwrap();
    funding.assign(0, 1, 60).unwrap();
    funding.assign(0, 2, 20).unwrap();
    funding.charge(1, 10).unwrap();
    funding.assign(1, 3, 50).unwrap();
    funding.charge(3, 50).unwrap();
    assert_eq!(funding.charge(1, 1), Err(abi::NO_MEMORY));
    assert_eq!(funding.charge(3, 1), Err(abi::NO_MEMORY));
    assert_eq!(funding.assign(1, 4, 1), Err(abi::BUSY));
    funding.charge(2, 20).unwrap();
    assert_eq!(funding.capacity(), 100);
    assert_eq!(funding.charged(), 90);
    funding.refund(3, 50);
    funding.assign(1, 3, 0).unwrap();
    funding.refund(1, 10);
    funding.assign(0, 1, 0).unwrap();
    assert_eq!(funding.snapshot(0), [80, 10]);
    assert_eq!(funding.snapshot(2), [20, 20]);
}

#[test]
fn retained_sponsor_charge_prevents_releasing_or_laundering_funding() {
    let mut funding = Funding::new([100, 0, 0, 0, 0, 0, 0, 0]);
    funding.assign(0, 1, 40).unwrap();
    funding.charge(1, 24).unwrap(); // image/tables/stack + retained object
    funding.refund(1, 20); // producer exits; reader remains
    assert_eq!(funding.assign(0, 1, 0), Err(abi::BUSY));
    assert_eq!(funding.snapshot(1), [40, 4]);
    funding.charge(1, 36).unwrap(); // restart never resets the old charge
    assert_eq!(funding.charge(1, 1), Err(abi::NO_MEMORY));
    funding.refund(1, 36);
    funding.refund(1, 4); // final reader release
    funding.assign(0, 1, 0).unwrap();
    assert_eq!(funding.snapshot(0), [100, 0]);
}

#[test]
fn invalid_and_overflow_requests_are_atomic() {
    let mut funding = Funding::new([100, 0, 0, 0, 0, 0, 0, 0]);
    for (owner, child, pages, error) in [
        (0, 0, 0, abi::INVALID_ARGUMENT),
        (0, abi::MAX_TASKS, 0, abi::INVALID_ARGUMENT),
        (0, 1, usize::MAX, abi::NO_MEMORY),
    ] {
        assert_eq!(funding.assign(owner, child, pages), Err(error));
        assert_eq!(funding.snapshot(0), [100, 0]);
        assert_eq!(funding.snapshot(1), [0, 0]);
    }
    assert_eq!(funding.charge(0, usize::MAX), Err(abi::NO_MEMORY));
    assert_eq!(funding.charged(), 0);
}
