//! Rust wiring. Start at aarch64.rs.
mod context;
mod interrupts;
mod legacy;
mod memory;
mod user;
pub use crate::image::{UserSegment, valid_segments};
pub use context::{Context, SwitchCause, UserFault, set_switch_handler, suspend};
pub use interrupts::{
    Fault, calibrate_cpu_clock, cpu_time_us, install_interrupts, last_irq_stack, start_timer,
    test_breakpoint, ticks, wait_for_ticks,
};
pub use legacy::{
    disk_read, disk_transfer, disk_write, keyboard_irq, keyboard_read, keyboard_write,
};
pub use memory::{BootLayout, MemoryError, StackFrames, prepare_memory};
pub use user::{
    ImageRange, USER_CODE, USER_IMAGE_END, USER_STACK, USER_STACK_TOP, UserSpace,
    begin_user_session, end_user_session, select_user_root,
};
include!("aarch64.rs");
