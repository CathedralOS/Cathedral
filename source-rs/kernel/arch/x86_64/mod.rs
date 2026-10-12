//! Rust module plumbing. Start reading in `x86_64.rs`.

mod context;
mod cpu_clock;
pub use cpu_clock::{calibrate_cpu_clock, cpu_time_us};
mod disk;
mod interrupts;
mod keyboard;
pub use disk::{disk_read, disk_transfer, disk_write};
pub use keyboard::{keyboard_irq, keyboard_read, keyboard_write};
mod memory;
mod stacks;
mod user;
pub use crate::x86::{disable_interrupts, halt_forever, in8, out8, qemu_exit};
pub use context::{Context, SwitchCause, UserFault, probe_registers, set_switch_handler, suspend};
pub use interrupts::{
    Fault, install_interrupts, last_irq_stack, probe_double_fault, probe_invalid_opcode,
    start_timer, test_breakpoint, ticks, wait_for_ticks,
};
pub use memory::{BootLayout, MemoryError, StackRange, prepare_memory};
pub use stacks::{MAX_TASK_SLOTS, StackFrames, allocate_stack, release_stack, task_stack_range};
pub use user::memory::{
    USER_CODE, USER_DATA, USER_IMAGE_END, USER_STACK, USER_STACK_TOP, UserSegment, valid_segments,
};
pub use user::{
    ImageRange, UserSpace, begin_user_session, end_user_session, select_user_root, user_probe,
};

include!("x86_64.rs");
