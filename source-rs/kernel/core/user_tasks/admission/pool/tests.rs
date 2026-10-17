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
    images.install(0, Code::Template(4), 10).unwrap();
    images.install(1, Code::Template(5), 10).unwrap();
    assert_eq!(images.resolve(0, 10), Ok(Code::Template(4)));
    assert_eq!(images.resolve(1, 10), Ok(Code::Template(5)));
    assert_eq!(images.resolve(2, 10), Err(abi::BAD_HANDLE));
    assert_eq!(images.close(0), [None; abi::MAX_LAUNCHES]);
    images.install(0, Code::Template(4), 11).unwrap();
    assert_eq!(images.resolve(0, 10), Err(abi::BAD_HANDLE));
    assert_eq!(images.resolve(1, 10), Ok(Code::Template(5)));
}
#[test]
fn object_grants_are_distinct_single_use_and_report_their_references_on_close() {
    let mut images = Executables::new(2).unwrap();
    let first = Code::Object { region: 3, role: 2 };
    let second = Code::Object { region: 3, role: 7 };
    // A template is shared per holder; every object grant is its own reference.
    assert_eq!(images.install(0, Code::Template(1), 20), Ok(20));
    assert_eq!(images.install(0, Code::Template(1), 21), Ok(20));
    assert_eq!(images.install(0, first, 22), Ok(22));
    assert_eq!(images.install(0, second, 23), Ok(23));
    assert_eq!(images.resolve(0, 23), Ok(second));
    assert_eq!(images.take(0, 22), Ok(first));
    assert_eq!(images.resolve(0, 22), Err(abi::BAD_HANDLE));
    assert_eq!(images.take(0, 22), Err(abi::BAD_HANDLE));
    let mut held = [None; abi::MAX_LAUNCHES];
    held[2] = Some(3);
    assert_eq!(images.close(0), held);
    assert_eq!(images.resolve(0, 20), Err(abi::BAD_HANDLE));
}
