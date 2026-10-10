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
