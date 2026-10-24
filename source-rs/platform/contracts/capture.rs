//! Experimental capture endpoint, separately declared and supplied. The provider
//! selects one source/crop/mask grant from the accepted endpoint incarnation;
//! packets cannot choose a PID, sibling, source handle or unmasked backing.
//! One pending request/result per endpoint. All unused words are zero.
//! INFO -> [0,width,height,stride,0..]. BEGIN -> [0,ticket,0..].
//! TAKE(ticket) -> PENDING, EXPIRED, refusal, or [0,ticket,width,height,stride,0..]
//! with one sealed, already-masked RGBX object. Successful transfer consumes the
//! retained result. A full receive queue retains the same result for retry.
//! CANCEL(ticket) releases pending work/result. Ticket IDs are not authority.
pub const INFO: u64 = 0x43415000;
pub const BEGIN: u64 = 0x43415001;
pub const TAKE: u64 = 0x43415002;
pub const CANCEL: u64 = 0x43415003;
pub const PENDING: u64 = 0x43410001;
pub const EXPIRED: u64 = 0x43410002;
