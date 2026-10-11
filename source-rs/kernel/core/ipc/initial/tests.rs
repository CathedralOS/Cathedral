use super::*;
fn dormant() -> Ipc {
    let mut ipc = Ipc::new(1, 3, &[]).unwrap();
    ipc.close_task(1);
    ipc.close_task(2);
    ipc
}
#[test]
fn ordered_bundle_is_installed_before_first_child_operation() {
    let mut ipc = dormant();
    let [send, input] = ipc.create(0).unwrap();
    let [output, recv] = ipc.create(0).unwrap();
    ipc.prepare_child_with(0, 0, 1, 2, 0, &[input, output])
        .unwrap();
    let names = ipc.initial(1).unwrap().to_vec();
    assert_eq!(names.len(), 2);
    assert!(ipc.initial(0).unwrap().is_empty());
    assert_eq!(ipc.close(0, input), Err(abi::BAD_HANDLE));
    assert_eq!(ipc.close(0, output), Err(abi::BAD_HANDLE));
    assert_eq!(ipc.send(1, names[0], b"wrong right"), Err(abi::DENIED));
    ipc.send(0, send, b"work").unwrap();
    assert_eq!(ipc.receive(1, names[0], 64).unwrap().unwrap().len, 4);
    ipc.send(1, names[1], b"answer").unwrap();
    ipc.close_task(1);
    assert_eq!(ipc.receive(0, recv, 64).unwrap().unwrap().len, 6);
    ipc.prepare_child_with(0, 0, 1, 3, 0, &[]).unwrap();
    assert!(ipc.initial(1).unwrap().is_empty());
    assert_eq!(ipc.send(1, names[1], b"stale"), Err(abi::BAD_HANDLE));
    ipc.close_task(0);
    ipc.close_task(1);
    assert!(ipc.endpoints[MAX_ENDPOINTS..].iter().all(Option::is_none));
}
#[test]
fn every_validation_failure_preserves_all_sources_and_dormant_child() {
    let mut ipc = dormant();
    let [s, r] = ipc.create(0).unwrap();
    for (handles, error) in [
        (alloc::vec![s, 0], abi::BAD_HANDLE),
        (alloc::vec![s, s], abi::INVALID_ARGUMENT),
        (alloc::vec![s;5], abi::INVALID_ARGUMENT),
    ] {
        assert_eq!(ipc.prepare_child_with(0, 0, 1, 2, 0, &handles), Err(error));
        assert!(!ipc.live(1));
        assert_eq!(ipc.initial(1), Err(abi::BAD_HANDLE));
        ipc.send(0, s, b"retained").unwrap();
        assert_eq!(ipc.receive(0, r, 64).unwrap().unwrap().len, 8);
    }
    ipc.prepare_child(0, 0, 2, 2);
    ipc.accept_child(0, 0);
    let boot = ipc.handle(0, 0).unwrap();
    assert_eq!(
        ipc.prepare_child_with(2, 0, 1, 3, 0, &[s, boot]),
        Err(abi::DENIED)
    );
    assert_eq!(
        ipc.prepare_child_with(2, 0, 1, 3, 2, &[s]),
        Err(abi::BAD_HANDLE)
    );
    ipc.check(0, s, Right::Send).unwrap();
}
#[test]
fn queued_receivers_and_late_queue_changes_are_rechecked_before_commit() {
    let mut ipc = dormant();
    let [s, r] = ipc.create(0).unwrap();
    ipc.validate_initial(0, 1, &[r]).unwrap();
    ipc.send(0, s, b"arrived").unwrap();
    assert_eq!(ipc.prepare_child_with(0, 0, 1, 2, 0, &[r]), Err(abi::BUSY));
    assert_eq!(ipc.receive(0, r, 64).unwrap().unwrap().len, 7);
    ipc.prepare_child_with(0, 0, 1, 2, 0, &[r]).unwrap();
    assert_eq!(
        ipc.prepare_child_with(0, 0, 1, 3, 0, &[s]),
        Err(abi::BAD_HANDLE)
    );
}
#[test]
fn full_bundle_can_include_both_rights_and_snapshot_never_regrants() {
    let mut ipc = dormant();
    let a = ipc.create(0).unwrap();
    let b = ipc.create(0).unwrap();
    ipc.prepare_child_with(0, 0, 1, 2, 0, &[a[0], a[1], b[0], b[1]])
        .unwrap();
    assert_eq!(ipc.budget(0), Ok([4, 2, 16, 0]));
    assert_eq!(ipc.budget(1), Ok([4, 0, 16, 4]));
    ipc.close_task(0); // Both ends moved; creator death cannot erase their funding.
    assert_eq!(ipc.charged_channels(0), 2);
    let names = ipc.initial(1).unwrap().to_vec();
    assert_eq!(names.len(), 4);
    ipc.send(1, names[0], b"self").unwrap();
    assert_eq!(ipc.receive(1, names[1], 64).unwrap().unwrap().len, 4);
    ipc.close(1, names[2]).unwrap();
    assert_eq!(ipc.initial(1).unwrap(), names);
    assert_eq!(
        ipc.send(1, ipc.initial(1).unwrap()[2], b"revive"),
        Err(abi::BAD_HANDLE)
    );
    ipc.close_task(1);
    assert_eq!(ipc.charged_channels(0), 0);
    assert!(ipc.endpoints[MAX_ENDPOINTS..].iter().all(Option::is_none));
}
