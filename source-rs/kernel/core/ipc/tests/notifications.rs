use crate::ipc::{EndpointSpec, Ipc};
use cathedral_kernel_contracts::user as abi;

fn pair() -> Ipc {
    Ipc::new(
        1,
        3,
        &[EndpointSpec {
            sender: 0,
            receiver: 1,
            revoker: Some(2),
        }],
    )
    .unwrap()
}

#[test]
fn send_revoke_and_peer_close_notify_only_the_receiver_without_consuming_data() {
    let mut ipc = pair();
    let send = ipc.handle(0, 0).unwrap();
    let receive = ipc.handle(1, 0).unwrap();
    let revoke = ipc.handle(2, 0).unwrap();
    assert_eq!(ipc.take_changed_receivers(), 0);
    ipc.send(0, send, b"payload").unwrap();
    assert_eq!(ipc.take_changed_receivers(), 1 << 1);
    assert_eq!(ipc.send(0, send, b"full"), Err(abi::WOULD_BLOCK));
    assert_eq!(ipc.take_changed_receivers(), 0);
    ipc.close_task(0);
    assert_eq!(ipc.take_changed_receivers(), 1 << 1);
    assert_eq!(ipc.receive(1, receive, 64).unwrap().unwrap().len, 7);
    assert_eq!(ipc.receive(1, receive, 64), Err(abi::PEER_CLOSED));
    ipc.revoke(2, revoke).unwrap();
    assert_eq!(ipc.take_changed_receivers(), 1 << 1);
    assert_eq!(ipc.ready(1, receive), Err(abi::REVOKED));
}

#[test]
fn sealed_memory_and_endpoint_moves_use_the_same_receiver_notification() {
    let mut ipc = pair();
    let send = ipc.handle(0, 0).unwrap();
    let receive = ipc.handle(1, 0).unwrap();
    ipc.send_memory(0, send, b"pixels", 0);
    assert_eq!(ipc.take_changed_receivers(), 1 << 1);
    ipc.receive_memory(1, receive, 64).unwrap().unwrap();
    let [writer, reader] = ipc.create(0).unwrap();
    ipc.send_cap(0, send, b"move", Some(reader)).unwrap();
    assert_eq!(ipc.take_changed_receivers(), (1 << 0) | (1 << 1));
    let moved = ipc
        .receive_cap(1, receive, 64, true)
        .unwrap()
        .unwrap()
        .attachment
        .unwrap();
    ipc.send(0, writer, b"through moved right").unwrap();
    assert_eq!(ipc.take_changed_receivers(), 1 << 1);
    assert!(ipc.ready(1, moved).unwrap());
}

#[test]
fn discarded_escrow_notifies_its_peer_and_rebinding_invalidates_old_receiver() {
    let mut ipc = pair();
    let [writer, reader] = ipc.create(0).unwrap();
    ipc.send_cap(0, ipc.handle(0, 0).unwrap(), b"move", Some(writer))
        .unwrap();
    ipc.take_changed_receivers();
    ipc.close_task(1); // Drops the escrowed send right, waking its surviving receiver.
    assert_eq!(ipc.take_changed_receivers(), 1);
    assert_eq!(ipc.receive(0, reader, 64), Err(abi::PEER_CLOSED));

    // A live peer awaiting an old boot name must learn that the pair was replaced.
    let mut ipc = Ipc::new(1, 2, &[]).unwrap();
    ipc.close_task(1);
    ipc.prepare_child(0, 0, 1, 2);
    ipc.accept_child(0, 0);
    let old_receive = ipc.handle(0, 1).unwrap();
    ipc.close_task(1);
    ipc.take_changed_receivers();
    ipc.prepare_child(0, 0, 1, 3);
    assert_ne!(ipc.take_changed_receivers() & 1, 0);
    assert_eq!(ipc.ready(0, old_receive), Err(abi::BAD_HANDLE));
}
