//! Exhaustive bring-up exercises; never linked into an ordinary boot.
#[cfg(feature = "bundled-user")]
mod applications;
#[cfg(feature = "bundled-user")]
mod display;
#[cfg(feature = "bundled-user")]
mod ipc;
#[cfg(feature = "bundled-user")]
mod supervision;
mod task_lifecycle;
mod tasks;
mod users;
#[cfg(feature = "bundled-user")]
mod watchdog;

pub fn run(boot: &mut crate::handoff::BootState) {
    tasks::exercise(&mut boot.memory, &mut boot.console);
    task_lifecycle::exercise(&mut boot.memory, &mut boot.console);
    users::exercise(&mut boot.memory, &mut boot.console);
    #[cfg(feature = "bundled-user")]
    {
        applications::exercise(&mut boot.memory, &mut boot.console);
        ipc::exercise(&mut boot.memory, &mut boot.console);
        supervision::exercise(&mut boot.memory, &mut boot.console);
        watchdog::exercise(&mut boot.memory, &mut boot.console);
        display::exercise(&mut boot.memory, &mut boot.console);
    }
}
