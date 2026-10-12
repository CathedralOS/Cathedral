//! Exhaustive bring-up exercises; never linked into an ordinary boot.
#[cfg(feature = "bundled-user")]
mod applications;
#[cfg(feature = "bundled-user")]
mod display;
#[cfg(feature = "bundled-user")]
mod ipc;
mod kernel_tasks;
#[cfg(feature = "bundled-user")]
mod memory;
#[cfg(feature = "bundled-user")]
mod rendering;
#[cfg(feature = "bundled-user")]
mod supervision;
mod user_tasks;

pub fn run(boot: &mut crate::handoff::BootState) {
    kernel_tasks::exercise(&mut boot.memory, &mut boot.console);
    user_tasks::exercise(&mut boot.memory, &mut boot.console);
    #[cfg(feature = "bundled-user")]
    {
        applications::exercise(&mut boot.memory, &mut boot.console);
        ipc::exercise(&mut boot.memory, &mut boot.console);
        supervision::exercise(&mut boot.memory, &mut boot.console);
        memory::exercise(&mut boot.memory, &mut boot.console);
        rendering::exercise(&mut boot.memory, &mut boot.console);
        display::exercise(&mut boot.memory, &mut boot.console);
    }
}
