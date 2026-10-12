#![no_std]

//! Machine boundary: isa/ selects CPU mechanisms, board/ selects concrete controllers.
//! Core owns frame lifetime and scheduling; firmware handling belongs to boot.
pub mod board;
mod events;
mod isa;
pub mod memory;
pub use events::{Fault, SwitchCause, UserFault};
pub use isa::selected::{
    BootLayout, CPU_NAME, Context, ELF_MACHINE, INTERRUPT_READY, MAX_TASK_SLOTS, UserMappings,
    allocate_stack, begin_user_session, calibrate_cpu_clock, cpu_time_us, disable_interrupts,
    end_user_session, enter_stack, halt_forever, install_interrupts, last_irq_stack,
    prepare_memory, release_stack, select_user_root, set_switch_handler, stack_pointer,
    start_timer, suspend, task_stack_range, test_breakpoint, ticks, wait_for_ticks,
    without_interrupts,
};
#[cfg(target_arch = "x86_64")]
pub use isa::selected::{probe_double_fault, probe_invalid_opcode};
pub use memory::{FrameSource, ImageRange, MemoryError, StackRange};
