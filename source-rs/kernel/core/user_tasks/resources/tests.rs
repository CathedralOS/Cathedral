use super::*;
use memory::Grant;

#[test]
fn live_growth_is_atomic_across_five_families_and_conserves_existing_charges() {
    use crate::user_tasks::admission::pool::Pool;
    let mut ipc = Ipc::with_reservations(1, 6, &[], &[16, 0, 0, 0, 0, 0]).unwrap();
    let mut regions = Regions::new(
        6,
        &[Grant {
            task: 0,
            private_pages: 32,
            shared_pages: 32,
        }],
    )
    .unwrap();
    let mut frames = physical::Funding::new(core::array::from_fn(|i| if i == 0 { 100 } else { 0 }));
    let mut pool = Pool::new(1, 6);
    let (child, mask) = pool.prepare(0, 1).unwrap();
    pool.commit(0, child, mask);
    grow(
        &mut ipc,
        &mut regions,
        &mut frames,
        &mut pool,
        0,
        child,
        [8, 8, 4, 1, 30],
    )
    .unwrap();
    let object = regions.allocate(child, 4).unwrap();
    ipc.create(child).unwrap();
    frames.charge(child, 10).unwrap();
    let before = snapshot(&ipc, &regions, 0);
    let held = snapshot(&ipc, &regions, child);
    for delta in [
        [1, 1, 99, 1, 1],
        [1, 1, 1, 9, 1],
        [1, 1, 1, 1, 999],
        [usize::MAX, 0, 0, 0, 0],
    ] {
        assert!(
            grow(
                &mut ipc,
                &mut regions,
                &mut frames,
                &mut pool,
                0,
                child,
                delta
            )
            .is_err()
        );
        assert_eq!(snapshot(&ipc, &regions, 0), before);
        assert_eq!(snapshot(&ipc, &regions, child), held);
        assert_eq!(pool.available(child), 2);
        assert_eq!(frames.snapshot(child), [30, 10]);
    }
    grow(
        &mut ipc,
        &mut regions,
        &mut frames,
        &mut pool,
        0,
        child,
        [4, 4, 2, 1, 10],
    )
    .unwrap();
    assert_eq!(snapshot(&ipc, &regions, child), [12, 12, 6, 4, 0, 1]);
    assert_eq!(regions.entries[object].account, Some(child));
    assert_eq!(frames.snapshot(child), [40, 10]);
    assert_eq!(frames.capacity(), 100);
    assert_eq!(pool.available(0) + pool.available(child), 4);
}

#[test]
fn nested_subdivision_conserves_all_three_pools_and_failed_assignment_is_atomic() {
    let mut ipc = Ipc::with_reservations(1, 4, &[], &[8, 0, 0, 4]).unwrap();
    let mut regions = Regions::new(
        4,
        &[Grant {
            task: 0,
            private_pages: 16,
            shared_pages: 16,
        }],
    )
    .unwrap();
    assign(&mut ipc, &mut regions, 0, 1, [12, 10, 6]).unwrap();
    assign(&mut ipc, &mut regions, 1, 2, [8, 6, 4]).unwrap();
    assert_eq!(snapshot(&ipc, &regions, 0)[..3], [4, 6, 2]);
    assert_eq!(snapshot(&ipc, &regions, 1)[..3], [4, 4, 2]);
    assert_eq!(snapshot(&ipc, &regions, 2)[..3], [8, 6, 4]);
    let before = snapshot(&ipc, &regions, 1);
    assert_eq!(
        assign(&mut ipc, &mut regions, 1, 2, [9, 7, 7]),
        Err(abi::NO_MEMORY)
    );
    assert_eq!(snapshot(&ipc, &regions, 1), before);
    assert_eq!(snapshot(&ipc, &regions, 2)[..3], [8, 6, 4]);
    for _ in 0..4 {
        ipc.create(3).unwrap();
    }
    assert_eq!(ipc.create(3), Err(abi::NO_MEMORY));
}

#[test]
fn accepted_pages_pin_original_sponsor_through_death_and_cannot_be_withdrawn() {
    let mut ipc = Ipc::with_reservations(1, 3, &[], &[0, 0, 0]).unwrap();
    let mut regions = Regions::new(
        3,
        &[
            Grant {
                task: 0,
                private_pages: 2,
                shared_pages: 0,
            },
            Grant {
                task: 2,
                private_pages: 0,
                shared_pages: 2,
            },
        ],
    )
    .unwrap();
    assign(&mut ipc, &mut regions, 0, 1, [2, 0, 0]).unwrap();
    let index = regions.allocate(1, 2).unwrap();
    let name = regions.handle(1, index);
    regions.offer(1, name, 2).unwrap();
    regions.accept(2, 1, name).unwrap();
    let reader = regions.handle(2, index);
    regions.close(index, 1);
    assert_eq!(regions.allocate(1, 1), Err(abi::NO_MEMORY));
    assert_eq!(
        assign(&mut ipc, &mut regions, 0, 1, [0, 0, 0]),
        Err(abi::BUSY)
    );
    assert_eq!(regions.access(2, reader), Ok(index));
    regions.release(2, reader).unwrap();
    assign(&mut ipc, &mut regions, 0, 1, [0, 0, 0]).unwrap();
    assert_eq!(regions.allocate(0, 2), Ok(index));
}

#[test]
fn retained_endpoint_prevents_withdrawing_its_capacity_after_creator_death() {
    let mut ipc = Ipc::with_reservations(1, 2, &[], &[2, 0]).unwrap();
    let mut regions = Regions::new(2, &[]).unwrap();
    assign(&mut ipc, &mut regions, 0, 1, [0, 0, 1]).unwrap();
    ipc.close_task(1);
    ipc.prepare_child(0, 0, 1, 2);
    ipc.accept_child(0, 0);
    let [_, receive] = ipc.create(1).unwrap();
    ipc.send_cap(1, ipc.handle(1, 1).unwrap(), b"retain", Some(receive))
        .unwrap();
    let received = ipc
        .receive_cap(0, ipc.handle(0, 1).unwrap(), 64, true)
        .unwrap()
        .unwrap()
        .attachment
        .unwrap();
    ipc.close_task(1);
    assert_eq!(
        assign(&mut ipc, &mut regions, 0, 1, [0, 0, 0]),
        Err(abi::BUSY)
    );
    ipc.close(0, received).unwrap();
    assign(&mut ipc, &mut regions, 0, 1, [0, 0, 0]).unwrap();
    assert_eq!(ipc.budget(0).unwrap()[0], 2);
}
