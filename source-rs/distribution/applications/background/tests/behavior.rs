use super::fixture::{BOOT, SURFACE, VIEW, with_app};
use cathedral_kernel_contracts::user as abi;
use cathedral_platform_contracts::{matrix as wire, services};

#[test]
fn denied_storage_leaves_interaction_usable_and_paints_the_original_view() {
    with_app(|peer| {
        peer.through(BOOT.len() - 1);
        // Both acquisitions and the diagnostic overwrite the response packet;
        // painting must retain the original view in Omega-owned session state.
        peer.reply([0; 8], 0);
        assert_eq!(peer.next(5)[0], wire::POLL);
    });
}

#[test]
fn unchanged_view_sleeps_and_new_epoch_reuses_the_surface_at_its_new_size() {
    with_app(|peer| {
        peer.through(6);
        for _ in 0..3 {
            peer.reply([0; 8], 0);
            assert_eq!(peer.next(5)[0], wire::POLL);
            peer.reply(VIEW, 0);
            peer.next(6);
        }
        peer.reply([0; 8], 0);
        peer.next(5);
        peer.reply([0, 8, 960, 540, 0, 0, 0, 0], 0);
        assert_eq!(
            peer.next(5),
            [wire::PAINT, 8, SURFACE, 0, 0, 960, 540, 0x101827]
        );
        peer.reply([0; 8], 0);
        peer.next(6);
    });
}

#[test]
fn unavailable_view_still_checks_scope_then_waits_for_placement() {
    with_app(|peer| {
        peer.through(1);
        peer.reply([abi::DENIED, 99, 1, 1, 0, 0, 0, 0], 0);
        peer.next(2);
        peer.reply([0; 8], services::UNAVAILABLE);
        peer.next(8);
        peer.reply([0; 8], 0);
        peer.next(6);
        peer.reply([0; 8], 0);
        peer.next(5);
        peer.reply(VIEW, 0);
        assert_eq!(peer.next(5), [wire::SURFACE_CREATE, 7, 0, 0, 0, 0, 0, 0]);
    });
}

#[test]
fn lifecycle_accepts_only_canonical_health_and_loss_requests() {
    with_app(|peer| {
        peer.through(6);
        for verb in [wire::HEALTH, wire::LOSS, wire::POLL, u64::MAX] {
            for extra in 0..8 {
                let mut request = [0; 8];
                request[0] = verb;
                if extra != 0 {
                    request[extra] = 1;
                }
                peer.reply(request, 1);
                let accepted = extra == 0 && matches!(verb, wire::HEALTH | wire::LOSS);
                assert_eq!(
                    peer.next(7),
                    [if accepted { 0 } else { abi::DENIED }, 0, 0, 0, 0, 0, 0, 0]
                );
                peer.reply([0; 8], 0);
                assert_eq!(peer.next(5)[0], wire::POLL);
                peer.reply(VIEW, 0);
                peer.next(6);
            }
        }
    });
}

#[test]
fn every_transport_failure_stops_without_replaying_work() {
    for step in 0..BOOT.len() {
        with_app(|peer| {
            peer.through(step);
            peer.reply([0; 8], abi::IO_ERROR);
            peer.failed(abi::IO_ERROR);
        });
    }
    with_app(|peer| {
        peer.through(6);
        peer.reply([wire::LOSS, 0, 0, 0, 0, 0, 0, 0], 1);
        peer.next(7);
        peer.reply([0; 8], abi::IO_ERROR);
        peer.failed(abi::IO_ERROR);
    });
}

#[test]
fn surface_and_paint_provider_failures_stop_without_retry() {
    for step in [4, 5] {
        with_app(|peer| {
            peer.through(step);
            peer.reply([abi::NO_MEMORY, 0, 0, 0, 0, 0, 0, 0], 0);
            peer.failed(abi::NO_MEMORY);
        });
    }
}

#[test]
fn unexpected_storage_authority_fails_without_reading_or_writing_storage() {
    with_app(|peer| {
        peer.through(2);
        peer.reply([0; 8], 0);
        peer.failed(abi::DENIED);
    });
}
