//! Experimental Rust-lab ABI, not Cathedral's accepted component contract.
//! x86-64: INT 0x80; RAX is call/result, RDI/RSI/RDX are arguments. Entry receives
//! two ordinary arguments in RDI/RSI and a 16-byte-aligned stack without a caller.
//! AArch64: SVC #0; x8 is call, x0 is result, x0/x1/x2 are arguments. Entry
//! receives two ordinary arguments in x0/x1 with a 16-byte-aligned stack.
//! Below, arg0/arg1/arg2 refer to the selected architecture's argument registers.
//! Write accepts at most 256 bytes within one mapped user page.

pub const WRITE: u64 = 0;
pub const YIELD: u64 = 1;
pub const EXIT: u64 = 2;
pub const IPC_HANDLE: u64 = 3;
pub const IPC_SEND: u64 = 4;
pub const IPC_RECEIVE: u64 = 5;
pub const IPC_REVOKE: u64 = 6;
pub const TASK_LAUNCH: u64 = 7;
pub const TASK_SPAWN: u64 = 8;
pub const TASK_WAIT: u64 = 9;
pub const TASK_PORT: u64 = 10;
pub const TASK_CONNECT: u64 = 11;
/// `clock handle, address, 8` writes the monotonic tick; `24` writes the tick,
/// the kernel's microsecond clock behind it and this task's measured execution
/// microseconds, one read for a trace stamp that lines up across tasks finer
/// than a tick. Both need the clock grant.
pub const CLOCK_READ: u64 = 12;
pub const TASK_CANCEL: u64 = 13;
pub const TASK_WAIT_UNTIL: u64 = 14;
pub const CLOCK_HANDLE: u64 = 15;
pub const DISPLAY_INFO: u64 = 16;
/// Raw controller read: low eight bits are data; AUX identifies the mouse port.
pub const KEYBOARD_AUX: u64 = 512;
/// Loss is a separate record, never a tagged data byte.
pub const KEYBOARD_LOST: u64 = 256;
pub const KEYBOARD_READ: u64 = 17;
pub const KEYBOARD_WRITE: u64 = 18;
/// Receive into exactly MAX_MESSAGE writable bytes; arg2 is the absolute deadline.
pub const IPC_RECEIVE_UNTIL: u64 = 19;
/// arg0 is an absolute deadline. Requires both keyboard and clock grants.
pub const KEYBOARD_READ_UNTIL: u64 = 20;
pub const LINK_HANDLE: u64 = 21;
pub const LINK_CONNECT: u64 = 22;
pub const IPC_WAIT_TWO: u64 = 23;
pub const TIMED_OUT: u64 = (-110i64) as u64;
/// Maximum unambiguous forward distance in the wrapping tick clock.
pub const MAX_INTERVAL: u64 = (1 << 63) - 1;
pub const BUSY: u64 = (-16i64) as u64;
pub const NO_MEMORY: u64 = (-12i64) as u64;
/// Resource growth cannot transfer the requested unused descendant slots.
/// Includes slots below the child's ordering boundary; no account IDs disclosed.
pub const NO_PROCESS_CAPACITY: u64 = (-201i64) as u64;
pub const BAD_EXECUTABLE: u64 = (-8i64) as u64;
pub const EXIT_BYTES: usize = 40;
pub const EXIT_RETURNED: u64 = 0;
pub const EXIT_FAULT: u64 = 1;
pub const EXIT_CANCELLED: u64 = 2;
pub const MAX_MESSAGE: usize = 64;
pub const BAD_HANDLE: u64 = (-9i64) as u64;
pub const WOULD_BLOCK: u64 = (-11i64) as u64;
pub const DENIED: u64 = (-13i64) as u64;
pub const PEER_CLOSED: u64 = (-32i64) as u64;
pub const TOO_SMALL: u64 = (-90i64) as u64;
pub const REVOKED: u64 = (-125i64) as u64;
pub const MAX_WRITE: usize = 256;
pub const IO_ERROR: u64 = (-5i64) as u64;
pub const BAD_ADDRESS: u64 = (-14i64) as u64;
pub const INVALID_ARGUMENT: u64 = (-22i64) as u64;
pub const UNKNOWN: u64 = (-38i64) as u64;

/// Bounded lab arenas. Task slots follow composition; IPC reserves one fixed arena.
pub const MAX_TASKS: usize = 31;
// Core account masks use u32; reserve a bit so full-mask construction cannot overflow.
const _: () = assert!(MAX_TASKS < u32::BITS as usize);
pub const MAX_LAUNCHES: usize = MAX_TASKS - 1;
pub const MAX_ENDPOINTS: usize = 64;
/// Maximum LBA28 sectors per bootstrap PIO command. Transfers remain 512 bytes.
pub const DISK_BATCH_SECTORS: usize = 32;
pub const DISK_REGISTER_READ: u64 = 24;
pub const DISK_REGISTER_WRITE: u64 = 25;
pub const DISK_TRANSFER: u64 = 26;
pub const MEMORY_ALLOCATE: u64 = 27;
pub const MEMORY_ADDRESS: u64 = 28;
pub const MEMORY_RELEASE: u64 = 29;
pub const MEMORY_SEAL: u64 = 30;
pub const MEMORY_MAP: u64 = 31;
pub const MEMORY_PAGES: u64 = 32;

/// Wait on 1..=MAX_WAIT_HANDLES receive handles copied from a user array; return ready index.
pub const IPC_WAIT_MANY: u64 = 33;
pub const MAX_WAIT_HANDLES: usize = 32;

/// Ordinary processes may create bounded, one-way copied-message channels.
/// CREATE writes [send, receive] to 16 writable bytes at arg0; arg1/arg2 must be zero.
pub const IPC_CREATE: u64 = 34;
pub const IPC_CLOSE: u64 = 35;
/// SEND_CAP: arg0 destination, arg1 packet address, arg2 zero. Moves one dynamic
/// endpoint on success. Packet: length u64, attachment handle u64, 64 payload bytes.
pub const IPC_SEND_CAP: u64 = 36;
/// RECEIVE_CAP: arg0 receiver, arg1 writable packet, arg2 zero. Blocks like RECEIVE.
pub const IPC_RECEIVE_CAP: u64 = 37;
pub const IPC_PACKET_BYTES: usize = 80;
pub const MAX_DYNAMIC_CHANNELS: usize = 192;
pub const MAX_CREATED_CHANNELS: usize = 188;
pub const MAX_DYNAMIC_HANDLES: usize = 48;

/// Move up to five dynamic endpoints into a newly admitted child. arg0 launch,
/// arg1 56-byte record (argument, count, five handles), arg2 zero. Unused words zero.
pub const TASK_SPAWN_WITH: u64 = 38;
/// Copy the caller's initial endpoint names: count, five handles (48 bytes).
/// arg0 writable destination, arg1/arg2 zero. This snapshot never reinstalls grants.
pub const TASK_ENVIRONMENT: u64 = 39;
pub const MAX_INITIAL_ENDPOINTS: usize = 5;
pub const SPAWN_BYTES: usize = 16 + MAX_INITIAL_ENDPOINTS * 8;
pub const ENVIRONMENT_BYTES: usize = 8 + MAX_INITIAL_ENDPOINTS * 8;

/// SPAWN_WITH's prefix followed by count and MAX_SPAWN_LOANS grants.
/// Successful admission lends idle launch/peer authority until the child exits.
/// Endpoints move as usual. arg0 launch, arg1 SPAWN_TREE_BYTES-byte record, arg2 zero; tails zero.
pub const TASK_SPAWN_TREE: u64 = 40;
/// Fixed wire capacity: saved executables use this 176-byte request. Increasing
/// the kernel's task arena must not change how much user memory this call reads.
/// This bounds explicit launch loans per spawn, not dynamic descendant budgets.
pub const MAX_SPAWN_LOANS: usize = 14;
pub const SPAWN_TREE_BYTES: usize = SPAWN_BYTES + 8 + MAX_SPAWN_LOANS * 8;
const _: () = assert!(MAX_SPAWN_LOANS <= MAX_LAUNCHES);

/// Caller-local IPC account snapshot at arg0: channel reservation, charged channels,
/// dynamic-handle limit, charged handles (including escrow). arg1/arg2 must be zero.
pub const IPC_BUDGET: u64 = 41;
pub const IPC_BUDGET_BYTES: usize = 32;
/// Stable per-account default; larger compositions must explicitly subdivide
/// the bounded backing pool.
pub const DEFAULT_CHANNELS: usize = 4;
/// Lab CPU burst allowances refill at allowance/period of one execution CPU.
pub const CPU_REFILL_PERIOD_US: u64 = 100_000;
/// Caller-only measured execution microseconds. Same grant/copy ABI as CLOCK_READ.
pub const CPU_READ: u64 = 42;
/// Own account: private pages, reader pages, channels, then corresponding usage.
/// arg0 points to 48 writable bytes; arg1/arg2 zero. No foreign/global selector.
pub const RESOURCE_BUDGET: u64 = 43;
/// arg0 owned idle launch; arg1 points to three u64 absolute child limits; arg2 zero.
/// Unused capacity moves between owner and child atomically, never minted.
pub const RESOURCE_ASSIGN: u64 = 44;
pub const TASK_EXECUTABLE: u64 = 45; // owned launch -> executable-only authority
pub const TASK_RESERVE: u64 = 46; // executable, 48-byte quotas/slots/load ceiling/physical frames, zero
pub const TASK_RELEASE: u64 = 47; // idle dynamic launch, zero, zero
pub const TASK_POOL_ASSIGN: u64 = 48; // owned idle launch, available child-slot count, zero
pub const TASK_LAUNCH_PORT: u64 = 49; // owned launch -> owner-local peer port
pub const TASK_IMAGE_AT: u64 = 50; // local executable grant index, zero, zero
pub const TASK_POOL_AVAILABLE: u64 = 51; // zero, zero, zero; own unused process reservations
pub const MAX_ADMISSION_FRAMES: usize = 288;

pub const TASK_IMAGE_ALLOW: u64 = 52; // owned idle launch, executable grant, zero; stage copied startup authority
pub const TASK_CPU_ASSIGN: u64 = 53; // owned idle launch, burst/refill ceiling, zero; retains debt
pub const PHYSICAL_BUDGET: u64 = 54; // writable [reserved frames, charged frames], zero, zero
pub const PHYSICAL_ASSIGN: u64 = 55; // owned idle launch, physical frames, zero

/// Sealed-memory capabilities: same 80-byte packet as endpoint IPC, separate typed receive.
pub const IPC_SEND_MEMORY: u64 = 56;
pub const IPC_RECEIVE_MEMORY: u64 = 57;
pub const IPC_TRY_RECEIVE_MEMORY: u64 = 58;
pub const MEMORY_FREEZE: u64 = 59; // private handle -> immutable; no peer binding
pub const MEMORY_MAP_OBJECT: u64 = 60; // local sealed handle -> local address
pub const MEMORY_UNMAP_OBJECT: u64 = 61; // retain handle, remove only own mapping

// Caller-local non-DMA register/IRQ grants. Only boot-selected child grants exist.
pub const DEVICE_HANDLE: u64 = 62; // local index, zero, zero -> handle
pub const DEVICE_READ8: u64 = 63; // handle, register offset, zero -> byte
pub const DEVICE_WRITE8: u64 = 64; // handle, register offset, byte
pub const DEVICE_IRQ_WAIT: u64 = 65; // handle, block (0/1), zero -> notification
pub const DEVICE_IRQ_ARM: u64 = 66; // handle, zero, zero; BUSY until notification consumed
pub const DEVICE_RELEASE: u64 = 67; // handle, zero, zero; reset/revoke until next launch
/// address of count little-endian [kind, local handle] records, count (0..=8), deadline.
/// Requires a clock grant. Returns first ready index, without consuming; validates
/// the entire set first. Empty sets wait only for the deadline. IPC closure is ready.
pub const EVENT_WAIT: u64 = 68;
/// Sealed object handle, first entry argument, zero -> caller-local executable
/// grant backed by that object, consumed by one reservation and admitted once.
pub const TASK_IMAGE_FROM_OBJECT: u64 = 69;
/// Exclusive bootstrap input grant: command, data, writable 16-byte reply.
/// Fixed virtual pointer transport, not arbitrary I/O or hypervisor access.
pub const INPUT_VIRTUAL_POINTER: u64 = 70;
/// Owned launch, pointer to five u64 increments, zero: private pages, readers,
/// channels, unused descendant slots, physical frames. Atomic conserving growth,
/// including live children. Does not change mapping/handle maxima or CPU limits.
pub const RESOURCE_GROW: u64 = 71;
/// Drop an unused caller-local object executable grant; consumed grants are invalid.
pub const TASK_IMAGE_RELEASE: u64 = 72;
pub const WAIT_IPC: u64 = 0;
pub const WAIT_DEVICE: u64 = 1;

// Notification facility: typed caller-local rights, independent of payload IPC.
pub const NOTIFY_CREATE: u64 = 73; // writable [wait, signal, control], zero, zero
pub const NOTIFY_DROP: u64 = 74; // local grant, zero, zero; not object closure
pub const NOTIFY_SIGNAL: u64 = 75; // signal grant, broadcast (0/1), zero
pub const NOTIFY_WAIT: u64 = 76; // wait grant, deadline, timed (0/1)
// 32-byte [signal grant, wait grant, deadline, timed], zero, zero.
// Negative result: signal did not commit. Nonnegative: signal committed, wait outcome.
pub const NOTIFY_SIGNAL_WAIT: u64 = 77;
pub const NOTIFY_CONTROL: u64 = 78; // control grant, action below, zero
// Transitional checked grant handoff through existing endpoints, not payload transport.
// SEND copies one right into recipient escrow, retaining the source; 80-byte packet ABI.
pub const NOTIFY_SEND: u64 = 79; // send endpoint, packet address, zero
pub const NOTIFY_RECEIVE: u64 = 80; // receive endpoint, writable packet, zero
pub const NOTIFY_BUDGET: u64 = 81; // writable [shared object limit, notifications charged, handle limit, handles used]
pub const NOTIFY_HANDLES: usize = 16;
pub const NOTIFY_SIGNALED: u64 = 0;
pub const NOTIFY_TIMED_OUT: u64 = 1;
pub const NOTIFY_CLOSED: u64 = 2;
pub const NOTIFY_REVOKED: u64 = 3;
pub const NOTIFY_CANCELLED: u64 = 4;
pub const NOTIFY_CLOSE: u64 = 0;
pub const NOTIFY_WITHDRAW_WAITS: u64 = 1; // revoke all wait grants; cannot be undone
pub const NOTIFY_CANCEL_WAITS: u64 = 2; // cancel currently registered waits only
