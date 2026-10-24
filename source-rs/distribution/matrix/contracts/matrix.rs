//! Desktop layout, configured launcher and its recovery policy.
pub use cathedral_platform_contracts::matrix::*;
#[path = "config.rs"]
pub mod config;

/// Reader pages a chooser is lent: it maps one listing page of its realm at a
/// time while naming tiles or users, and nothing else.
pub const CHOOSER_READERS: u64 = 2;

/// The entry argument a stock matrix gives a program it runs from its realm:
/// the program's own object number above a role byte, so the program knows
/// which entry it is without reading anything. Roles: 0 an ordinary program or
/// the configured launcher, 1 the user picker, 2 the picker after a failed
/// selection. The argument carries no authority; the realm grant does.
pub const LAUNCH_ROLE_BITS: u32 = 8;
pub const fn launch_argument(role: u64, object: u64) -> u64 {
    (object << LAUNCH_ROLE_BITS) | (role & 0xff)
}
pub const fn launch_role(argument: u64) -> u64 {
    argument & 0xff
}
pub const fn launch_object(argument: u64) -> u64 {
    argument >> LAUNCH_ROLE_BITS
}

/// Child interaction request: [APPLY_LAYOUT, current epoch, margin, strip width,
/// gap, 0, 0, 0]. Only the launcher explicitly selected by stock CONFIGURE may
/// request this. Changes geometry, never budgets, services, keys or executable
/// selection. Saving configuration and applying it are separate operations.
/// A retry with the same geometry is idempotent after refreshing the local view.
pub const APPLY_LAYOUT: u64 = 16;

/// Only the configured fallback's current interaction endpoint may request this.
/// [RETRY_LAUNCHER, 0..] acknowledges one attempt before that fallback is retired.
/// No automatic retry is replenished; app state and placement remain intact.
pub const RETRY_LAUNCHER: u64 = 20;

/// Ask this stock matrix to load its own Matrix configuration. Already configured
/// returns success without rearming recovery; other occupied arrangements refuse.
/// Arms one local retry of a faulted/nonzero-exit launcher under unchanged
/// authority. Explicit stop/owner policy changes cancel it; app state survives.
pub const CONFIGURE: u64 = 122;

/// Owner-only: [LAUNCHER_FALLBACK, executable index + 1, argument, 0..]. Zero
/// removes the policy. The first nonzero supply can precede CONFIGURE or follow
/// realm startup while the launcher is running, preserving its retry allowance.
/// Replacement/revocation cancels configured recovery, including queued retries.
/// An initial supply during pending recovery also cancels that context. After
/// launcher retry exhaustion, the fallback has local interaction and retry authority;
/// no storage, layout, task-management or descendant capacity. Template
/// admission retains its own code grant, with no additional executable grants.
pub const LAUNCHER_FALLBACK: u64 = 127;

/// Startup may read configuration and a program through several storage hops.
pub const CONFIGURE_DEADLINE_TICKS: u64 = 8 * STORAGE_DEADLINE_TICKS + 100;

// Owner-only routing. Three bits per hop (1..=5 = local child 0..=4); child endpoints
// reject the entire encoded opcode. No process IDs or global paths are accepted.
pub const OP_BITS: u32 = 16;
pub fn route(mut words: Packet, child: usize) -> Option<Packet> {
    if child > 4 || words[0] >> 61 != 0 {
        return None;
    }
    words[0] = (words[0] & 0xffff) | (((words[0] >> OP_BITS) * 8 + child as u64 + 1) << OP_BITS);
    Some(words)
}

/// Child requests on the interaction connection. LAUNCH: [LAUNCH, object, withhold, 0..]
/// asks the host to run an object from the requester's realm as its sibling,
/// in a free configured app slot (legacy lab: the other owner-opened slot). The
/// reply names the slot and local observation receipt. Configured desktops accept
/// this only from chrome. CLOSE: [CLOSE, 0..] asks to stop the selected app,
/// which it does only after a LOSS probe answers with nothing it would lose.
/// Repeated authorized launches may share a realm; no implicit clone or
/// singleton rule. A program's exclusive-instance protocol is not defined here.
/// See wiki/spec/execution/components_and_matrices.md#multiple-instances-over-one-realm.
pub const LAUNCH: u64 = 14;
/// Loading code, growing delegated funding and binding storage are sequential
/// operations. The caller must outwait their bounded work before retiring its
/// interaction connection; the ordinary short UI timeout is insufficient.
pub const LAUNCH_DEADLINE_TICKS: u64 = 8 * STORAGE_DEADLINE_TICKS + 100;
pub const CLOSE: u64 = 15;

/// Delegated interaction: observe save availability, or request one explicit retry.
/// No task/provider identifiers cross this interface. Ordinary apps receive neither.
pub const SAVE_AVAILABILITY: u64 = 21;
pub const RETRY_SAVES: u64 = 22;
pub const SAVES_READY: u64 = 0;
pub const SAVES_OFFLINE: u64 = 1;
pub const SAVES_RESTARTING: u64 = 2;
/// Owner: [SAVE_RECOVERY_GRANT, slot, enabled (0/1), 0..]. Bound to live interaction.
pub const SAVE_RECOVERY_GRANT: u64 = 128;
/// Root owner: publish state and consume one still-authorized queued retry.
/// [SAVE_RECOVERY_POLL, state, 0..] -> [0, requested (0/1), 0..].
pub const SAVE_RECOVERY_POLL: u64 = 129;

/// Configured chrome only: enumerate one of three local app positions.
/// [APP_ENTRY,index,0..] -> [0,opaque instance ticket,program object,selected,0..].
/// Empty positions return zero. No descendant/process identities are disclosed.
pub const APP_ENTRY: u64 = 23;
/// Configured chrome only: select an exact live instance, preserving other apps.
pub const SELECT_APP: u64 = 24;
