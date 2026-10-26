//! Test-only failures selected by composition features, absent from ordinary F5.
use super::{Error, Stage, abi};
pub(super) fn refusal(argument: u64) -> Result<(), Error> {
    // Refuse after channels/funding are staged but before any endpoint move.
    if argument >> 56 == 4 {
        Err(Error(abi::NO_MEMORY as i64))
    } else {
        Ok(())
    }
}
pub(super) fn argument(stage: Stage, attempt: u64, normal: u64) -> u64 {
    let selected = match stage {
        Stage::Display => cfg!(feature = "bootstrap-display"),
        Stage::Input => cfg!(feature = "bootstrap-input"),
        Stage::Root => cfg!(feature = "bootstrap-root"),
    };
    if !selected || (attempt != 0 && !cfg!(feature = "bootstrap-repeat")) {
        return normal;
    }
    let mode = if cfg!(feature = "bootstrap-refusal") {
        4
    } else if cfg!(feature = "bootstrap-bad-ready") {
        5
    } else if cfg!(feature = "bootstrap-malformed") {
        3
    } else if cfg!(feature = "bootstrap-stall") {
        2
    } else {
        1
    };
    normal | (mode << 56)
}
