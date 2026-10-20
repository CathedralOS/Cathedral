//! Object/transport transaction tests; actual page edits and faults run in QEMU.
use super::*;
use crate::ipc::{EndpointSpec, Ipc};
fn setup() -> (Regions, Ipc) {
    let regions = Regions::new(
        3,
        // Enough pages for a reader slot per page, so slot pressure is what
        // these tests reach, except where a larger object is chosen on purpose.
        &[0, 1, 2].map(|task| Grant {
            task,
            private_pages: MAX_SHARED_REGIONS,
            shared_pages: MAX_SHARED_REGIONS,
        }),
    )
    .unwrap();
    let ipc = Ipc::new(
        1,
        3,
        &[
            EndpointSpec {
                sender: 0,
                receiver: 1,
                revoker: Some(0),
            },
            EndpointSpec {
                sender: 1,
                receiver: 2,
                revoker: None,
            },
        ],
    )
    .unwrap();
    (regions, ipc)
}
#[test]
fn forwarded_rights_survive_both_senders_and_preserve_sponsor_until_last_release() {
    let (mut model, mut ipc) = setup();
    let object = model.allocate(0, 2).unwrap();
    let owner = model.handle(0, object);
    assert_eq!(model.reserve_read(0, owner, 1), Err(abi::DENIED));
    model.freeze(0, owner).unwrap();
    let a = ipc.handle(0, 0).unwrap();
    let b = ipc.handle(1, 0).unwrap();
    let escrow = model
        .reserve_read(0, owner, ipc.memory_receiver(0, a, 1).unwrap())
        .unwrap();
    let pending = model.reads[escrow].unwrap().name;
    assert_eq!(model.access(1, pending), Err(abi::BAD_HANDLE));
    ipc.send_memory(0, a, b"x", escrow);
    model.release(0, owner).unwrap();
    ipc.close_task(0);
    assert!(model.entries[object].live());
    assert_eq!(model.charged(0), [2, 0]);
    assert_eq!(ipc.receive_cap(1, b, 64, true), Err(abi::TOO_SMALL));
    assert_eq!(ipc.receive(1, b, 64), Err(abi::TOO_SMALL));
    let m = ipc.receive_memory(1, b, 64).unwrap().unwrap();
    let reader = model.publish_read(1, m.memory.unwrap());
    assert_eq!(model.mapped(1, reader), Ok(None));
    let forward = model.reserve_read(1, reader, 2).unwrap();
    ipc.send_memory(1, ipc.handle(1, 1).unwrap(), b"y", forward);
    model.drop_read(escrow);
    ipc.close_task(1);
    let m = ipc
        .receive_memory(2, ipc.handle(2, 0).unwrap(), 64)
        .unwrap()
        .unwrap();
    let final_name = model.publish_read(2, m.memory.unwrap());
    assert_eq!(model.access(2, final_name), Ok(object));
    assert_eq!(model.charged(0), [2, 0]);
    model.drop_read(forward);
    assert!(!model.entries[object].live());
    assert_eq!(model.charged(0), [0, 0]);
    assert_eq!(model.access(2, final_name), Err(abi::BAD_HANDLE));
}
#[test]
fn queue_discard_returns_escrow_and_capacity_without_touching_sender() {
    let (mut model, mut ipc) = setup();
    // Over half the reader pages, so a second reservation is page pressure.
    let object = model.allocate(0, MAX_SHARED_REGIONS / 2 + 1).unwrap();
    let owner = model.handle(0, object);
    model.freeze(0, owner).unwrap();
    let send = ipc.handle(0, 0).unwrap();
    let slot = model.reserve_read(0, owner, 1).unwrap();
    ipc.send_memory(0, send, b"x", slot);
    assert_eq!(model.reserve_read(0, owner, 1), Err(abi::NO_MEMORY));
    assert_eq!(ipc.memory_receiver(0, send, 1), Err(abi::WOULD_BLOCK));
    ipc.revoke(0, ipc.handle(0, 1).unwrap()).unwrap();
    let retired = ipc.take_retired_memory();
    assert_eq!(retired[slot / 64], 1 << (slot % 64));
    assert_eq!(retired.iter().map(|word| word.count_ones()).sum::<u32>(), 1);
    assert!(!ipc.memory_retirement_pending());
    model.drop_read(slot);
    assert_eq!(model.charged(1), [0, 0]);
    assert_eq!(model.access(0, owner), Ok(object));
    let next = model.reserve_read(0, owner, 1).unwrap();
    assert!(!model.reads[next].unwrap().published);
}
#[test]
fn duplicate_handles_get_independent_mappings_and_bounded_metadata() {
    let (mut model, _) = setup();
    let object = model.allocate(0, 1).unwrap();
    let owner = model.handle(0, object);
    model.freeze(0, owner).unwrap();
    let mut grants = [0; MAX_SHARED_REGIONS];
    for name in &mut grants {
        let slot = model.reserve_read(0, owner, 1).unwrap();
        *name = model.publish_read(1, slot);
        let (_, mapping) = model.prepare_mapping(1, *name).unwrap();
        model.set_mapping(1, *name, Some(mapping));
    }
    assert_eq!(model.reserve_read(0, owner, 1), Err(abi::NO_MEMORY));
    assert_ne!(model.mapped(1, grants[0]), model.mapped(1, grants[1]));
    model.set_mapping(1, grants[0], None);
    assert!(model.access(1, grants[0]).is_ok());
    assert!(model.mapped(1, grants[1]).unwrap().is_some());
}

#[test]
fn nested_endpoint_escrow_discard_releases_memory_and_never_rebinds_it() {
    let (mut model, mut ipc) = setup();
    let object = model.allocate(0, 1).unwrap();
    let owner = model.handle(0, object);
    model.freeze(0, owner).unwrap();
    let [send, receive] = ipc.create(0).unwrap();
    let transport = ipc.handle(0, 0).unwrap();
    ipc.send_cap(0, transport, b"endpoint", Some(receive))
        .unwrap();
    // The receive right is still in escrow at B when A queues a page on it.
    let receiver = ipc.memory_receiver(0, send, 1).unwrap();
    assert_eq!(receiver, 1);
    let escrow = model.reserve_read(0, owner, receiver).unwrap();
    ipc.send_memory(0, send, b"x", escrow);
    assert_eq!(
        ipc.receive_memory(1, ipc.handle(1, 0).unwrap(), 64),
        Err(abi::TOO_SMALL)
    );
    ipc.close_task(1);
    let retired = ipc.take_retired_memory();
    assert_eq!(retired[escrow / 64], 1 << (escrow % 64));
    assert_eq!(retired.iter().map(|word| word.count_ones()).sum::<u32>(), 1);
    model.drop_read(escrow);
    assert_eq!(model.charged(1), [0, 0]);
    assert_eq!(ipc.memory_receiver(0, send, 1), Err(abi::PEER_CLOSED));
    // Fresh rights get fresh names even in the same local grant slot.
    let next = model.reserve_read(0, owner, 1).unwrap();
    let fresh = model.publish_read(1, next);
    model.drop_read(next);
    assert_eq!(model.access(1, fresh), Err(abi::BAD_HANDLE));
    assert_eq!(model.access(0, owner), Ok(object));
}
