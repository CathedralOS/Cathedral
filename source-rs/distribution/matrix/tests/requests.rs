//! Execute the compiled Omega router using Rust contract values and hostile packets.
use crate::policy::requests::{self as r, Caller as C, Operation as O, Request};
use cathedral_desktop_matrix_contracts::matrix as w;
use cathedral_platform_contracts::{services, storage};
fn request(caller: C, words: [u64; 8]) -> Request {
    Request {
        caller,
        words: Some(words),
        separate_producer: false,
        storage_stream: caller == C::Storage,
    }
}
fn packet(op: u64) -> [u64; 8] {
    [op, 0, 0, 0, 0, 0, 0, 0]
}
#[test]
fn connection_identity_is_not_selected_by_the_message() {
    for op in 0..512 {
        let words = [op, 1, 2, 3, 4, 5, 6, 7];
        for caller in [C::Child, C::Storage, C::Source, C::Producer, C::Unknown] {
            let r = request(caller, words);
            assert!(!matches!(
                r.route(),
                O::Owner
                    | O::Resolve
                    | O::Pixel
                    | O::RecoveryOwner
                    | O::StorageOwner
                    | O::LayoutOwner
            ));
            assert_eq!(r.owner_fallback(), O::Deny);
            assert_eq!(r.owner_effects(5, true), 0);
        }
        assert_eq!(request(C::Source, words).route(), O::Source);
        assert_eq!(request(C::Producer, words).route(), O::Producer);
    }
    // Even a packet containing a provider/owner operation cannot turn a catalog
    // connection into input or layout authority.
    for op in [w::PLACE, w::POINTER_SOURCE, w::VIEW, w::LAUNCH, w::FUND] {
        assert_eq!(request(C::Storage, packet(op)).route(), O::DenyNow);
    }
}
#[test]
fn acquisition_and_storage_have_distinct_connection_ceilings() {
    for caller in [C::Child, C::Source, C::Storage] {
        assert_eq!(
            request(caller, packet(services::ACQUIRE)).route(),
            O::Acquire
        );
    }
    assert_eq!(
        request(C::Producer, packet(services::ACQUIRE)).route(),
        O::Producer
    );
    assert_eq!(
        request(C::Owner, packet(services::ACQUIRE)).route(),
        O::Owner
    );
    assert_eq!(
        request(C::Unknown, packet(services::ACQUIRE)).route(),
        O::Deny
    );
    for op in [
        w::STORAGE_INFO,
        w::STORAGE_BASE,
        w::STORAGE_BASE + 255,
        (91 << 16) | (w::STORAGE_BASE + storage::READ),
    ] {
        assert_eq!(request(C::Child, packet(op)).route(), O::DenyNow);
        assert_eq!(request(C::Storage, packet(op)).route(), O::Storage);
        let mut invalid = request(C::Storage, packet(op));
        invalid.storage_stream = false;
        assert_eq!(invalid.route(), O::DenyNow);
    }
    for op in [w::STORAGE_BASE - 1, w::STORAGE_BASE + 256, u64::MAX] {
        assert_eq!(request(C::Storage, packet(op)).route(), O::DenyNow);
    }
}
#[test]
fn child_contract_operations_reach_their_owners() {
    for (op, expected) in [
        (w::LAUNCH, O::Launch),
        (w::CLOSE, O::Close),
        (w::FUND, O::Fund),
        (w::APP_ENTRY, O::AppEntry),
        (w::SELECT_APP, O::Select),
        (w::APPLY_LAYOUT, O::Layout),
        (w::LAUNCH_STATUS, O::Status),
        (w::SAVE_AVAILABILITY, O::SaveAvailability),
        (w::RETRY_SAVES, O::RetrySaves),
        (w::MANAGED_TASK, O::ManagedTask),
        (w::RETRY_LAUNCHER, O::RetryLauncher),
        (w::TERMINATE_TASK, O::Terminate),
        (w::PRESENT_BEGIN, O::Present),
        (w::PRESENT_COMMIT, O::Present),
        (w::PRESENT_ABORT, O::Present),
        (w::PRESENT_RECTS, O::Present),
        (w::SURFACE_CREATE, O::Surface),
        (w::PAINT, O::Surface),
        (w::ERASE, O::Surface),
        (w::SURFACE_IMAGE, O::Surface),
        (w::SURFACE_CURSOR, O::Surface),
        (w::POLL_TAGGED, O::World),
        (w::INPUT_DIRECTORY, O::World),
    ] {
        assert_eq!(request(C::Child, packet(op)).route(), expected, "op {op}");
    }
}
#[test]
fn malformed_app_operations_fall_to_the_local_validator() {
    for (op, reserved_from) in [
        (w::LAUNCH, 3),
        (w::CLOSE, 1),
        (w::FUND, 6),
        (w::APP_ENTRY, 2),
        (w::SELECT_APP, 2),
        (w::LAUNCH_STATUS, 2),
        (w::SAVE_AVAILABILITY, 1),
        (w::RETRY_SAVES, 1),
        (w::MANAGED_TASK, 1),
        (w::RETRY_LAUNCHER, 1),
        (w::TERMINATE_TASK, 2),
    ] {
        for index in reserved_from..8 {
            let mut words = packet(op);
            words[index] = u64::MAX;
            assert_eq!(
                request(C::Child, words).route(),
                O::World,
                "op {op}, reserved {index}"
            );
        }
        // Non-reserved arguments are preserved, including extremes; semantic
        // validators still decide whether they designate a live resource.
        let mut words = packet(op);
        words[1..reserved_from].fill(u64::MAX);
        assert_ne!(request(C::Child, words).route(), O::World);
    }
}
#[test]
fn malformed_packets_cannot_request_owner_or_attachment_work() {
    for caller in [
        C::Unknown,
        C::Owner,
        C::Source,
        C::Child,
        C::Storage,
        C::Producer,
    ] {
        let mut r = request(caller, packet(w::BIND));
        r.words = None;
        assert_eq!(
            r.route(),
            match caller {
                C::Producer => O::Producer,
                C::Storage => O::DenyNow,
                _ => O::Deny,
            }
        );
        assert_eq!(r.owner_fallback(), O::Deny);
        assert_eq!(r.owner_effects(5, true), 0);
        assert_eq!(r.attachment(), O::DenyNow);
    }
}
#[test]
fn owner_diagnostics_and_success_effects_preserve_failure_atomicity() {
    let mut path = packet(w::RESOLVE_PATH);
    path[1..5].fill(u64::MAX);
    assert_eq!(request(C::Owner, path).route(), O::Resolve);
    for index in 5..8 {
        let mut invalid = path;
        invalid[index] = 1;
        assert_eq!(request(C::Owner, invalid).route(), O::Owner);
    }
    let mut pixel = packet(w::READ_PIXEL);
    pixel[1..3].fill(u64::MAX);
    assert_eq!(request(C::Owner, pixel).route(), O::Pixel);
    for index in 3..8 {
        let mut invalid = pixel;
        invalid[index] = 1;
        assert_eq!(request(C::Owner, invalid).route(), O::Owner);
    }
    for (op, route, effects) in [
        (w::SAVE_RECOVERY_GRANT, O::RecoveryOwner, r::CHANGED),
        (w::SAVE_RECOVERY_POLL, O::RecoveryOwner, r::CHANGED),
        (
            w::STORAGE_GRANT,
            O::StorageOwner,
            r::CHANGED | r::REMEMBER_STORAGE,
        ),
        (
            w::STORAGE_REVOKE,
            O::StorageOwner,
            r::CHANGED | r::REMEMBER_STORAGE,
        ),
        (
            w::STORAGE_STATUS,
            O::StorageOwner,
            r::CHANGED | r::REMEMBER_STORAGE,
        ),
        (
            w::STORAGE_BINDING,
            O::StorageOwner,
            r::CHANGED | r::REMEMBER_STORAGE,
        ),
        (w::PLACE, O::LayoutOwner, r::CHANGED | r::REPLAY_SLOT),
        (
            w::REVOKE,
            O::LayoutOwner,
            r::CHANGED | r::CLEAR_SLOT | r::REPLAY_SLOT,
        ),
        (
            w::BIND,
            O::LayoutOwner,
            r::CHANGED | r::CLEAR_PAIR | r::REPLAY_ALL,
        ),
    ] {
        let r = request(C::Owner, packet(op));
        assert_eq!(r.owner_fallback(), route);
        assert_eq!(r.owner_effects(5, true), effects);
        assert_eq!(r.owner_effects(5, false), 0);
    }
    let mut revoke = packet(w::REVOKE);
    revoke[1] = 4;
    assert_ne!(
        request(C::Owner, revoke).owner_effects(5, true) & r::CLEAR_SLOT,
        0
    );
    revoke[1] = 5;
    assert_eq!(
        request(C::Owner, revoke).owner_effects(5, true) & r::CLEAR_SLOT,
        0
    );
}
#[test]
fn attachment_allowlist_preserves_scope_bits_without_granting_a_connection() {
    for op in 0..256 {
        let expected = if matches!(
            op,
            storage::PUBLISH
                | storage::CREATE_NAMED
                | storage::SET_METADATA
                | storage::LOOKUP
                | storage::FILTER
                | storage::COPY_TREE
                | storage::COPY
        ) {
            O::Storage
        } else {
            O::DenyNow
        };
        for scope in [0, 1, u64::MAX >> 16] {
            let words = packet((scope << 16) | (w::STORAGE_BASE + op));
            assert_eq!(request(C::Storage, words).attachment(), expected);
            for caller in [C::Unknown, C::Owner, C::Source, C::Child, C::Producer] {
                assert_eq!(request(caller, words).attachment(), O::DenyNow);
            }
            let mut unbound = request(C::Storage, words);
            unbound.storage_stream = false;
            assert_eq!(unbound.attachment(), O::DenyNow);
        }
    }
}
#[test]
fn separated_producer_cannot_bypass_its_dedicated_connection() {
    for op in 39..49 {
        let mut r = request(C::Child, packet(op));
        r.separate_producer = true;
        assert_eq!(r.route() == O::Deny, w::producer_operation(op));
        r.separate_producer = false;
        assert_ne!(r.route(), O::Deny);
    }
}
