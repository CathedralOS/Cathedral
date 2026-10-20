use super::{Pending, Work, return_to_boot};

#[test]
fn pending_work_coalesces_rotates_and_cancels_before_reuse() {
    let mut pending = Pending::default();
    for slot in 0..31 {
        pending.insert(Work::Memory, slot);
    }
    pending.insert(Work::Memory, 0);
    for slot in 0..31 {
        assert_eq!(pending.next(Work::Memory), Some(slot));
        pending.insert(Work::Memory, slot); // Multi-step work cannot monopolize dispatch.
    }
    pending.remove(Work::Memory, 0);
    assert_eq!(pending.next(Work::Memory), Some(1));
    for slot in 0..31 {
        pending.remove(Work::Memory, slot);
    }
    assert!(!pending.any());
    pending.insert(Work::Admission, 0);
    assert_eq!(pending.next(Work::Memory), None);
    assert_eq!(pending.next(Work::Admission), Some(0));
    assert!(!pending.any());
}

#[test]
fn cancellation_drain_includes_new_descendants_before_returning_loans() {
    let mut pending = Pending::default();
    pending.insert(Work::Cancel, 1);
    pending.insert(Work::Cancel, 8);
    assert_eq!(pending.first(Work::Cancel), Some(1));
    // Closing this owner adds a descendant even when its launch was earlier in
    // the arena than the already queued unrelated cancellation.
    pending.insert(Work::Cancel, 3);
    pending.insert(Work::Retire, 2);
    pending.insert(Work::Return, 3);
    assert_eq!(pending.first(Work::Cancel), Some(3));
    assert_eq!(pending.first(Work::Cancel), Some(8));
    assert_eq!(pending.first(Work::Cancel), None);
    assert!(pending.any());
    assert_eq!(pending.first(Work::Retire), Some(2));
    assert_eq!(pending.first(Work::Return), Some(3));
    assert!(!pending.any());
}

#[test]
fn only_an_explicit_boot_yield_may_run_peers_past_pending_work() {
    assert!(!return_to_boot(true, false, true));
    assert!(return_to_boot(true, true, false)); // Next user IRQ/syscall repays boot work.
    assert!(return_to_boot(true, false, false)); // Idle IRQ cannot bypass reclamation.
    assert!(return_to_boot(true, true, true));
    assert!(!return_to_boot(false, true, false));
    assert!(!return_to_boot(false, false, false));
}
