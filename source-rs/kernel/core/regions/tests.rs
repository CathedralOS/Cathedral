use super::*;
#[test]
fn producer_check_does_not_consume_another_clients_offer() {
    let mut regions = model();
    let index = regions.allocate(0, 1, 99).unwrap();
    let handle = regions.entries[index].handle(index);
    regions.offer(0, handle, 1).unwrap();
    assert_eq!(regions.check_owner(handle, 2), Err(abi::DENIED));
    assert!(!regions.entries[index].accepted);
    assert_eq!(regions.check_owner(handle, 0), Ok(()));
    regions.accept(1, handle).unwrap();
    regions.close(index, 0);
    assert_eq!(regions.check_owner(handle, 0), Err(abi::DENIED));
    assert_eq!(regions.access(1, handle), Ok(index));
}
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
        ],
    )
    .unwrap()
}
#[test]
fn budgets_handles_and_readers_are_bound() {
    let mut regions = model();
    let index = regions.allocate(0, 4, 1).unwrap();
    let handle = regions.entries[index].handle(index);
    assert_eq!(regions.allocate(0, 1, 2), Err(abi::NO_MEMORY));
    assert_eq!(regions.allocate(2, 1, 2), Err(abi::DENIED));
    assert_eq!(regions.access(1, handle), Err(abi::DENIED));
    assert_eq!(regions.offer(1, handle, 0), Err(abi::DENIED));
    assert_eq!(regions.offer(0, handle, 2), Err(abi::DENIED));
    regions.offer(0, handle, 1).unwrap();
    assert_eq!(regions.release(0, handle), Err(abi::BUSY));
    assert_eq!(regions.accept(2, handle), Err(abi::DENIED));
    regions.accept(1, handle).unwrap();
    assert_eq!(regions.accept(1, handle), Err(abi::BUSY));
    regions.release(1, handle).unwrap();
    regions.release(0, handle).unwrap();
    let reused = regions.allocate(0, 1, 2).unwrap();
    assert_eq!(index, reused);
    assert_eq!(regions.find(handle), Err(abi::BAD_HANDLE));
}
#[test]
fn owner_failure_preserves_only_an_accepted_readers_lease() {
    for accepted in [false, true] {
        let mut regions = model();
        let index = regions.allocate(0, 1, 1).unwrap();
        let handle = regions.entries[index].handle(index);
        regions.offer(0, handle, 1).unwrap();
        if accepted {
            regions.accept(1, handle).unwrap();
        }
        regions.close(index, 0);
        assert_eq!(regions.entries[index].live(), accepted);
        if accepted {
            assert_eq!(regions.access(0, handle), Err(abi::DENIED));
            regions.release(1, handle).unwrap();
        }
        assert!(!regions.entries[index].live());
    }
}
#[test]
fn receiver_death_does_not_grant_its_replacement_the_old_offer() {
    let mut regions = model();
    let index = regions.allocate(0, 2, 1).unwrap();
    let handle = regions.entries[index].handle(index);
    regions.offer(0, handle, 1).unwrap();
    regions.close(index, 1);
    assert_eq!(regions.accept(1, handle), Err(abi::DENIED));
    assert!(regions.entries[index].sealed);
    regions.release(0, handle).unwrap();
    assert!(!regions.entries[index].live());
}
#[test]
fn accepted_pages_remain_charged_after_owner_death() {
    let mut regions = model();
    let a = regions.allocate(0, 3, 1).unwrap();
    let ah = regions.entries[a].handle(a);
    let b = regions.allocate(1, 2, 2).unwrap();
    // Give the second object to another authorized producer's reader using a separate model grant.
    regions.private[2] = 4;
    let c = regions.allocate(2, 2, 3).unwrap();
    let ch = regions.entries[c].handle(c);
    regions.offer(0, ah, 1).unwrap();
    regions.offer(2, ch, 1).unwrap();
    regions.accept(1, ah).unwrap();
    regions.close(a, 0);
    assert_eq!(regions.accept(1, ch), Err(abi::NO_MEMORY));
    regions.release(1, ah).unwrap();
    regions.accept(1, ch).unwrap();
    regions.close(c, 1);
    regions.close(b, 1);
    assert!(!regions.entries[b].live());
    assert_eq!(regions.entries[c].owner, Some(2));
}

#[test]
fn global_capacity_accounts_for_readers_left_by_exited_producers() {
    let grants = core::array::from_fn::<_, { abi::MAX_TASKS }, _>(|task| Grant {
        task,
        private_pages: 4,
        shared_pages: 4,
    });
    let mut regions = Regions::new(abi::MAX_TASKS, &grants).unwrap();
    for epoch in 1..=MAX_REGIONS {
        regions.allocate((epoch - 1) / 4, 1, epoch as u64).unwrap();
    }
    let handle = regions.entries[0].handle(0);
    regions.offer(0, handle, 1).unwrap();
    regions.accept(1, handle).unwrap();
    regions.close(0, 0);
    assert_eq!(regions.allocate(0, 1, 100), Err(abi::NO_MEMORY));
    regions.release(1, handle).unwrap();
    assert_eq!(regions.allocate(0, 1, 101), Ok(0));
    assert_eq!(regions.find(handle), Err(abi::BAD_HANDLE));
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
            private_pages: 5,
            shared_pages: 0
        }],
        alloc::vec![Grant {
            task: 0,
            private_pages: 0,
            shared_pages: 5
        }],
        alloc::vec![Grant { task: 0, private_pages: 1, shared_pages: 0 }; 2],
    ] {
        assert!(Regions::new(2, &grants).is_err());
    }
    assert!(Regions::new(abi::MAX_TASKS + 1, &[]).is_err());
    let mut regions = model();
    regions.allocate(0, 1, 1).unwrap();
    for token in [0, 256, 33, u64::MAX, (2 << 8) | 1] {
        assert_eq!(regions.find(token), Err(abi::BAD_HANDLE));
    }
}
