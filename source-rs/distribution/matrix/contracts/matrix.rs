//! Desktop layout, configured launcher and its recovery policy.
pub use cathedral_platform_contracts::matrix::*;
#[path = "config.rs"]
pub mod config;

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

/// Ask this stock matrix to load its own Matrix configuration; refuses if running.
/// Arms one local retry of a faulted/nonzero-exit launcher under unchanged
/// authority. Explicit stop/owner policy changes cancel it; app state survives.
pub const CONFIGURE: u64 = 122;

/// Owner-only: [LAUNCHER_FALLBACK, executable index + 1, argument, 0..]. Zero
/// removes the policy. Set before CONFIGURE; changing it cancels any previously
/// configured recovery, including queued retries. CONFIGURE may use this code
/// after launcher retry exhaustion. The fallback has local interaction and retry authority;
/// no storage, layout, task-management or descendant capacity. Template
/// admission retains its own code grant, with no additional executable grants.
pub const LAUNCHER_FALLBACK: u64 = 127;

/// Startup may read configuration and a program through several storage hops.
pub const CONFIGURE_DEADLINE_TICKS: u64 = 8 * STORAGE_DEADLINE_TICKS + 100;

// Owner-only routing. Two bits per hop (1/2 = local child 0/1); child endpoints
// reject the entire encoded opcode. No process IDs or global paths are accepted.
pub const OP_BITS: u32 = 16;
pub fn route(mut words: Packet, child: usize) -> Option<Packet> {
    if child > 1 || words[0] >> 62 != 0 {
        return None;
    }
    words[0] = (words[0] & 0xffff) | (((words[0] >> OP_BITS) * 4 + child as u64 + 1) << OP_BITS);
    Some(words)
}

/// Child requests on the interaction connection. LAUNCH: [LAUNCH, object, withhold, 0..]
/// asks the host to run an object from the requester's realm as its sibling,
/// in the other slot, which the owner must have opened with LAUNCH_POLICY; the
/// reply names the slot and local observation receipt. CLOSE: [CLOSE, 0..] asks the host to stop the sibling,
/// which it does only after a LOSS probe answers with nothing it would lose.
pub const LAUNCH: u64 = 14;
pub const CLOSE: u64 = 15;
