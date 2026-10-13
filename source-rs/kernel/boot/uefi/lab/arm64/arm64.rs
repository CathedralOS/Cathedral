// Portability proof sequencing. Machine-specific assertions exercise the existing shared core.
#[cfg(feature = "bundled-user")]
pub fn run(boot: &mut crate::handoff::BootState) {
    use core::fmt::Write;
    super::kernel_tasks::exercise(&mut boot.memory, &mut boot.console);
    let memory = &mut boot.memory;
    let console = &mut boot.console;
    assert!(
        memory.framebuffer.is_none(),
        "ARM64 proof must remain headless"
    );
    let baseline = memory.frames.allocated();
    // SAFETY: Sole CPU outside a session, exclusive heap ownership.
    let heap = unsafe { crate::heap::HEAP.used() };
    super::shared_memory::exercise(memory, console);
    let admission = processes::exchange(memory, console, baseline, heap);
    processes::isolation(memory, console, baseline, heap);
    sharing::exercise(memory, console, baseline, heap);
    processes::rollback(memory, console, baseline, heap, admission);
    writeln!(
        console,
        "Cathedral Rust lab: ARM64 portability proof passed"
    )
    .ok();
}
#[cfg(not(feature = "bundled-user"))]
pub fn run(boot: &mut crate::handoff::BootState) {
    super::kernel_tasks::exercise(&mut boot.memory, &mut boot.console);
}
