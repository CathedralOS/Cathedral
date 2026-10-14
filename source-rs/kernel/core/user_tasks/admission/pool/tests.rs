use super::*;
#[test]
fn high_accounts_delegate_exhaust_and_reclaim_without_truncating_masks() {
    let mut p = Pool::new(8, abi::MAX_TASKS);
    let (child, mask) = p.prepare(0, 3).unwrap();
    assert_eq!(child, 8);
    p.commit(0, child, mask);
    assert_eq!(p.available(0), 0);
    assert_eq!(p.available(child), 3);
    let (leaf, mask) = p.prepare(child, 2).unwrap();
    p.commit(child, leaf, mask);
    assert_eq!(p.available(leaf), 2);
    assert_eq!(p.prepare(child, 0), Err(abi::NO_MEMORY));
    p.reclaim(leaf, child);
    p.release(child, leaf);
    p.reclaim(child, 0);
    p.release(0, child);
    assert_eq!(p.available(0), 4);
}
#[test]
fn funded_nested_slots_cannot_steal_sibling_capacity_and_death_returns_them() {
    let mut p = Pool::new(3, 8);
    p.assign(0, 1, 3).unwrap();
    assert_eq!(p.available(0), 2);
    let (child, descendants) = p.prepare(1, 1).unwrap();
    p.commit(1, child, descendants);
    assert_eq!(p.available(1), 1);
    assert_eq!(p.available(child), 1);
    assert_eq!(p.prepare(1, 1), Err(abi::NO_MEMORY));
    assert_eq!(p.available(0), 2);
    p.reclaim(1, 0);
    p.reclaim(child, 0);
    assert_eq!(p.available(0), 4);
    p.release(0, child);
    assert_eq!(p.available(0), 5);
}
#[test]
fn executable_names_are_local_and_retired_on_death() {
    let mut images = Executables::new(3).unwrap();
    images.install(0, 4, 10).unwrap();
    images.install(1, 5, 10).unwrap();
    assert_eq!(images.resolve(0, 10), Ok(4));
    assert_eq!(images.resolve(1, 10), Ok(5));
    assert_eq!(images.resolve(2, 10), Err(abi::BAD_HANDLE));
    images.close(0);
    images.install(0, 4, 11).unwrap();
    assert_eq!(images.resolve(0, 10), Err(abi::BAD_HANDLE));
    assert_eq!(images.resolve(1, 10), Ok(5));
}
