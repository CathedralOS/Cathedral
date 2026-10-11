use super::*;
fn model() -> Regions {
    Regions::new(
        3,
        &[
            Grant {
                task: 0,
                private_pages: 4,
                shared_pages: 0,
            },
            Grant {
                task: 1,
                private_pages: 4,
                shared_pages: 4,
            },
            Grant {
                task: 2,
                private_pages: 4,
                shared_pages: 0,
            },
        ],
    )
    .unwrap()
}
fn allocate(regions: &mut Regions, owner: usize, pages: usize) -> (usize, u64) {
    let i = regions.allocate(owner, pages).unwrap();
    (i, regions.handle(owner, i))
}
#[test]
fn local_names_and_addresses_ignore_unrelated_arena_activity() {
    let mut a = model();
    let mut b = model();
    for _ in 0..3 {
        allocate(&mut b, 2, 1);
    }
    let (ai, ah) = allocate(&mut a, 0, 1);
    let (bi, bh) = allocate(&mut b, 0, 1);
    assert_ne!(ai, bi);
    assert_eq!(ah, bh);
    assert_eq!(a.mapping(0, ai), b.mapping(0, bi));
    for regions in [&mut a, &mut b] {
        assert_eq!(regions.access(1, ah), Err(abi::BAD_HANDLE));
        assert_eq!(regions.access(1, 900), Err(abi::BAD_HANDLE));
        assert_eq!(regions.release(1, ah), Err(abi::BAD_HANDLE));
        regions.offer(0, ah, 1).unwrap();
        let index = regions.accept(1, 0, ah).unwrap();
        assert_eq!(regions.handle(1, index), 1);
        assert_eq!(regions.mapping(1, index), MAX_PRIVATE_REGIONS);
    }
}
#[test]
fn same_numeric_offer_from_different_producers_is_scoped_by_link() {
    let mut regions = model();
    let (a, ah) = allocate(&mut regions, 0, 1);
    let (b, bh) = allocate(&mut regions, 2, 1);
    assert_eq!(ah, bh); // Names are local, not bearer credentials across processes.
    regions.offer(0, ah, 1).unwrap();
    assert_eq!(regions.accept(1, 2, ah), Err(abi::BAD_HANDLE));
    assert!(!regions.entries[a].accepted);
    regions.offer(2, bh, 1).unwrap();
    assert_eq!(regions.accept(1, 2, bh), Ok(b));
    assert_eq!(regions.accept(1, 0, ah), Ok(a));
    let ar = regions.handle(1, a);
    let br = regions.handle(1, b);
    assert_ne!(ar, br);
    assert_eq!(regions.access(1, ar), Ok(a));
    assert_eq!(regions.access(1, br), Ok(b));
}
#[test]
fn budgets_stale_handles_and_reader_completion_are_bound() {
    let mut regions = model();
    let (index, handle) = allocate(&mut regions, 0, 4);
    assert_eq!(regions.allocate(0, 1), Err(abi::NO_MEMORY));
    assert_eq!(regions.offer(1, handle, 0), Err(abi::BAD_HANDLE));
    assert_eq!(regions.offer(0, handle, 2), Err(abi::DENIED));
    regions.offer(0, handle, 1).unwrap();
    assert_eq!(regions.release(0, handle), Err(abi::BUSY));
    assert_eq!(regions.accept(2, 0, handle), Err(abi::BAD_HANDLE));
    regions.accept(1, 0, handle).unwrap();
    let reader = regions.handle(1, index);
    assert_eq!(regions.accept(1, 0, handle), Err(abi::BUSY));
    regions.release(1, reader).unwrap();
    assert_eq!(regions.access(1, reader), Err(abi::BAD_HANDLE));
    regions.release(0, handle).unwrap();
    let (reused, fresh) = allocate(&mut regions, 0, 1);
    assert_eq!(index, reused);
    assert_ne!(fresh, handle);
    assert_eq!(regions.access(0, handle), Err(abi::BAD_HANDLE));
}
#[test]
fn owner_failure_preserves_only_accepted_readers_and_their_budget() {
    for accepted in [false, true] {
        let mut regions = model();
        let (index, handle) = allocate(&mut regions, 0, 3);
        regions.offer(0, handle, 1).unwrap();
        if accepted {
            regions.accept(1, 0, handle).unwrap();
        }
        let reader = regions.entries[index].reader_name;
        regions.close(index, 0);
        assert_eq!(regions.entries[index].live(), accepted);
        assert_eq!(regions.access(0, handle), Err(abi::BAD_HANDLE));
        if accepted {
            assert_eq!(regions.access(1, reader), Ok(index));
            let (_, other) = allocate(&mut regions, 2, 2);
            regions.offer(2, other, 1).unwrap();
            assert_eq!(regions.accept(1, 2, other), Err(abi::NO_MEMORY));
            regions.release(1, reader).unwrap();
            regions.accept(1, 2, other).unwrap();
        }
        assert!(!regions.entries[index].live());
    }
}
#[test]
fn receiver_replacement_cannot_reuse_old_offer_or_mapping() {
    let mut regions = model();
    let (index, handle) = allocate(&mut regions, 0, 2);
    regions.offer(0, handle, 1).unwrap();
    regions.accept(1, 0, handle).unwrap();
    let old = regions.handle(1, index);
    regions.close(index, 1);
    assert_eq!(regions.accept(1, 0, handle), Err(abi::BAD_HANDLE));
    assert_eq!(regions.access(1, old), Err(abi::BAD_HANDLE));
    regions.release(0, handle).unwrap();
    let (index, fresh) = allocate(&mut regions, 0, 2);
    regions.offer(0, fresh, 1).unwrap();
    regions.accept(1, 0, fresh).unwrap();
    assert_ne!(regions.handle(1, index), old);
    assert_eq!(regions.access(1, old), Err(abi::BAD_HANDLE));
}
#[test]
fn global_capacity_covers_all_object_budgets_including_orphan_readers() {
    let grants = core::array::from_fn::<_, { abi::MAX_TASKS }, _>(|task| Grant {
        task,
        private_pages: MAX_PAGES,
        shared_pages: MAX_PAGES,
    });
    let mut regions = Regions::new(abi::MAX_TASKS, &grants).unwrap();
    for owner in 0..abi::MAX_TASKS {
        for _ in 0..MAX_SHARED_REGIONS {
            let (index, handle) = allocate(&mut regions, owner, 1);
            let peer = (owner + 1) % abi::MAX_TASKS;
            regions.offer(owner, handle, peer).unwrap();
            regions.accept(peer, owner, handle).unwrap();
            regions.close(index, owner);
        }
    }
    for owner in 0..abi::MAX_TASKS {
        for _ in 0..MAX_PRIVATE_REGIONS {
            allocate(&mut regions, owner, 1);
        }
        assert_eq!(regions.allocate(owner, 1), Err(abi::NO_MEMORY));
    }
    assert_eq!(
        regions.entries.iter().filter(|r| r.live()).count(),
        MAX_REGIONS
    );
    let reader = regions.handle(1, 0);
    regions.release(1, reader).unwrap();
    assert_eq!(regions.allocate(0, 1), Err(abi::NO_MEMORY));
    let owned = regions
        .entries
        .iter()
        .position(|r| r.owner == Some(0))
        .unwrap();
    regions.release(0, regions.handle(0, owned)).unwrap();
    assert_eq!(regions.allocate(0, MAX_PAGES - 3), Ok(0));
    assert_eq!(regions.access(1, reader), Err(abi::BAD_HANDLE));
}
#[test]
fn object_pressure_is_separate_from_page_pressure() {
    let mut regions = model();
    for _ in 0..MAX_PRIVATE_REGIONS {
        allocate(&mut regions, 0, 1);
    }
    assert_eq!(regions.allocate(0, 1), Err(abi::NO_MEMORY));
    let (peer, _) = allocate(&mut regions, 1, 4);
    for index in 0..MAX_SHARED_REGIONS {
        let h = regions.handle(0, index);
        regions.offer(0, h, 1).unwrap();
        regions.accept(1, 0, h).unwrap();
        regions.close(index, 0);
    }
    let (extra, h) = allocate(&mut regions, 2, 1);
    regions.offer(2, h, 1).unwrap();
    assert_eq!(regions.accept(1, 2, h), Err(abi::NO_MEMORY));
    assert!(!regions.entries[extra].accepted);
    regions.release(1, regions.handle(1, 0)).unwrap();
    regions.accept(1, 2, h).unwrap();
    regions.release(1, regions.handle(1, peer)).unwrap();
    assert!(regions.allocate(1, 4).is_ok());
}
#[test]
fn invalid_grants_and_tokens_fail_closed() {
    for grants in [
        alloc::vec![Grant {
            task: 2,
            private_pages: 1,
            shared_pages: 0
        }],
        alloc::vec![Grant {
            task: 0,
            private_pages: MAX_PAGES + 1,
            shared_pages: 0
        }],
        alloc::vec![Grant {
            task: 0,
            private_pages: 0,
            shared_pages: MAX_PAGES + 1
        }],
        alloc::vec![Grant { task: 0, private_pages: 1, shared_pages: 0 }; 2],
    ] {
        assert!(Regions::new(2, &grants).is_err());
    }
    assert!(Regions::new(abi::MAX_TASKS + 1, &[]).is_err());
    let mut regions = model();
    allocate(&mut regions, 0, 1);
    for token in [0, 256, MAX_REGIONS as u64 + 1, u64::MAX] {
        assert_eq!(regions.access(0, token), Err(abi::BAD_HANDLE));
    }
}
