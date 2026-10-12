//! Experimental Rust-lab ABI, not Cathedral's accepted component contract.
//! x86-64: INT 0x80; RAX is call/result, RDI/RSI/RDX are arguments. Entry receives
//! two ordinary arguments in RDI/RSI and a 16-byte-aligned stack without a caller.
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
/// Receive into exactly MAX_MESSAGE writable bytes; RDX is the absolute deadline.
pub const IPC_RECEIVE_UNTIL: u64 = 19;
/// RDI is an absolute deadline. Requires both keyboard and clock grants.
pub const KEYBOARD_READ_UNTIL: u64 = 20;
pub const LINK_HANDLE: u64 = 21;
pub const LINK_CONNECT: u64 = 22;
pub const IPC_WAIT_TWO: u64 = 23;
pub const TIMED_OUT: u64 = (-110i64) as u64;
/// Maximum unambiguous forward distance in the wrapping tick clock.
pub const MAX_INTERVAL: u64 = (1 << 63) - 1;
pub const BUSY: u64 = (-16i64) as u64;
pub const NO_MEMORY: u64 = (-12i64) as u64;
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
pub const MAX_TASKS: usize = 8;
pub const MAX_LAUNCHES: usize = MAX_TASKS - 1;
pub const MAX_ENDPOINTS: usize = 32;
pub const DISK_REGISTER_READ: u64 = 24;
pub const DISK_REGISTER_WRITE: u64 = 25;
pub const DISK_TRANSFER: u64 = 26;
pub const MEMORY_ALLOCATE: u64 = 27;
pub const MEMORY_ADDRESS: u64 = 28;
pub const MEMORY_RELEASE: u64 = 29;
pub const MEMORY_SEAL: u64 = 30;
pub const MEMORY_MAP: u64 = 31;
pub const MEMORY_PAGES: u64 = 32;

/// Wait on 1..=8 receive handles copied from a user array; return ready index.
pub const IPC_WAIT_MANY: u64 = 33;
pub const MAX_WAIT_HANDLES: usize = 8;

/// Ordinary processes may create bounded, one-way copied-message channels.
/// CREATE writes [send, receive] to 16 writable bytes at RDI; RSI/RDX must be zero.
pub const IPC_CREATE: u64 = 34;
pub const IPC_CLOSE: u64 = 35;
/// SEND_CAP: RDI destination, RSI packet address, RDX zero. Moves one dynamic
/// endpoint on success. Packet: length u64, attachment handle u64, 64 payload bytes.
pub const IPC_SEND_CAP: u64 = 36;
/// RECEIVE_CAP: RDI receiver, RSI writable packet, RDX zero. Blocks like RECEIVE.
pub const IPC_RECEIVE_CAP: u64 = 37;
pub const IPC_PACKET_BYTES: usize = 80;
pub const MAX_DYNAMIC_CHANNELS: usize = 32;
pub const MAX_CREATED_CHANNELS: usize = 16;
pub const MAX_DYNAMIC_HANDLES: usize = 16;

/// Move up to four dynamic endpoints into a newly admitted child. RDI launch,
/// RSI 48-byte record (argument, count, four handles), RDX zero. Unused words zero.
pub const TASK_SPAWN_WITH: u64 = 38;
/// Copy the caller's initial endpoint names: count, four handles (40 bytes).
/// RDI writable destination, RSI/RDX zero. This snapshot never reinstalls grants.
pub const TASK_ENVIRONMENT: u64 = 39;
pub const MAX_INITIAL_ENDPOINTS: usize = 4;
pub const SPAWN_BYTES: usize = 48;
pub const ENVIRONMENT_BYTES: usize = 40;

/// SPAWN_WITH's 48-byte prefix followed by count and seven launch grants.
/// Successful admission lends idle launch/peer authority until the child exits.
/// Endpoints move as usual. RDI launch, RSI 112-byte record, RDX zero; tails zero.
pub const TASK_SPAWN_TREE: u64 = 40;
pub const SPAWN_TREE_BYTES: usize = SPAWN_BYTES + 8 + MAX_LAUNCHES * 8;

/// Caller-local IPC account snapshot at RDI: channel reservation, charged channels,
/// dynamic-handle limit, charged handles (including escrow). RSI/RDX must be zero.
pub const IPC_BUDGET: u64 = 41;
pub const IPC_BUDGET_BYTES: usize = 32;
/// Conservative default fits every reserved task slot without overcommit.
pub const DEFAULT_CHANNELS: usize = MAX_DYNAMIC_CHANNELS / MAX_TASKS;
/// Lab CPU burst allowances refill at allowance/period of one execution CPU.
pub const CPU_REFILL_PERIOD_US: u64 = 100_000;
/// Caller-only measured execution microseconds. Same grant/copy ABI as CLOCK_READ.
pub const CPU_READ: u64 = 42;
/// Own account: private pages, reader pages, channels, then corresponding usage.
/// RDI points to 48 writable bytes; RSI/RDX zero. No foreign/global selector.
pub const RESOURCE_BUDGET: u64 = 43;
/// RDI owned idle launch; RSI points to three u64 absolute child limits; RDX zero.
/// Unused capacity moves between owner and child atomically, never minted.
pub const RESOURCE_ASSIGN: u64 = 44;
pub const TASK_EXECUTABLE: u64 = 45; // owned launch -> executable-only authority
pub const TASK_RESERVE: u64 = 46; // executable, 48-byte quotas/slots/load ceiling/physical frames, zero
pub const TASK_RELEASE: u64 = 47; // idle dynamic launch, zero, zero
pub const TASK_POOL_ASSIGN: u64 = 48; // owned idle launch, available child-slot count, zero
pub const TASK_LAUNCH_PORT: u64 = 49; // owned launch -> owner-local peer port
pub const TASK_IMAGE_AT: u64 = 50; // local executable grant index, zero, zero
pub const TASK_POOL_AVAILABLE: u64 = 51; // zero, zero, zero; own unused process reservations
pub const MAX_ADMISSION_FRAMES: usize = 256;

pub const TASK_IMAGE_ALLOW: u64 = 52; // owned idle launch, executable grant, zero; stage copied startup authority
pub const TASK_CPU_ASSIGN: u64 = 53; // owned idle launch, burst/refill ceiling, zero; retains debt
pub const PHYSICAL_BUDGET: u64 = 54; // writable [reserved frames, charged frames], zero, zero
pub const PHYSICAL_ASSIGN: u64 = 55; // owned idle launch, physical frames, zero
