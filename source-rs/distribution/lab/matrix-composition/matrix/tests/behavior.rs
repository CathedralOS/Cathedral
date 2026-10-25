//! Execute Omega's policy with explicit host facts; no mirrored Rust state layout.
use super::code::Code;
use cathedral_kernel_contracts::user as abi;
use cathedral_omega_app::{Instance, Step};
use cathedral_platform_contracts::{imports, matrix as wire, services};
include!(env!("OMEGA_MATRIX_CONSTANTS"));
fn run(test: impl FnOnce(&mut Instance<'_>)) {
    let code = Code::new(BYTES);
    // SAFETY: The checked component is mapped executable for the whole test.
    let step: Step = unsafe { core::mem::transmute(code.address.add(ENTRY)) };
    let mut state = [0; 512];
    let mut packet = [0; 4096];
    // SAFETY: Exclusive disjoint buffers and validated live ABI-1 export.
    let mut app = unsafe { Instance::new(step, &mut state, &mut packet) }.unwrap();
    test(&mut app);
}
fn boot(app: &mut Instance<'_>) {
    assert_eq!(app.advance(), 13);
    assert_eq!(&app.packet[..4], &[1, 300, 600, wire::KEY_MASK]);
    for slot in 0..3 {
        assert_eq!(app.advance(), 11);
        assert_eq!(
            &app.packet[..3],
            &[
                slot,
                slot,
                if slot == 2 {
                    imports::ALL
                } else {
                    imports::INTERACTION
                }
            ]
        );
        app.packet[..8].fill(0);
        if slot == 0 {
            assert_eq!(app.advance(), 20);
            assert_eq!(app.packet[0], abi::BUSY);
        }
        assert_eq!(app.advance(), 14);
        assert_eq!(
            &app.packet[..8],
            &[
                wire::PLACE,
                slot,
                0,
                slot * 200,
                300,
                200,
                wire::KEY_MASK,
                0
            ]
        );
        app.packet[..8].fill(0);
    }
    assert_eq!(app.advance(), 20);
    assert_eq!(app.packet[0], abi::NO_MEMORY);
    assert_eq!(app.advance(), 10);
}
fn request(app: &mut Instance<'_>, caller: u64, words: wire::Packet) {
    app.packet[..14].fill(0);
    app.packet[..8].copy_from_slice(&words);
    app.packet[10] = caller;
    app.packet[11] = 77;
    app.packet[12] = 1;
    app.packet[13] = 64;
}
#[test]
fn startup_refusal_stops_before_placement_or_later_admission() {
    run(|a| {
        assert_eq!(a.advance(), 13);
        assert_eq!(a.advance(), 11);
        a.packet[9] = abi::NO_MEMORY;
        assert_eq!(a.advance(), 0);
    });
}
#[test]
fn untrusted_words_cannot_choose_owner_dispatch() {
    run(|a| {
        boot(a);
        for caller in 0..3 {
            for op in [
                wire::SPAWN,
                wire::STOP,
                wire::PLACE,
                wire::KEY_SOURCE,
                (1u64 << 32) | wire::VIEW,
            ] {
                request(a, caller, [op, 2, 0, 0, 0, 0, 0, 0]);
                assert_eq!(a.advance(), 15); // Host routes with its retained endpoint identity.
                a.packet[..8].copy_from_slice(&wire::error(abi::DENIED));
                assert_eq!(a.advance(), 16);
                assert_eq!(a.packet[0], abi::DENIED);
                assert_eq!(a.advance(), 10);
            }
        }
        request(a, 2, services::request(imports::STORAGE, 1, 1, 0));
        assert_eq!(a.advance(), 17);
        assert_eq!(a.advance(), 10);
    });
}
#[test]
fn owner_replacement_preserves_service_policy_and_returns_admission_failure() {
    run(|a| {
        boot(a);
        for slot in 0..3 {
            for mode in 0..=2 {
                request(a, 3, [wire::SPAWN, slot, mode, 0, 0, 0, 0, 0]);
                assert_eq!(a.advance(), 11);
                assert_eq!(
                    &a.packet[..3],
                    &[
                        slot,
                        if mode == 0 { slot } else { mode + 2 },
                        if slot == 2 {
                            imports::ALL
                        } else {
                            imports::INTERACTION
                        }
                    ]
                );
                a.packet[9] = abi::BUSY;
                assert_eq!(a.advance(), 16);
                assert_eq!(&a.packet[..8], &wire::error(abi::BUSY));
                a.packet[9] = 0;
                assert_eq!(a.advance(), 10);
            }
        }
    });
}
#[test]
fn malformed_requests_and_memory_cannot_launch_children() {
    run(|a| {
        boot(a);
        for (kind, len, slot, mode) in [(1, 7, 0, 0), (3, 64, 0, 0), (1, 64, 3, 0), (1, 64, 0, 3)] {
            request(a, 3, [wire::SPAWN, slot, mode, 0, 0, 0, 0, 0]);
            a.packet[12] = kind;
            a.packet[13] = len;
            assert_eq!(a.advance(), 16);
            assert_eq!(a.packet[0], abi::DENIED);
            assert_eq!(a.advance(), 10);
        }
        a.packet[..14].fill(0);
        a.packet[12] = 2;
        a.packet[10] = 1;
        assert_eq!(a.advance(), 12);
        assert_eq!(a.packet[0], 1);
        assert_eq!(a.advance(), 10);
    });
}
