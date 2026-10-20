//! Rust wiring. Start at aarch64.rs.
mod context;
mod interrupts;
mod memory;
mod stacks;
mod user;
pub use context::{Context, service_pending_interrupts, set_switch_handler, suspend};
pub use interrupts::{
    calibrate_cpu_clock, cpu_time_us, install_interrupts, last_irq_stack, start_timer,
    test_breakpoint, ticks, wait_for_ticks,
};
pub use memory::{BootLayout, prepare_memory};
pub use stacks::{MAX_TASK_SLOTS, allocate_stack, release_stack, task_stack_range};
pub use user::{UserMappings, begin_user_session, end_user_session, select_user_root};
include!("aarch64.rs");
