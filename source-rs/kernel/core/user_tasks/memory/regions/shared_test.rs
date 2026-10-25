//! Mutable views must never acquire the immutable-object promise. Lifetime and
//! sponsor accounting deliberately reuse the already exercised read escrow owner.
use super::*;
use crate::ipc::{EndpointSpec, Ipc};
fn model() -> Regions {
    Regions::new(
        3,
        &[0, 1, 2].map(|task| Grant {
            task,
            private_pages: 4,
            shared_pages: 4,
        }),
    )
    .unwrap()
}
#[test]
fn sharing_is_irreversible_and_cannot_enter_frozen_paths() {
    let mut m = model();
    let object = m.allocate(0, 1).unwrap();
    let owner = m.handle(0, object);
    assert_eq!(m.reserve_view(0, owner, 1, true), Err(abi::DENIED));
    m.share_mutable(0, owner).unwrap();
    assert!(!m.entries[object].sealed);
    assert_eq!(m.freeze(0, owner), Err(abi::BUSY));
    assert_eq!(m.offer(0, owner, 1), Err(abi::BUSY));
    assert_eq!(m.reserve_read(0, owner, 1), Err(abi::DENIED));
    assert_eq!(m.access(1, owner), Err(abi::BAD_HANDLE));
    let escrow = m.reserve_view(0, owner, 1, true).unwrap();
    let reader = m.publish_read(1, escrow);
    assert_eq!(m.share_mutable(1, reader), Err(abi::DENIED));
    assert_eq!(m.freeze(1, reader), Err(abi::DENIED));
    assert_eq!(m.reserve_read(1, reader, 2), Err(abi::DENIED));
    let frozen = m.allocate(0, 1).unwrap();
    let frozen = m.handle(0, frozen);
    m.freeze(0, frozen).unwrap();
    assert_eq!(m.share_mutable(0, frozen), Err(abi::BUSY));
    assert_eq!(m.reserve_view(0, frozen, 1, true), Err(abi::DENIED));
}
#[test]
fn mutable_escrow_requires_distinct_acceptance_and_survives_creator_death() {
    let mut m = model();
    let mut ipc = Ipc::new(
        1,
        3,
        &[EndpointSpec {
            sender: 0,
            receiver: 1,
            revoker: Some(0),
        }],
    )
    .unwrap();
    let object = m.allocate(0, 2).unwrap();
    let owner = m.handle(0, object);
    m.share_mutable(0, owner).unwrap();
    let escrow = m.reserve_view(0, owner, 1, true).unwrap();
    let pending = m.reads[escrow].unwrap().name;
    assert_eq!(m.access(1, pending), Err(abi::BAD_HANDLE));
    let send = ipc.handle(0, 0).unwrap();
    let recv = ipc.handle(1, 0).unwrap();
    ipc.send_memory_kind(0, send, b"view", escrow, true);
    assert_eq!(ipc.receive_memory(1, recv, 64), Err(abi::TOO_SMALL));
    assert_eq!(ipc.receive(1, recv, 64), Err(abi::TOO_SMALL));
    assert_eq!(ipc.receive_shared(1, recv, 1), Err(abi::TOO_SMALL));
    m.close(object, 0);
    ipc.close_task(0);
    assert_eq!(m.charged(0)[0], 2);
    let offer = ipc
        .receive_shared(1, recv, 64)
        .unwrap()
        .unwrap()
        .memory
        .unwrap();
    assert_eq!(m.publish_read(1, offer), pending);
    let forwarded = m.reserve_view(1, pending, 2, true).unwrap();
    let name = m.publish_read(2, forwarded);
    m.drop_read(offer);
    assert_eq!(m.access(1, pending), Err(abi::BAD_HANDLE));
    assert_eq!(m.access(2, name), Ok(object));
    assert_eq!(m.charged(0)[0], 2);
    m.drop_read(forwarded);
    assert!(!m.entries[object].live());
    assert_eq!(m.charged(0)[0], 0);
    let next = m.allocate(0, 1).unwrap();
    assert_ne!(m.handle(0, next), owner);
}
#[test]
fn mutable_escrow_reserves_quota_and_discard_returns_it() {
    let mut m = model();
    let mut ipc = Ipc::new(
        1,
        3,
        &[EndpointSpec {
            sender: 0,
            receiver: 1,
            revoker: Some(0),
        }],
    )
    .unwrap();
    let object = m.allocate(0, 4).unwrap();
    let owner = m.handle(0, object);
    m.share_mutable(0, owner).unwrap();
    let grant = m.reserve_view(0, owner, 1, true).unwrap();
    assert_eq!(m.reserve_view(0, owner, 1, true), Err(abi::NO_MEMORY));
    ipc.send_memory_kind(0, ipc.handle(0, 0).unwrap(), b"", grant, true);
    ipc.revoke(0, ipc.handle(0, 1).unwrap()).unwrap();
    let retired = ipc.take_retired_memory();
    assert_eq!(retired[grant / 64], 1 << (grant % 64));
    m.drop_read(grant);
    assert_eq!(m.charged(1)[1], 0);
    assert_eq!(m.charged(0)[0], 4);
    m.release(0, owner).unwrap();
    assert_eq!(m.charged(0)[0], 0);
}
