//! Compiled Omega child-control routing, packet validation and exclusion precedence.
use crate::policy::control::{Facts, Operation as O, decide};
use cathedral_desktop_matrix_contracts::matrix as w;
use cathedral_kernel_contracts::{memory, user as abi};
fn owner() -> Facts {
    Facts {
        owner: true,
        ..Default::default()
    }
}
fn packet(op: u64) -> w::Packet {
    [op, 0, 0, 0, 0, 0, 0, 0]
}
#[test]
fn child_control_cannot_be_selected_without_an_authenticated_owner() {
    for op in 0..512 {
        let words = packet(op);
        for routed in [
            words,
            w::route(words, 0).unwrap(),
            w::route(words, 4).unwrap(),
        ] {
            assert_eq!(decide(routed, 5, Facts::default()).operation, O::Deny);
        }
    }
    for count in [0, 8, usize::MAX] {
        assert_eq!(decide(packet(w::STOP), count, owner()).operation, O::Deny);
    }
}
#[test]
fn loss_and_imports_share_an_opcode_but_not_a_packet_shape() {
    assert_eq!(w::LOSS, w::LAUNCH_IMPORTS);
    assert_eq!(decide(packet(w::LOSS), 5, owner()).operation, O::Survey);
    assert_eq!(
        decide(
            packet(w::LOSS),
            5,
            Facts {
                loss: true,
                ..owner()
            }
        )
        .operation,
        O::Survey
    );
    let words = [w::LAUNCH_IMPORTS, 1, 0, 0, 0, 0, 0, 0];
    assert_eq!(decide(words, 5, owner()).operation, O::Imports);
    assert_eq!(
        decide(
            words,
            5,
            Facts {
                loss: true,
                ..owner()
            }
        )
        .operation,
        O::Busy
    );
    // An unrelated owner operation must still reach its own handler during a survey.
    assert_eq!(
        decide(
            packet(w::PLACE),
            5,
            Facts {
                loss: true,
                ..owner()
            }
        )
        .operation,
        O::Unhandled
    );
}
#[test]
fn forwarding_preserves_one_pending_request_per_exact_local_hop() {
    for target in 0..5 {
        for op in [w::STOP, w::CONFIGURE, u16::MAX as u64] {
            let words = w::route(w::route(packet(op), 2).unwrap(), target).unwrap();
            let d = decide(words, 5, owner());
            assert_eq!(d.operation, O::Forward);
            assert_eq!(d.slot, target);
            assert_eq!(
                d.deadline,
                if op == w::CONFIGURE {
                    w::CONFIGURE_DEADLINE_TICKS
                } else {
                    100
                }
            );
            assert_eq!(
                decide(
                    words,
                    5,
                    Facts {
                        pending: 1 << target,
                        ..owner()
                    }
                )
                .operation,
                O::Busy
            );
            assert_eq!(
                decide(
                    words,
                    5,
                    Facts {
                        pending: 1 << ((target + 1) % 5),
                        ..owner()
                    }
                )
                .operation,
                O::Forward
            );
            assert_eq!(
                decide(
                    words,
                    5,
                    Facts {
                        loss: true,
                        ..owner()
                    }
                )
                .operation,
                O::Busy
            );
        }
    }
    for hop in [0, 6, 7] {
        let mut words = packet(w::STOP);
        words[0] |= ((8 + hop) as u64) << w::OP_BITS;
        assert_eq!(decide(words, 5, owner()).operation, O::Deny);
    }
}
#[test]
fn control_shapes_and_limits_are_bound_to_the_rust_contracts() {
    for (op, operation, reserved) in [
        (w::LAUNCH_IMAGES, O::Images, 3),
        (w::LAUNCHER_FALLBACK, O::Fallback, 3),
        (w::MANAGEMENT_GRANT, O::Management, 3),
        (w::CONFIGURE, O::Configure, 1),
        (w::RESTORE_STORAGE, O::Restore, 1),
        (w::RUN_PAGES, O::Pages, 4),
        (w::RUN_MEMORY, O::Memory, 3),
        (w::LAUNCH_PLACE, O::Placement, 7),
        (w::SPAWN, O::Spawn, 5),
        (w::RUN, O::Run, 8),
        (w::STOP, O::Stop, 2),
        (w::LAUNCH_POLICY, O::Policy, 8),
    ] {
        let words = packet(op);
        assert_eq!(
            decide(
                words,
                5,
                Facts {
                    standing: 1,
                    ..owner()
                }
            )
            .operation,
            operation,
            "op {op}"
        );
        for at in reserved..8 {
            let mut malformed = words;
            malformed[at] = 1;
            assert_eq!(
                decide(
                    malformed,
                    5,
                    Facts {
                        standing: 1,
                        ..owner()
                    }
                )
                .operation,
                O::Deny,
                "op {op}, word {at}"
            );
        }
    }
    for (op, at, limit) in [
        (w::LAUNCH_IMAGES, 2, (1 << abi::MAX_LAUNCHES) - 1),
        (w::MANAGEMENT_GRANT, 2, 5),
        (w::RUN_PAGES, 2, memory::MAX_PRIVATE_PAGES as u64),
        (w::RUN_PAGES, 3, memory::MAX_READER_PAGES as u64),
        (w::LAUNCH_POLICY, 2, memory::MAX_PRIVATE_PAGES as u64),
        (w::LAUNCH_POLICY, 3, memory::MAX_READER_PAGES as u64),
        (w::LAUNCH_POLICY, 7, w::MAX_SURFACES as u64),
    ] {
        let mut words = packet(op);
        words[at] = limit;
        assert_ne!(decide(words, 5, owner()).operation, O::Deny);
        words[at] += 1;
        assert_eq!(decide(words, 5, owner()).operation, O::Deny);
    }
    let words = [
        w::LAUNCH_IMPORTS,
        4,
        cathedral_platform_contracts::imports::SUPPORTED,
        0,
        0,
        0,
        0,
        0,
    ];
    assert_eq!(decide(words, 5, owner()).operation, O::Imports);
    let mut malformed = words;
    malformed[2] = u64::MAX;
    assert_eq!(decide(malformed, 5, owner()).operation, O::Deny);
}
#[test]
fn launch_refusals_and_policy_changes_preserve_their_original_order() {
    let mut words = [w::RUN_OBJECT, 4, 10, 20, 0, 0, 0, 0];
    assert_eq!(decide(words, 5, owner()).operation, O::Stored);
    assert_eq!(decide(words, 5, owner()).slot, 4);
    words[2] = 0;
    assert_eq!(decide(words, 5, owner()).operation, O::Invalid);
    for f in [
        Facts {
            launching: true,
            ..owner()
        },
        Facts {
            storing: true,
            ..owner()
        },
        Facts {
            occupied: 16,
            ..owner()
        },
        Facts {
            unavailable: 16,
            ..owner()
        },
    ] {
        assert_eq!(decide(words, 5, f).operation, O::Busy);
    }
    words[1] = 5;
    assert_eq!(
        decide(
            words,
            5,
            Facts {
                launching: true,
                ..owner()
            }
        )
        .operation,
        O::Deny
    );
    for op in [w::RUN_MEMORY, w::RUN_PAGES] {
        assert_eq!(
            decide(
                packet(op),
                5,
                Facts {
                    occupied: 1,
                    ..owner()
                }
            )
            .operation,
            O::Busy
        );
    }
    // Standing policy may change while a child exists; it is not an allocation.
    assert_eq!(
        decide(
            packet(w::LAUNCH_POLICY),
            5,
            Facts {
                occupied: 1,
                ..owner()
            }
        )
        .operation,
        O::Policy
    );
    assert_eq!(
        decide(packet(w::LAUNCH_PLACE), 5, owner()).operation,
        O::Deny
    );
    assert_eq!(
        decide(
            packet(w::LAUNCH_PLACE),
            5,
            Facts {
                standing: 1,
                ..owner()
            }
        )
        .operation,
        O::Placement
    );
}
#[test]
fn every_control_slot_is_bounded_before_it_can_select_host_storage() {
    for op in [
        w::SPAWN,
        w::STOP,
        w::RUN,
        w::RUN_MEMORY,
        w::RUN_PAGES,
        w::RUN_OBJECT,
        w::LAUNCH_IMAGES,
        w::LAUNCH_POLICY,
        w::LAUNCH_PLACE,
        w::LAUNCH_IMPORTS,
        w::MANAGEMENT_GRANT,
    ] {
        for slot in [5, 6, 7, 64, u64::MAX] {
            let mut words = packet(op);
            words[1] = slot;
            assert_eq!(
                decide(
                    words,
                    5,
                    Facts {
                        standing: u64::MAX,
                        ..owner()
                    }
                )
                .operation,
                O::Deny
            );
        }
    }
}
