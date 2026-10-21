// Exhaustive bring-up exercises; never linked into an ordinary boot.

pub fn run(boot: &mut crate::handoff::BootState) {
    kernel_tasks::exercise(&mut boot.memory, &mut boot.console);
    startup::exercise(&mut boot.memory, &mut boot.console);
    user_tasks::exercise(&mut boot.memory, &mut boot.console);
    #[cfg(feature = "bundled-user")]
    {
        devices::exercise(&mut boot.memory, &mut boot.console);
        shared_memory::exercise(&mut boot.memory, &mut boot.console);
        applications::exercise(&mut boot.memory, &mut boot.console);
        ipc::exercise(&mut boot.memory, &mut boot.console);
        waits::exercise(&mut boot.memory);
        supervision::exercise(&mut boot.memory, &mut boot.console);
        memory::exercise(&mut boot.memory, &mut boot.console);
        rendering::exercise(&mut boot.memory, &mut boot.console);
        display::exercise(&mut boot.memory, &mut boot.console);
    }
}
