use super::*;
#[test]
fn grants_are_bound_and_child_outcomes_must_be_consumed_before_reuse() {
    let mut model = Supervisor::new(1, 0, 1, 2).unwrap();
    let launch = model.launch(0).unwrap();
    let port = model.port(1).unwrap();
    assert_eq!(model.request(1, launch, 0), Err(abi::DENIED));
    assert_eq!(model.request(0, 0, 0), Err(abi::BAD_HANDLE));
    model.request(0, launch, 7).unwrap();
    assert_eq!(model.pending(), Some(7));
    assert_eq!(model.request(0, launch, 0), Err(abi::BUSY));
    let first = model.started(2);
    assert_eq!(model.wait(0, first), Ok(None));
    assert_eq!(model.wait(1, first), Err(abi::DENIED));
    assert_eq!(model.connect(0, port), Err(abi::DENIED));
    model.connect(1, port).unwrap();
    assert_eq!(model.connect(1, port), Err(abi::BUSY));
    model.reaped([42; abi::EXIT_BYTES]);
    assert_eq!(model.wait(0, first), Ok(Some([42; abi::EXIT_BYTES])));
    assert_eq!(model.request(0, launch, 0), Err(abi::BUSY));
    model.consume();
    model.request(0, launch, 8).unwrap();
    let second = model.started(3);
    assert_ne!(first, second);
    assert_eq!(model.wait(0, first), Err(abi::BAD_HANDLE));
    model.connect(1, port).unwrap();
}
#[test]
fn failed_admission_can_retry_and_parent_death_cancels_live_child() {
    let mut model = Supervisor::new(1, 0, 1, 2).unwrap();
    let grant = model.launch(0).unwrap();
    model.request(0, grant, 0).unwrap();
    model.failed();
    model.request(0, grant, 1).unwrap();
    model.started(2);
    model.close(0);
    assert!(model.cancel_child());
    assert_eq!(model.launch(0), Err(abi::DENIED));
    model.reaped([0; abi::EXIT_BYTES]);
    assert!(!model.cancel_child());
}
#[test]
fn dead_peer_and_invalid_boot_grants_fail_closed() {
    assert!(Supervisor::new(1, 0, 0, 2).is_err());
    assert!(Supervisor::new(1, 2, 1, 2).is_err());
    assert!(Supervisor::new(0, 0, 1, 2).is_err());
    let mut model = Supervisor::new(1, 0, 1, 2).unwrap();
    let grant = model.launch(0).unwrap();
    model.close(1);
    assert_eq!(model.request(0, grant, 0), Err(abi::PEER_CLOSED));
}
