mod timer;
pub(super) use timer::acknowledge;
pub use timer::{
    calibrate_cpu_clock, cpu_time_us, last_irq_stack, start_timer, ticks, wait_for_ticks,
};
include!("interrupts.rs");
