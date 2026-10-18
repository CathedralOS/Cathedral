//! Distribution-private control between the startup owner and its recovery UI.
//! HEALTH consumes a pending retry in word 1. STATUS changes presentation and
//! discards input queued for the previous state; it grants no capabilities.
pub const STATUS: u64 = 0xff01;
pub const STARTING: u64 = 0;
pub const FAILED: u64 = 1;
