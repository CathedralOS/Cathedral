//! Execute Omega's stock launch choices independently of Rust capability custody.
use crate::policy::admission::{self as p, Facts};
use cathedral_kernel_contracts::user as abi;
use cathedral_platform_contracts::imports;
fn ready() -> Facts {
    Facts {
        busy: false,
        desktop: true,
        chrome: true,
        requester: Some(0),
        occupied: 1,
        unavailable: 0,
        storing: false,
        receipts: true,
        tickets: true,
        standing: 31,
        binding: true,
    }
}
#[test]
fn desktop_selects_first_empty_app_position_and_never_chrome_or_background() {
    for occupied in 0..32 {
        let result = p::decide(
            1,
            0,
            Facts {
                occupied,
                ..ready()
            },
        );
        match (1..4).find(|s| occupied & (1 << s) == 0) {
            Some(slot) => {
                assert_eq!(result.status, 0);
                assert_eq!(result.slot, slot);
                assert!(!result.full);
            }
            None => {
                assert_eq!(result.status, abi::NO_MEMORY);
                assert!(result.full);
            }
        }
    }
    // A free record with an unavailable connection is still BUSY; do not skip
    // it and silently choose a different target than the owner's arrangement.
    assert_eq!(
        p::decide(
            1,
            0,
            Facts {
                unavailable: 2,
                ..ready()
            }
        )
        .status,
        abi::BUSY
    );
    assert_eq!(
        p::decide(
            1,
            0,
            Facts {
                chrome: false,
                ..ready()
            }
        )
        .status,
        abi::DENIED
    );
}
#[test]
fn owner_driven_pairs_choose_the_other_slot_without_desktop_privileges() {
    for requester in 0..2 {
        let result = p::decide(
            1,
            0,
            Facts {
                desktop: false,
                chrome: false,
                requester: Some(requester),
                occupied: 1 << requester,
                ..ready()
            },
        );
        assert_eq!(result.status, 0);
        assert_eq!(result.slot, 1 - requester);
    }
    for requester in [None, Some(2), Some(4), Some(usize::MAX)] {
        assert_eq!(
            p::decide(
                1,
                0,
                Facts {
                    desktop: false,
                    requester,
                    ..ready()
                }
            )
            .status,
            abi::DENIED
        );
    }
}
#[test]
fn refusal_precedence_does_not_turn_resource_pressure_into_an_authority_change() {
    assert_eq!(
        p::decide(
            0,
            u64::MAX,
            Facts {
                busy: true,
                chrome: false,
                ..ready()
            }
        )
        .status,
        abi::BUSY
    );
    assert_eq!(
        p::decide(
            0,
            u64::MAX,
            Facts {
                chrome: false,
                storing: true,
                ..ready()
            }
        )
        .status,
        abi::DENIED
    );
    let full = p::decide(
        0,
        u64::MAX,
        Facts {
            occupied: 14,
            storing: true,
            ..ready()
        },
    );
    assert_eq!(full.status, abi::NO_MEMORY);
    assert!(full.full);
    assert_eq!(
        p::decide(
            0,
            u64::MAX,
            Facts {
                storing: true,
                receipts: false,
                ..ready()
            }
        )
        .status,
        abi::BUSY
    );
    for facts in [
        Facts {
            receipts: false,
            binding: false,
            ..ready()
        },
        Facts {
            tickets: false,
            standing: 0,
            ..ready()
        },
    ] {
        let result = p::decide(0, u64::MAX, facts);
        assert_eq!(result.status, abi::NO_MEMORY);
        assert!(!result.full);
    }
    assert_eq!(
        p::decide(
            0,
            0,
            Facts {
                standing: 0,
                binding: false,
                ..ready()
            }
        )
        .status,
        abi::INVALID_ARGUMENT
    );
    assert_eq!(
        p::decide(
            1,
            0,
            Facts {
                standing: 0,
                ..ready()
            }
        )
        .status,
        abi::DENIED
    );
    assert_eq!(
        p::decide(
            1,
            0,
            Facts {
                binding: false,
                ..ready()
            }
        )
        .status,
        abi::DENIED
    );
    assert_eq!(
        p::decide(
            1,
            0,
            Facts {
                standing: 2,
                ..ready()
            }
        )
        .status,
        0
    );
}
#[test]
fn withheld_imports_must_be_supported_but_need_not_be_granted() {
    for bit in 0..64 {
        let withheld = 1 << bit;
        assert_eq!(
            p::decide(u64::MAX, withheld, ready()).status,
            if imports::SUPPORTED & withheld != 0 {
                0
            } else {
                abi::INVALID_ARGUMENT
            }
        );
    }
    assert_eq!(p::decide(1, imports::SUPPORTED, ready()).status, 0);
    assert_eq!(p::decide(0, 0, ready()).status, abi::INVALID_ARGUMENT);
}
#[test]
fn launch_roles_distinguish_picker_selection_child_placement_and_desktop_activation() {
    for login in [false, true] {
        for entered in [false, true] {
            for desktop in [false, true] {
                for requester in [None, Some(0), Some(1), Some(4), Some(usize::MAX)] {
                    for slot in 0..5 {
                        let roles = p::roles(login, entered, slot, desktop, requester);
                        assert_eq!(roles.selection, login && !entered && slot == 1);
                        assert_eq!(roles.child, requester.is_some());
                        assert_eq!(roles.desktop, desktop && requester == Some(0));
                    }
                }
            }
        }
    }
}
#[test]
fn owner_picker_recognition_requires_the_local_realm_pair() {
    for upstream in [false, true] {
        for slot in 0..5 {
            for scope in [0, 1, u64::MAX] {
                assert_eq!(
                    p::inspect_picker(upstream, slot, scope),
                    !upstream && slot == 0 && scope != 0
                );
            }
        }
    }
    assert!(p::matches_picker(Some(2), Some(3), 2, 3));
    for (login, users, object, scope) in [
        (None, Some(3), 2, 3),
        (Some(2), None, 2, 3),
        (Some(2), Some(3), 3, 2),
        (Some(0), Some(0), 0, 0),
    ] {
        assert!(!p::matches_picker(login, users, object, scope));
    }
}
