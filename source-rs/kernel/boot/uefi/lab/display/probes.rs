use super::*;
pub(super) fn exercise(memory: &mut PreparedMemory, console: &mut SerialPort) {
    let frames = memory.frames.allocated();
    // SAFETY: No active user session.
    let heap = unsafe { HEAP.used() };
    let reports = run(memory, 4, usize::MAX);
    assert!(
        reports[..2]
            .iter()
            .all(|report| report.exit == Some(Exit::Returned(0)))
    );
    assert_eq!(reports[0].reaped, 4);
    assert!(
        matches!(reports[2].exit, Some(Exit::Fault(fault)) if fault.vector == 14 && fault.error == 4
        && fault.address == USER_ADDRESS + memory.framebuffer.unwrap().bytes)
    );
    let count = reports[2].frames;
    drop(reports);
    reclaimed(memory, frames, heap);
    // Every admission allocation, including the device leaf tables, must roll
    // back on failure. Repeat each attempt while an unrelated peer stays alive.
    for budget in 0..count {
        let reports = run(memory, 5, budget);
        assert!(
            reports[..2]
                .iter()
                .all(|report| report.exit == Some(Exit::Returned(0)))
        );
        assert_eq!(reports[0].spawned, 0);
        assert_eq!(reports[2].exit, None);
        drop(reports);
        reclaimed(memory, frames, heap);
    }
    writeln!(console, "Cathedral Rust lab: framebuffer NX guards and checked copies passed; {count} admission failure boundaries reclaimed all task memory").ok();
}
fn run(memory: &mut PreparedMemory, role: u64, budget: usize) -> alloc::vec::Vec<users::Report> {
    // SAFETY: Sole boot CPU, IRQs off, reserved exclusive device and owned images.
    unsafe {
        users::run_configured(
            &mut memory.frames,
            &memory.layout,
            memory.image,
            &[program(role), program(6)],
            user_output,
            Config {
                links: &[],
                frame_limit: usize::MAX,
                endpoints: &[EndpointSpec {
                    sender: 0,
                    receiver: 1,
                    revoker: None,
                }],
                clock_readers: &[],
                supervision: &[Supervision {
                    keyboard: false,
                    owner: 0,
                    peer: 1,
                    framebuffer: memory.framebuffer,
                    program: Program {
                        executable: Executable::Elf(SERVICE),
                        arguments: [u64::MAX, 0],
                    },
                    frame_limit: budget,
                }],
            },
        )
    }
    .unwrap()
}
fn reclaimed(memory: &PreparedMemory, frames: usize, heap: usize) {
    assert_eq!(memory.frames.allocated(), frames);
    // SAFETY: Session and report vectors were dropped.
    assert_eq!(unsafe { HEAP.used() }, heap);
}
