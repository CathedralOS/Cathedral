//! Real Omega catalogue state: instance identity, selection and retirement.
use crate::policy::catalogue::{Catalogue, Selection as S};

fn ticket(c: &Catalogue, slot: usize) -> u64 {
    c.entry(slot as u64 - 1).unwrap()[1]
}
#[test]
fn repeated_programs_are_distinct_instances_and_switches_keep_their_tickets() {
    let mut c = Catalogue::default();
    for slot in 1..=3 {
        assert!(c.publish(slot, 42));
        assert_eq!(ticket(&c, slot), slot as u64);
        assert_eq!(c.entry(slot as u64 - 1).unwrap()[2], 42);
    }
    for slot in [3, 1, 2, 3] {
        assert_eq!(c.select(ticket(&c, slot), false), S::Switch(slot));
        assert!(c.activate(slot));
        assert_eq!(c.select(ticket(&c, slot), false), S::Current);
        assert_eq!(c.active(), Some(slot));
        assert_eq!(c.fallback(), None);
        for other in 1..=3 {
            assert_eq!(ticket(&c, other), other as u64);
            assert_eq!(
                c.entry(other as u64 - 1).unwrap()[3],
                u64::from(other == slot)
            );
        }
    }
}
#[test]
fn retired_ticket_never_selects_a_replacement_even_for_the_same_program() {
    let mut c = Catalogue::default();
    for _ in 0..16 {
        assert!(c.publish(1, 42));
        let old = ticket(&c, 1);
        assert!(c.activate(1));
        c.retire(1);
        assert_eq!(c.active(), None);
        assert_eq!(c.entry(0), Some([0; 8]));
        assert!(!c.activate(1));
        assert!(c.publish(1, 42));
        assert!(ticket(&c, 1) > old);
        assert_eq!(c.select(old, false), S::Missing);
        assert_eq!(c.select(old, true), S::Missing);
        c.retire(1);
    }
}
#[test]
fn busy_precedes_current_but_unknown_and_zero_tickets_are_always_missing() {
    let mut c = Catalogue::default();
    assert!(c.publish(2, 99));
    assert!(c.activate(2));
    assert_eq!(c.select(ticket(&c, 2), true), S::Busy);
    assert_eq!(c.select(ticket(&c, 2), false), S::Current);
    for busy in [false, true] {
        assert_eq!(c.select(0, busy), S::Missing);
        assert_eq!(c.select(u64::MAX, busy), S::Missing);
    }
}
#[test]
fn fallback_chooses_first_survivor_only_when_active_instance_is_gone() {
    for mask in 0..8 {
        for removed in 1..=3 {
            let mut c = Catalogue::default();
            for slot in 1..=3 {
                if mask & (1 << (slot - 1)) != 0 {
                    assert!(c.publish(slot, slot as u64));
                }
            }
            let first = (1..=3).find(|s| ticket(&c, *s) != 0);
            assert_eq!(c.fallback(), first);
            if let Some(first) = first {
                assert!(c.activate(first));
                c.retire(removed);
                if removed == first {
                    assert_eq!(c.active(), None);
                    assert_eq!(c.fallback(), (1..=3).find(|s| ticket(&c, *s) != 0));
                } else {
                    assert_eq!(c.active(), Some(first));
                    assert_eq!(c.fallback(), None);
                }
            }
        }
    }
}
#[test]
fn invalid_positions_and_duplicate_publication_cannot_damage_the_catalogue() {
    let mut c = Catalogue::default();
    assert!(c.publish(2, 42));
    assert!(c.activate(2));
    for slot in [0, 4, 5, usize::MAX] {
        assert!(!c.publish(slot, 7));
        assert!(!c.activate(slot));
        c.retire(slot);
    }
    for index in [3, 4, u64::MAX] {
        assert_eq!(c.entry(index), None);
    }
    assert!(!c.publish(2, 7));
    assert_eq!(c.entry(1), Some([0, 1, 42, 1, 0, 0, 0, 0]));
    assert!(c.publish(3, 7));
    assert_eq!(
        ticket(&c, 3),
        2,
        "refused publication must not consume a ticket"
    );
}
#[test]
fn ticket_exhaustion_does_not_wrap_or_reopen_after_retirement() {
    // Seed only the private native fixture, not a production reset/import API.
    let mut p = [0; 128];
    p[0] = 13;
    p[16] = u64::MAX - 1;
    p[2] = 1;
    p[3] = 42;
    assert_eq!(super::decide(&mut p), 1);
    assert_eq!(p[18], u64::MAX);
    p[1] = 7;
    assert_eq!(super::decide(&mut p), 0);
    p[1] = 0;
    p[2] = 2;
    assert_eq!(super::decide(&mut p), 0);
    assert_eq!(p[20], 0);
    p[1] = 1;
    p[2] = 1;
    assert_eq!(super::decide(&mut p), 1);
    p[1] = 0;
    assert_eq!(super::decide(&mut p), 0);
    assert_eq!(p[16], u64::MAX);
    assert_eq!(p[18], 0);
}
