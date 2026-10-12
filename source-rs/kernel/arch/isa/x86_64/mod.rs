//! Rust module plumbing. Start reading in `x86_64.rs`.

mod context;
mod cpu_clock;
pub use cpu_clock::{calibrate_cpu_clock, cpu_time_us};
mod interrupts;
mod memory;
mod stacks;
mod user;
pub use crate::isa::x86::{disable_interrupts, halt_forever};
pub use context::{Context, set_switch_handler, suspend};
pub use interrupts::{
    install_interrupts, last_irq_stack, probe_double_fault, probe_invalid_opcode, start_timer,
    test_breakpoint, ticks, wait_for_ticks,
};
pub use memory::{BootLayout, prepare_memory};
pub use stacks::{MAX_TASK_SLOTS, allocate_stack, release_stack, task_stack_range};
pub use user::{UserMappings, begin_user_session, end_user_session, select_user_root};

include!("x86_64.rs");

use crate::{MemoryError, StackRange};
