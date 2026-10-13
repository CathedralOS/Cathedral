use super::*;

#[test]
fn authority_bounds_and_restart_names() {
    let mut devices = Devices::default();
    devices.install(2, 7, 8);
    assert_eq!(devices.handle(1, 0), Err(abi::BAD_HANDLE));
    assert_eq!(devices.check(1, 7), Err(abi::BAD_HANDLE));
    assert_eq!(devices.handle(2, 1), Err(abi::BAD_HANDLE));
    assert_eq!(devices.register(2, 7, 7), Ok(()));
    assert_eq!(devices.register(2, 7, 8), Err(abi::INVALID_ARGUMENT));
    assert_eq!(devices.register(2, 7, u64::MAX), Err(abi::INVALID_ARGUMENT));
    devices.arm(2, 7).unwrap();
    devices.interrupt(2);
    assert!(devices.close(2));
    assert!(!devices.interrupt(2));
    devices.install(2, 8, 8);
    assert_eq!(devices.check(2, 7), Err(abi::BAD_HANDLE));
    assert_eq!(devices.wait(2, 8, false), Err(abi::WOULD_BLOCK));
}

#[test]
fn irq_before_wait_and_wait_before_irq_are_both_lossless() {
    let mut devices = Devices::default();
    devices.install(0, 3, 8);
    devices.arm(0, 3).unwrap();
    assert_eq!(devices.arm(0, 3), Err(abi::BUSY));
    assert!(!devices.interrupt(0));
    assert!(!devices.interrupt(0)); // Coalesce; only one notification per arm.
    assert_eq!(devices.arm(0, 3), Err(abi::BUSY));
    assert_eq!(devices.wait(0, 3, false), Ok(false));
    assert_eq!(devices.wait(0, 3, false), Err(abi::WOULD_BLOCK));
    devices.arm(0, 3).unwrap();
    assert_eq!(devices.wait(0, 3, true), Ok(true));
    assert!(devices.interrupt(0));
    assert!(!devices.interrupt(0));
    assert_eq!(devices.wait(0, 3, false), Err(abi::WOULD_BLOCK));
    devices.arm(0, 3).unwrap();
    devices.wait(0, 3, true).unwrap();
    devices.close(0);
    devices.install(0, 4, 8);
    assert!(!devices.interrupt(0)); // No inherited waiter or arm.
}
