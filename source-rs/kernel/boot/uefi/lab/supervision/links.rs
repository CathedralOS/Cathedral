use super::*;
pub(super) fn exercise(memory: &mut PreparedMemory, console: &mut SerialPort) {
    let baseline = memory.frames.allocated();
    // SAFETY: Boot context outside task execution.
    let heap = unsafe { HEAP.used() };
    let grants = [22, 22, 23, 22].map(|role| Supervision {
        disk: false,
        owner: 0,
        peer: 0,
        program: program(role, 0),
        frame_limit: usize::MAX,
        framebuffer: None,
        keyboard: false,
    });
    let links = [1, 2, 4].map(|service| cathedral_core::link::LinkSpec { client: 3, service });
    // SAFETY: Boot-owned ELF, fixed endpoint graph and initialized memory/entry paths.
    let reports = unsafe {
        users::run_configured(
            &mut memory.frames,
            &memory.layout,
            memory.image,
            &[program(21, 0)],
            user_output,
            Config {
                links: &links,
                frame_limit: usize::MAX,
                endpoints: &[],
                clock_readers: &[1, 2, 3, 4],
                supervision: &grants,
            },
        )
    }
    .unwrap();
    assert_eq!(reports[0].exit, Some(Exit::Returned(0)));
    assert_eq!(reports[0].spawned, 20);
    assert_eq!(reports[0].reaped, 20);
    assert!(
        reports[1..]
            .iter()
            .all(|report| report.exit == Some(Exit::Cancelled))
    );
    drop(reports);
    reclaimed(memory, baseline, heap);
    writeln!(console, "Cathedral Rust lab: peer graph survived 16 app/provider replacements; all endpoints, frames and heap reclaimed").ok();
}
