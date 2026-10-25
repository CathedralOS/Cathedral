// Scoped service acquisition. The authenticated endpoint selects the grant ceiling.
// Version 2 hands interaction a notification wait right; other parts move endpoints.
// Coordination is accepted in wiki/spec/communication/ipc.md, but has no wire
// operations in this version. These constants do not implement its protocol.
use crate::{imports, matrix::Packet};
pub const ACQUIRE: u64 = 0x53525601;
pub const VERSION: u64 = 2;
pub const UNAVAILABLE: u64 = 0x53520001;
pub const INCOMPATIBLE: u64 = 0x53520002;
pub const INVALID: u64 = 0x53520003;
pub const TAKEN: u64 = 0x53520004;

/// Select a preallocated right: interaction wait, storage send/receive,
/// input-producer send/receive, capture send/receive. Selection grants no scope.
/// No caller identity or desired scope comes from the request bytes.
pub fn select(granted: u64, words: Packet) -> Result<usize, u64> {
    let [ACQUIRE, version, contract, minimum, maximum, part, 0, 0] = words else {
        return Err(INVALID);
    };
    if minimum == 0 || minimum > maximum {
        return Err(INVALID);
    }
    if version != VERSION {
        return Err(INCOMPATIBLE);
    }
    if !matches!(
        contract,
        imports::INTERACTION | imports::STORAGE | imports::INPUT_PRODUCER | imports::CAPTURE
    ) || granted & contract == 0
    {
        return Err(UNAVAILABLE);
    }
    if !(minimum..=maximum).contains(&1) {
        return Err(INCOMPATIBLE);
    }
    match (contract, part) {
        (imports::INTERACTION, 0) => Ok(0),
        (imports::STORAGE, 0) => Ok(1),
        (imports::STORAGE, 1) => Ok(2),
        (imports::INPUT_PRODUCER, 0) => Ok(3),
        (imports::INPUT_PRODUCER, 1) => Ok(4),
        (imports::CAPTURE, 0) => Ok(5),
        (imports::CAPTURE, 1) => Ok(6),
        _ => Err(INVALID),
    }
}
pub fn request(contract: u64, minimum: u64, maximum: u64, part: u64) -> Packet {
    [ACQUIRE, VERSION, contract, minimum, maximum, part, 0, 0]
}
