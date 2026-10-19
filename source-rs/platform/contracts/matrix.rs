//! Experimental local-world protocol. Children can address only their own binding.
//! No ancestor coordinates, global IDs, sibling enumeration or depth on the wire.
pub type Packet = [u64; 8];
pub const VIEW: u64 = 1;
pub const NEXT: u64 = 2;
pub const PAINT: u64 = 3; // epoch, owned surface handle, local x,y,width,height,rgb
pub const ERASE: u64 = 4; // epoch, owned surface handle,0,0,0,0,0 (release)
/// Atomic local view plus one event; response tag includes pointer buttons and MORE.
pub const POLL: u64 = 5;
pub const MORE: u64 = 1 << 16;
pub fn poll_event(reply: Packet) -> Packet {
    [
        reply[0],
        reply[1],
        reply[5] & 255,
        reply[6],
        reply[7],
        (reply[5] >> 8) & 7,
        0,
        0,
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
pub const KEY_MASK: u64 = 0x1f; // Existing navigation-only grants.
pub const VALID_KEY_MASK: u64 = crate::input::KEY_MASK;
// Owner connection only. Child requests using these opcodes are uniformly denied.
pub const BIND: u64 = 100; // width,height,key_mask; physical/root adapter only
pub const PLACE: u64 = 101; // local child slot,x,y,width,height,key_mask
pub const REVOKE: u64 = 102; // local child slot
pub const POINTER_SOURCE: u64 = 103; // x,y,buttons; physical/root adapter only
pub const KEY_SOURCE: u64 = 104; // key,state; physical/root adapter only
pub const HEALTH: u64 = 105;
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
pub const MAX_SURFACES: usize = 16;
/// Owner-only conserving subdivision of the matrix's upstream allowance.
pub const SURFACE_LIMIT: u64 = 110; // child slot, allowance,0,0,0,0,0
/// POLL carries the surface allowance above the key mask in word four.
pub const SURFACE_BITS: u32 = 56;

/// Same surface geometry as PAINT; word seven is pixel stride, plus one sealed attachment.
/// RGBX little-endian u32 pixels; clipping retains the original row stride.
pub const SURFACE_IMAGE: u64 = 8;

/// [op, view epoch, nonzero revision, 0..]. Stage at most four distinct existing
/// surfaces with PAINT/ERASE/SURFACE_IMAGE; no publication until COMMIT succeeds.
/// A rejected stage poisons the batch. ABORT releases its readers. Repeating the
/// latest successful COMMIT is idempotent within this connection and view epoch.
pub const PRESENT_BEGIN: u64 = 9;
pub const PRESENT_COMMIT: u64 = 10;
pub const PRESENT_ABORT: u64 = 11;
pub const PRESENT_LIMIT: usize = 4;

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

/// Child-local storage binding: [INFO,0..] -> [0, binding revision, realm,0..].
/// Storage packets retain storage.rs's layout; opcode is
/// (binding revision << STORAGE_SCOPE_BITS) | (STORAGE_BASE + storage opcode).
/// No root selector crosses the child connection. A binding is a realm: the
/// subtree under the granted scope object, or the whole of what the matrix
/// itself holds when the grant names no scope. The matrix stamps a granted
/// scope into the request's free scope words, so the provider enforces the
/// entire chain of attenuation.
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
/// retiring its provider stream. The current 100-Hz timer makes 600 ticks
/// nominally six seconds, not 600 milliseconds. Nested forwards and program
/// reads add a margin. This is a failure bound, not a commit latency estimate.
pub const STORAGE_DEADLINE_TICKS: u64 = 600;

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
