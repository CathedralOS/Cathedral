//! Experimental local-world protocol. Children can address only their own binding.
//! Ordinary interaction exposes no ancestor coordinates, global IDs, sibling
//! enumeration or depth. An explicit producer grant additionally exposes its
//! authorized input extent; it never makes the connection an owner endpoint.
pub type Packet = [u64; 8];
/// Child interaction request: five incremental amounts (private pages, reader
/// pages, channels, descendant slots, physical frames), then two zero words.
/// The authenticated immediate owner may grant or refuse, and may ask its owner
/// for its own shortfall. Success transfers all requested amounts atomically on
/// this edge. Earlier ancestor grants may remain reserved after downstream refusal.
/// No global/ancestor budget inspection or automatic retry after transport failure.
pub const FUND: u64 = 25;
pub const VIEW: u64 = 1;
pub const NEXT: u64 = 2;
pub const PAINT: u64 = 3; // epoch, owned surface handle, local x,y,width,height,rgb
pub const ERASE: u64 = 4; // epoch, owned surface handle,0,0,0,0,0 (release)
/// Atomic local view plus one event; response tag includes pointer buttons and MORE.
pub const POLL: u64 = 5;
/// Correlated snapshot request: [POLL_TAGGED, nonzero u32 id, 0..].
/// Reply retains POLL's payload. Word zero packs id in its high 32 bits and
/// signed i32 status in its low 32 bits. IDs are local to one connection,
/// never authority, and must not wrap or be reused on that connection.
pub const POLL_TAGGED: u64 = 26;
pub fn tag_poll(mut reply: Packet, id: u32) -> Packet {
    reply[0] = ((id as u64) << 32) | (reply[0] as u32 as u64);
    reply
}
pub fn untag_poll(mut reply: Packet, id: u32) -> Option<Packet> {
    if id == 0 || (reply[0] >> 32) != id as u64 {
        return None;
    }
    reply[0] = reply[0] as u32 as i32 as i64 as u64;
    Some(reply)
}
pub const MORE: u64 = 1 << 16;
/// Experimental multiplexed input: nonzero u32 names scoped to the supplying
/// interaction connection/view binding, not process IDs or injection authority.
/// Each event has its stream in NEXT word six / POLL word five's high 32 bits.
/// A stream name must not be reused within a binding. NONE and GROUP_RETIRED
/// have stream zero; GROUP_RETIRED still carries its former group name.
/// Four streams with sixteen ordinary queued events each are the current lab
/// limits; one extra slot reserves a keyboard-cancellation boundary.
/// NEXT word seven / POLL tag bits 17..31 carry the child-local focus group.
/// Channels may share a group while keeping independent buttons and capture.
/// Delivery preserves order within a group across its streams; independent
/// groups are fairly drained without a cross-group ordering promise. CANCEL clears
/// that receiver stream's held state. The sender suppresses residual holds until
/// release, so the receiver must not wait for a release after CANCEL. A changed
/// view epoch resets every stream binding. RETIRED ends one channel permanently;
/// GROUP_RETIRED follows the retirement of every channel in that group. Reclaim
/// receiver state on these boundaries, not by reinterpreting an old name.
/// This partial encoding has owner registration and explicit per-child producer
/// quotas, explicitly offered service acquisition and a scoped binding directory.
pub const INPUT_STREAMS: usize = 4;
pub const INPUT_GROUPS: usize = 4;
/// Existing authenticated single-source adapter; no ambient child authority.
pub const SINGLE_INPUT_STREAM: u32 = 1;
pub fn poll_event(reply: Packet) -> Packet {
    [
        reply[0],
        reply[1],
        reply[5] & 255,
        reply[6],
        reply[7],
        (reply[5] >> 8) & 7,
        reply[5] >> 32,
        (reply[5] >> 17) & 0x7fff,
    ]
}
pub fn poll_view(mut reply: Packet) -> Packet {
    reply[4] &= VALID_KEY_MASK;
    reply[5..].fill(0);
    reply
}
/// Input-service adapter selector; the matrix itself derives its role from
/// the environment it is handed.
pub const SOURCE_MODE: u64 = 0x4d415452;
pub const READ_PIXEL: u64 = 108; // root owner diagnostic only: x,y
pub const NONE: u64 = 0;
pub const POINTER: u64 = 1;
pub const KEY: u64 = 2;
pub const CANCEL: u64 = 3;
/// End only this stream's keyboard interaction; pointer hold/capture survives.
pub const CANCEL_KEYS: u64 = 4;
/// Permanently ends this channel binding, including held keys/buttons. Pending
/// events for it are discarded. Receivers drop its local state, and must not
/// interpret a later channel as this one. Names remain non-reusable in a view.
pub const RETIRED: u64 = 5;
/// Ends a focus-group identity after all its channels have retired. Channel is
/// zero; the group field names the retired group. A replacement gets a fresh name.
pub const GROUP_RETIRED: u64 = 6;
/// Once before a described channel's first event on this edge: x/y contain two
/// little-endian label words, value contains controls (input::description).
/// Metadata is retained separately from ordinary queue pressure, and never
/// repeated on every event. It is not input, physical provenance or a credential.
/// Undescribed channels remain valid and have UNKNOWN directory descriptions.
pub const DESCRIBED: u64 = 7;
pub const KEY_MASK: u64 = 0x1f; // Existing navigation-only grants.
pub const VALID_KEY_MASK: u64 = crate::input::KEY_MASK;
// Owner connection only. Child requests using these opcodes are uniformly denied.
pub const BIND: u64 = 100; // width,height,key_mask; physical/root adapter only
pub const PLACE: u64 = 101; // local child slot,x,y,width,height,key_mask
pub const REVOKE: u64 = 102; // local child slot
pub const POINTER_SOURCE: u64 = 103; // x,y,buttons; physical/root adapter only
pub const KEY_SOURCE: u64 = 104; // key,state; physical/root adapter only
pub const HEALTH: u64 = 105;
/// Provider-owner input lifecycle. These commands require the existing owner
/// control connection; ordinary interaction/broker connections cannot use them.
/// CREATE replies [0, fresh local name, 0..]. Names never grant authority.
pub const INPUT_GROUP_CREATE: u64 = 130; // all zero
pub const INPUT_GROUP_RELEASE: u64 = 131; // group,0..; BUSY while channels retain it
pub const INPUT_CHANNEL_CREATE: u64 = 132; // retained group,0..
pub const INPUT_CHANNEL_RETIRE: u64 = 133; // channel,0..; ends held/queued input
pub const INPUT_POINTER: u64 = 134; // registered channel,x,y,buttons,0..
pub const INPUT_KEY: u64 = 135; // registered channel,key,state,0..
pub const INPUT_SELECT: u64 = 136; // local group, child slot + 1 (zero clears),0..
/// Explicit owner grant to an already connected child: slot, channel quota,
/// rights, reserved inherited channels, reserved inherited groups, 0, 0.
/// Reserves the quota immediately and supplies one private focus group. This is
/// a bounded delegation primitive, not manifest-based service acquisition.
pub const INPUT_PRODUCER_GRANT: u64 = 137;
pub const INPUT_PRODUCER_REVOKE: u64 = 138; // slot,0..
/// Allows this producer's pointer presses to select within its private group.
/// It grants no control of other groups, stacking, or sibling enumeration.
pub const PRODUCER_FOCUS: u64 = 1;
/// Granted child connection only. INFO returns status, grant generation,
/// permitted input width,height,key mask,channel quota,rights,0. Channel numbers
/// are 1..quota within that grant, unrelated to owner/recipient channel names.
/// Regrant changes the generation; geometry/revoke/disconnect ends this grant.
pub const PRODUCER_INFO: u64 = 40; // all zero
pub const PRODUCER_POINTER: u64 = 41; // generation,channel,x,y,buttons,0,0
pub const PRODUCER_KEY: u64 = 42; // generation,channel,key,state,0,0,0
pub const PRODUCER_RELEASE: u64 = 43; // generation,0..
/// Child sets its connection's maximum live input channel/group bindings:
/// [INPUT_CAPACITY, current view epoch, channels, groups, 0..]. Success echoes
/// status,epoch,channels,groups,0..; BUSY leaves the old limits when retained
/// aliases (including unread retirement boundaries) exceed the request.
/// Limits survive view changes on this connection, but not connection replacement.
/// New deliveries exceeding them refuse without mutating focus/held state.
pub const INPUT_CAPACITY: u64 = 44;
/// Read the caller's admitted input bindings, in increasing local-name order:
/// [INPUT_DIRECTORY, view epoch, revision or 0 to start, after name, 0..].
/// Start requires after=0. Reply: status,epoch,revision,name,group,state,keys,0.
/// Name=0 ends enumeration; its trailing fields are zero. A changed revision
/// returns BUSY: discard the partial snapshot and restart. Reading neither
/// consumes input nor registers/submits it. Associated metadata exposes only
/// channels already routed into this child's scope, never a global registry.
/// Revisions change on admission, retirement and consumption of RETIRED, with
/// the existing coalesced wake hint. CANCEL does not retire a channel. RETIRING
/// rows persist until the boundary drains; immutable old snapshots confer no
/// authority. View replacement resets the directory. Revision exhaustion makes
/// observation unavailable (NO_MEMORY) until a new view, never wraps an identity.
/// INPUT_DESCRIPTION reads the bounded immutable description at this revision.
/// Keys is the receiver's view-wide ceiling, not a source descriptor or promise
/// that this channel supplies those keys. Directory rows do not grant injection.
pub const INPUT_DIRECTORY: u64 = 46;
pub const CHANNEL_LIVE: u64 = 1;
pub const CHANNEL_RETIRING: u64 = 2;
/// Producer: generation, own channel ordinal, label word 0, label word 1,
/// controls,0,0. Set before any receiver binding exists; identical repeats are
/// harmless, changes or late first publication return BUSY. Labels grant nothing.
pub const PRODUCER_DESCRIBE: u64 = 47;
/// Receiver: view epoch, exact nonzero directory revision, local name,0.. .
/// Reply: status,epoch,revision,name,label word 0,label word 1,controls,0.
/// BUSY restarts the entire snapshot. A missing name is DENIED. Unknown is zero.
/// Retiring entries keep their description until RETIRED drains.
pub const INPUT_DESCRIPTION: u64 = 48;
pub fn producer_operation(op: u64) -> bool {
    (PRODUCER_INFO..=PRODUCER_RELEASE).contains(&op) || op == PRODUCER_DESCRIBE
}
pub const SPAWN: u64 = 106; // local child slot, current launch index, loan bitmask, argument
pub const STOP: u64 = 107; // local child slot; return after complete subtree reclamation
/// Host to child over its control link before a lifecycle decision: [LOSS, 0..].
/// The reply's word one is the work the child would lose
/// if stopped now, zero meaning none. A child that does not answer is kept.
/// A matrix asynchronously asks all its direct children and returns a boolean
/// aggregate. Unknown/busy/error answers count as loss; descendants are never
/// stopped by the query. This is an advisory observation, not a freeze/commit
/// transaction preventing subsequent edits or background work.
pub const LOSS: u64 = 121;
pub const MAX_EXTENT: u64 = 4096;
pub fn encode(words: Packet) -> [u8; 64] {
    let mut bytes = [0; 64];
    for (word, chunk) in words.iter().zip(bytes.chunks_exact_mut(8)) {
        chunk.copy_from_slice(&word.to_le_bytes());
    }
    bytes
}
pub fn decode(bytes: &[u8]) -> Option<Packet> {
    if bytes.len() != 64 {
        return None;
    }
    let mut words = [0; 8];
    for (word, chunk) in words.iter_mut().zip(bytes.chunks_exact(8)) {
        *word = u64::from_le_bytes(chunk.try_into().ok()?);
    }
    Some(words)
}
pub fn error(status: u64) -> Packet {
    [status, 0, 0, 0, 0, 0, 0, 0]
}

/// Owner selects a local executable and funds a dynamic child.
/// slot, image index, argument, channels, descendant slots, image mask, CPU ceiling.
pub const RUN: u64 = 109;
/// Owner-only physical allowance for the next RUN attempt on an empty local slot.
/// Consumed even on failure; omission uses 256 frames per requested process account.
pub const RUN_MEMORY: u64 = 111; // slot, frames,0,0,0,0,0

/// Child requests: create at a requested local stacking rank; query only own allowance.
pub const SURFACE_CREATE: u64 = 6; // epoch, rank,0,0,0,0,0; reply status, local handle
pub const SURFACE_INFO: u64 = 7; // all zero; reply status, allowance, allocated count
pub const MAX_SURFACES: usize = crate::display::composition::MATRIX_TILES;
/// Owner-only conserving subdivision of the matrix's upstream allowance.
pub const SURFACE_LIMIT: u64 = 110; // child slot, allowance,0,0,0,0,0
/// POLL carries the surface allowance above the key mask in word four.
pub const SURFACE_BITS: u32 = 56;

/// Same surface geometry as PAINT; word seven is pixel stride, plus one sealed attachment.
/// RGBX little-endian u32 pixels; clipping retains the original row stride.
pub const SURFACE_IMAGE: u64 = 8;
/// Bounded built-in arrow on an owned surface: epoch,name,x,y,w,h,rgb.
/// Dimensions clip the fixed cursor shape; they do not scale it. Ordinary
/// surface stacking/visibility apply. This grants no input or authenticated identity.
pub const SURFACE_CURSOR: u64 = 45;

/// [op, view epoch, nonzero revision, 0..]. Stage at most MAX_SURFACES distinct existing
/// surfaces with PAINT/ERASE/SURFACE_IMAGE/SURFACE_CURSOR; no publication until COMMIT succeeds.
/// A rejected stage poisons the batch. ABORT releases its readers. Repeating the
/// latest successful COMMIT is idempotent within this connection and view epoch.
pub const PRESENT_BEGIN: u64 = 9;
pub const PRESENT_COMMIT: u64 = 10;
pub const PRESENT_ABORT: u64 = 11;
pub const PRESENT_LIMIT: usize = MAX_SURFACES;

/// One atomic publication of one or two rectangles, using the same revision space
/// as BEGIN/COMMIT. Header: op/epoch/revision (24 bytes), then two 20-byte records:
/// local name u64, x/y/w/h u16 each, RGB u32; all little endian. An unused record
/// is entirely zero. No buffers, erases or allocation; names remain caller-local.
pub const PRESENT_RECTS: u64 = 13;

/// LAUNCH succeeds with [0, slot, receipt, 0..]. A receipt names only the
/// caller's latest successful launch, on that endpoint incarnation. Query with
/// [LAUNCH_STATUS, receipt, 0..] -> [0, state, 0..]. No PID, fault address, other
/// caller's activity or later occupant of the same slot is exposed.
pub const LAUNCH_STATUS: u64 = 17;
/// Explicitly delegated management, separate from launch observation. QUERY:
/// [MANAGED_TASK, 0..] -> [0, ticket, 0..], zero means the allowed slot is empty.
/// Tickets are local to the manager's endpoint and identify an exact task, never
/// a PID or a later occupant. [TERMINATE_TASK, ticket, 0..] cancels that subtree
/// without a LOSS survey. The grant authorizes this; human confirmation is the
/// chosen manager's responsibility. Missing/revoked grants and stale tickets deny.
pub const MANAGED_TASK: u64 = 18;
pub const TERMINATE_TASK: u64 = 19;
pub const LAUNCH_RUNNING: u64 = 1;
/// The task returned zero.
pub const LAUNCH_EXITED: u64 = 2;
/// The task faulted or returned nonzero; details stay with its owner.
pub const LAUNCH_FAILED: u64 = 3;
/// The owner cancelled the task.
pub const LAUNCH_STOPPED: u64 = 4;
pub fn rectangles(epoch: u64, revision: u64, updates: &[Packet]) -> Option<Packet> {
    if updates.is_empty() || updates.len() > 2 || epoch == 0 || revision == 0 {
        return None;
    }
    let (name, geometry, color) = rectangle_record(epoch, updates[0])?;
    let (second, second_geometry, second_color) = if updates.len() == 2 {
        rectangle_record(epoch, updates[1])?
    } else {
        (0, 0, 0)
    };
    if second == name {
        return None;
    }
    // Two 20-byte records cross the final three u64 boundaries. All names retain
    // 64 bits; these shifts implement the documented little-endian byte layout.
    Some([
        PRESENT_RECTS,
        epoch,
        revision,
        name,
        geometry,
        color | (second << 32),
        (second >> 32) | (second_geometry << 32),
        (second_geometry >> 32) | (second_color << 32),
    ])
}
fn rectangle_record(epoch: u64, p: Packet) -> Option<(u64, u64, u64)> {
    if p[0] != PAINT || p[1] != epoch || p[2] == 0 || p[7] > 0xff_ffff {
        return None;
    }
    let mut geometry = 0;
    for i in 0..4 {
        if p[3 + i] > u16::MAX as u64 {
            return None;
        }
        geometry |= p[3 + i] << (i * 16);
    }
    Some((p[2], geometry, p[7]))
}
fn rectangle_fields(epoch: u64, name: u64, geometry: u64, color: u64) -> Packet {
    [
        PAINT,
        epoch,
        name,
        geometry & 0xffff,
        (geometry >> 16) & 0xffff,
        (geometry >> 32) & 0xffff,
        geometry >> 48,
        color,
    ]
}
pub fn rectangle_updates(p: Packet) -> Option<([Packet; 2], usize)> {
    if p[0] != PRESENT_RECTS {
        return None;
    }
    let mut updates = [
        rectangle_fields(p[1], p[3], p[4], p[5] & 0xffff_ffff),
        rectangle_fields(
            p[1],
            (p[5] >> 32) | (p[6] << 32),
            (p[6] >> 32) | (p[7] << 32),
            p[7] >> 32,
        ),
    ];
    let count = if updates[1][2] == 0 { 1 } else { 2 };
    // Re-encoding checks ranges, distinct names, epoch/revision and zero padding.
    if rectangles(p[1], p[2], &updates[..count]) != Some(p) {
        return None;
    }
    if count == 1 {
        updates[1] = [0; 8];
    }
    Some((updates, count))
}

/// Child-local binding: [INFO,0,0,0,0,0,0,deadline] -> [0,revision,realm,0..].
/// Deadline zero requests the adapter's default; refreshes preserve an earlier bound.
/// Storage packets retain storage.rs's layout; opcode is
/// (binding revision << STORAGE_SCOPE_BITS) | (STORAGE_BASE + storage opcode).
/// No root selector crosses the child connection. A binding is a realm: the
/// subtree under the granted scope object, or the whole of what the matrix
/// itself holds when the grant names no scope. The matrix stamps a granted
/// scope into the request's free scope words. A full packet requires an
/// upstream containment proof before folding restrictions; see storage.rs and
/// the platform Scopes coordinator. Every data request preserves its absolute
/// deadline in word seven across all hops.
pub const STORAGE_INFO: u64 = 12;
pub const STORAGE_BASE: u64 = 256;
pub const STORAGE_SCOPE_BITS: u32 = 16;
/// Owner-only: [GRANT, child slot, available source slot, scope object, 0..],
/// [REVOKE,child,0..]. Sources are exclusive between siblings. Revoke aborts
/// pending transactions. A scope of zero grants everything the matrix holds.
pub const STORAGE_GRANT: u64 = 113;
pub const STORAGE_REVOKE: u64 = 114;
/// Root owner diagnostic only: retired provider-stream bitmask in reply word one.
/// Does not reconnect, grant authority or expose state to children.
pub const STORAGE_STATUS: u64 = 115;
pub const RUN_PAGES: u64 = 112; // owner: slot, private pages, reader pages; rest zero
/// Owner-only: run a program held in this matrix's realm. [RUN_OBJECT, slot,
/// object, scope, argument, channels, descendant slots, CPU ceiling]. The matrix
/// reads the object through its own binding, admits it with `argument` as its
/// second entry argument, and grants the child `scope` as its realm: the
/// object itself, so the program's state nests under its code, or zero for
/// everything the matrix holds. The root adapter reads from its first provider
/// root. RUN_PAGES and RUN_MEMORY stage the child's allowances exactly as for
/// RUN. Answered once the child is admitted, or with the provider's refusal.
pub const RUN_OBJECT: u64 = 117;
/// Owner-only standing policy for what a child's LAUNCH may put into a slot:
/// [LAUNCH_POLICY, slot, private pages, reader pages, channels, descendant
/// slots, CPU ceiling, surfaces], then [LAUNCH_PLACE, slot, x, y, width,
/// height, key mask, 0] for where it goes. A slot without a policy refuses
/// launches. A launched child is granted its object as its realm.
pub const LAUNCH_POLICY: u64 = 118;
pub const LAUNCH_PLACE: u64 = 119;
/// Owner-only: [STORAGE_BINDING, slot, 0..] -> [0, source, scope] of the
/// child's live storage binding, or DENIED while it has none.
pub const STORAGE_BINDING: u64 = 120;
/// Owner policy for subsequent store launches: [LAUNCH_IMPORTS, slot, offered
/// import mask, 0..]. Defaults to imports::ALL; cannot change an existing child.
/// LAUNCH's second payload word is a withholding mask, intersected with this
/// policy. Callers may remove offers, never add them. Undeclared imports are
/// not supplied; missing required/version-incompatible imports refuse launch.
pub const LAUNCH_IMPORTS: u64 = 121;
/// Monotonic ticks a matrix waits for one forwarded storage request before
/// retiring its provider stream. The current 100-Hz timer makes 1200 ticks
/// nominally twelve seconds. The larger scoped template can grow the disk extent
/// while copying; nested forwards share the client's absolute 2400-tick bound.
/// Program reads have a bounded multi-step workflow. This is a failure bound,
/// not a commit latency estimate.
pub const STORAGE_DEADLINE_TICKS: u64 = 1200;

/// Restore the storage grants this matrix recorded for its own children.
pub const RESTORE_STORAGE: u64 = 123;

/// Owner-only lookup: words 1..5 contain a zero-padded relative path (32 bytes).
pub const RESOLVE_PATH: u64 = 124;
/// Owner-only: [MANAGEMENT_GRANT, manager slot, target slot + 1, 0..]. Zero target
/// revokes. Requires a live manager and a different target slot. The grant is
/// tied to that connection incarnation; reconnecting does not inherit it.
pub const MANAGEMENT_GRANT: u64 = 125;
/// Owner-only: [LAUNCH_IMAGES, slot, executable mask, 0..]. Explicit executable
/// grants for future store-based admissions in this slot; default none. This
/// never grants code authority to a running child or changes its realm.
pub const LAUNCH_IMAGES: u64 = 126;

#[cfg(test)]
mod rectangle_tests {
    use super::*;
    #[test]
    fn rectangle_packets_preserve_local_names_and_reject_aliasing_or_truncation() {
        let a = [PAINT, 7, u64::MAX, 65535, 2, 0, 4096, 0xffffff];
        let b = [PAINT, 7, 0x123456789abcdef, 0, 1, 2, 3, 4];
        for updates in [&[a][..], &[a, b][..]] {
            let packed = rectangles(7, 9, updates).unwrap();
            let (decoded, count) = rectangle_updates(packed).unwrap();
            assert_eq!(&decoded[..count], updates);
        }
        let bytes = encode(rectangles(7, 9, &[a, b]).unwrap());
        assert_eq!(&bytes[44..52], &b[2].to_le_bytes());
        assert_eq!(&bytes[52..60], &[0, 0, 1, 0, 2, 0, 3, 0]);
        assert!(rectangles(7, 9, &[a, a]).is_none());
        let mut bad = a;
        bad[3] = 65536;
        assert!(rectangles(7, 9, &[bad]).is_none());
        let mut bad = rectangles(7, 9, &[a]).unwrap();
        bad[7] = 1;
        assert!(rectangle_updates(bad).is_none());
    }
}
