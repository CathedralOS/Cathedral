use crate::policy::{self, Action, Operation};
use cathedral_desktop_matrix_contracts::matrix as wire;
use cathedral_platform_contracts::{input, storage::metadata::Metadata};

fn view(width: u64, height: u64) -> wire::Packet {
    [0, 19, width, height, 0, 0, 0, 0]
}
fn key(code: u8, pressed: bool) -> wire::Packet {
    [0, 19, wire::KEY, code.into(), pressed.into(), 0, 700, 1]
}
fn pointer(x: u64, y: u64) -> wire::Packet {
    [0, 19, wire::POINTER, x, y, 1, 700, 1]
}

#[test]
fn compiled_contract_and_metadata_are_exact() {
    policy::verify();
    let (flags, label) =
        policy::classify(Metadata::new(b"Template", b"application/x-executable").unwrap());
    assert_eq!(flags, 3);
    assert_eq!(&label[..8], b"TEMPLATE");
    assert_eq!(
        policy::classify(Metadata::new(b"Background", b"application/x-executable").unwrap()).0,
        5
    );
    for name in [b"Template.exe".as_slice(), b"template", b"Notes.exe"] {
        assert_eq!(
            policy::classify(Metadata::new(name, b"text/plain").unwrap()).0,
            0
        );
    }
    assert_eq!(
        policy::classify(Metadata::new(b"Program", b"application/x-executableX").unwrap()).0,
        0
    );
    assert_eq!(
        policy::classify(Metadata::new(b"Program", b"application/x-executabl").unwrap()).0,
        0
    );
    let (_, label) = policy::classify(Metadata::new(b"abcdefghijklmnop", b"").unwrap());
    assert_eq!(&label, b"ABCDEFGHIJKLM");
}

#[test]
fn catalogue_resolves_hidden_roles_and_bounded_local_objects() {
    let objects = [80, 19, 40, 51, 70, 91, 92, 93, 94];
    let (ids, count) = policy::catalogue(true, 999, 40, 70, &objects).unwrap();
    assert_eq!(count, 4);
    assert_eq!(ids, [40, 80, 19, 51, 0, 0]);
    let (ids, count) = policy::catalogue(false, 51, 40, 70, &objects).unwrap();
    assert_eq!(count, 6);
    assert_eq!(ids, [51, 80, 19, 40, 91, 92]);
    assert!(policy::catalogue(false, 999, 40, 70, &objects).is_none());
    assert!(policy::catalogue(true, 51, 0, 70, &objects).is_none());
    assert_eq!(
        policy::catalogue(true, 0, 40, 0, &[]).unwrap(),
        ([40, 0, 0, 0, 0, 0], 1)
    );
    let page: [u64; 51] = core::array::from_fn(|i| (1u64 << 50) + i as u64);
    let (ids, count) = policy::catalogue(false, page[50], 0, page[0], &page).unwrap();
    assert_eq!(count, 6);
    assert_eq!(ids, [page[50], page[1], page[2], page[3], page[4], page[5]]);
}

#[test]
fn tiles_and_clicks_share_geometry_at_all_supported_counts() {
    for picker in [false, true] {
        for count in 1..=if picker { 4 } else { 6 } {
            for (w, h) in [(1200, 800), (640, 96), (64, 24)] {
                let geometry = policy::geometry(picker, count, w, h, 0);
                assert_eq!(geometry.count, count - 1 + usize::from(picker && count < 4));
                for i in 0..geometry.count {
                    let tile = policy::geometry(picker, count, w, h, i);
                    let (x, y) = tile.origin;
                    // Skip overlapping chrome controls at tiny synthetic extents.
                    if !picker && x >= w.saturating_sub(184) {
                        continue;
                    }
                    let (selected, changed, activate) = policy::selection(
                        1,
                        pointer(x, y),
                        view(w, h),
                        true,
                        count,
                        picker,
                        false,
                        false,
                    );
                    assert_eq!(
                        (selected, changed, activate),
                        (geometry.items[i], true, true)
                    );
                    let (_, changed, activate) = policy::selection(
                        1,
                        pointer(x, y),
                        view(w, h),
                        false,
                        count,
                        picker,
                        false,
                        false,
                    );
                    assert_eq!((changed, activate), (false, false));
                }
            }
        }
    }
    let login = policy::geometry(true, 2, 1024, 768, 0);
    assert_eq!(login.origin, (372, 336));
    assert_eq!(login.dimensions, (128, 96));
    assert_eq!(policy::geometry(false, 5, 1024, 96, 3).origin, (176, 24));
}

#[test]
fn keyboard_navigation_wraps_and_preserves_press_semantics() {
    let viewport = view(1200, 96);
    assert_eq!(
        policy::selection(
            1,
            key(input::LEFT, true),
            viewport,
            false,
            3,
            true,
            false,
            false
        ),
        (policy::NEW, true, false)
    );
    assert_eq!(
        policy::selection(
            policy::NEW,
            key(input::RIGHT, true),
            viewport,
            false,
            3,
            true,
            false,
            false
        ),
        (1, true, false)
    );
    assert_eq!(
        policy::selection(
            1,
            key(input::ENTER, false),
            viewport,
            false,
            3,
            true,
            false,
            false
        ),
        (1, false, false)
    );
    assert_eq!(
        policy::selection(
            1,
            key(input::F2, true),
            viewport,
            false,
            3,
            true,
            false,
            false
        ),
        (1, false, false)
    );
    assert_eq!(
        policy::selection(
            1,
            key(input::F2, true),
            viewport,
            false,
            3,
            false,
            false,
            false
        ),
        (7, true, true)
    );
    assert_eq!(
        policy::selection(
            1,
            key(input::F3, true),
            viewport,
            false,
            3,
            false,
            false,
            false
        ),
        (1, false, false)
    );
    assert_eq!(
        policy::selection(
            1,
            key(input::F3, true),
            viewport,
            false,
            3,
            false,
            true,
            false
        ),
        (8, true, true)
    );
    assert_eq!(
        policy::selection(
            1,
            key(input::F4, true),
            viewport,
            false,
            3,
            false,
            false,
            true
        ),
        (9, true, true)
    );
    assert_eq!(
        policy::selection(
            1,
            key(input::RIGHT, true),
            viewport,
            false,
            1,
            false,
            false,
            false
        ),
        (0, true, false)
    );
}

#[test]
fn control_hit_regions_do_not_spill_into_each_other() {
    let viewport = view(1200, 96);
    for (y, expected) in [(24, 8), (47, 8), (48, 9), (63, 9), (64, 7), (95, 7)] {
        assert_eq!(
            policy::selection(1, pointer(1016, y), viewport, true, 6, false, true, true),
            (expected, true, true)
        );
    }
    assert_eq!(
        policy::selection(1, pointer(1015, 24), viewport, true, 6, false, true, true),
        (1, false, false)
    );
}

#[test]
fn actions_never_turn_hidden_or_missing_tiles_into_launches() {
    let ids = [90, 7, 33, 0, 0, 0];
    assert_eq!(policy::action(true, 3, 0, &ids, false), Action::None);
    assert_eq!(policy::action(true, 3, 1, &ids, false), Action::Launch(7));
    assert_eq!(policy::action(true, 3, 3, &ids, false), Action::None);
    assert_eq!(
        policy::action(true, 3, policy::NEW, &ids, false),
        Action::Copy(90)
    );
    assert_eq!(
        policy::action(true, 4, policy::NEW, &ids, false),
        Action::None
    );
    assert_eq!(
        policy::action(false, 3, policy::NEW, &ids, false),
        Action::None
    );
    assert_eq!(policy::action(false, 3, 7, &ids, false), Action::Settings);
    assert_eq!(policy::action(false, 3, 8, &ids, false), Action::Manage);
    assert_eq!(policy::action(false, 3, 9, &ids, false), Action::None);
    assert_eq!(policy::action(false, 3, 9, &ids, true), Action::Retry);
    assert_eq!(&policy::user_name(1)[..6], b"User 1");
    assert_eq!(&policy::user_name(3)[..6], b"User 3");
}

#[test]
fn close_requires_successful_selection_of_the_exact_observed_ticket() {
    let entries = [
        [0, 701, 90, 1, 0, 0, 0, 0],
        [0, 702, 90, 0, 0, 0, 0, 0],
        [0; 8],
    ];
    let viewport = view(1200, 96);
    assert_eq!(
        policy::task_click(&entries, viewport, pointer(400 + 144 + 124, 32), true, 400),
        Some((702, true))
    );
    assert_eq!(
        policy::task_click(&entries, viewport, pointer(400 + 144 + 123, 32), true, 400),
        Some((702, false))
    );
    assert_eq!(
        policy::task_click(&entries, viewport, pointer(400 + 288, 32), true, 400),
        Some((0, false))
    );
    assert_eq!(
        policy::task_click(&entries, viewport, pointer(400 + 432, 32), true, 400),
        None
    );
    assert_eq!(
        policy::task_click(&entries, viewport, pointer(400, 64), true, 400),
        None
    );
    assert_eq!(
        policy::after_select((-16i64) as u64, true),
        (false, "TRY AGAIN")
    );
    assert_eq!(policy::after_select(0, true), (true, ""));
    assert_eq!(policy::after_select(0, false), (false, ""));
    assert_eq!(
        policy::outcome(Operation::Close, [(-16i64) as u64, 0, 0, 0, 0, 0, 0, 0]).0,
        "UNSAVED WORK"
    );
}

#[test]
fn launch_observation_and_failed_copies_do_not_claim_success() {
    assert_eq!(
        policy::outcome(Operation::Launch, [0, 1, 9001, 0, 0, 0, 0, 0]),
        ("", Some(9001))
    );
    for (error, message) in [(-16, "TRY AGAIN"), (-12, "NO ROOM"), (-13, "CANNOT OPEN")] {
        assert_eq!(
            policy::outcome(Operation::Launch, [error as u64, 1, 9001, 0, 0, 0, 0, 0]),
            (message, None)
        );
    }
    assert_eq!(
        policy::outcome(Operation::Copy, [(-12i64) as u64, 0, 0, 0, 0, 0, 0, 0]).0,
        "NO ROOM"
    );
    assert_eq!(
        policy::outcome(Operation::Copy, [(-5i64) as u64, 0, 0, 0, 0, 0, 0, 0]).0,
        "TRY AGAIN"
    );
    assert_eq!(
        policy::observation([0, wire::LAUNCH_RUNNING, 0, 0, 0, 0, 0, 0]),
        (true, "")
    );
    assert_eq!(
        policy::observation([0, wire::LAUNCH_FAILED, 0, 0, 0, 0, 0, 0]),
        (false, "APP STOPPED")
    );
    assert_eq!(
        policy::observation([(-2i64) as u64, wire::LAUNCH_RUNNING, 0, 0, 0, 0, 0, 0]),
        (false, "")
    );
}

#[test]
fn packet_bounds_and_unknown_operations_are_contained() {
    #[repr(C)]
    struct Guarded {
        before: [u64; 8],
        packet: [u64; 128],
        after: [u64; 8],
    }
    let mut memory = Guarded {
        before: [0xfeed; 8],
        packet: [u64::MAX; 128],
        after: [0xdead; 8],
    };
    for op in 0..=11 {
        memory.packet.fill(u64::MAX);
        memory.packet[0] = op;
        let _ = super::decide(&mut memory.packet);
        assert_eq!(memory.before, [0xfeed; 8]);
        assert_eq!(memory.after, [0xdead; 8]);
    }
    memory.packet[0] = 99;
    assert_eq!(super::decide(&mut memory.packet), 0);
}
