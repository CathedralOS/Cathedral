use super::*;

pub(super) fn exercise(memory: &mut PreparedMemory, console: &mut SerialPort) {
    let baseline = memory.frames.allocated();
    // SAFETY: Sole boot CPU, outside task execution.
    let heap = unsafe { HEAP.used() };
    for revoke in [0, 1] {
        // SAFETY: Boot-owned executables and endpoints; initialized memory/entry paths.
        let reports = unsafe {
            users::run_configured(
                &mut memory.frames,
                &memory.layout,
                memory.image,
                &[program(9, revoke), program(10, revoke)],
                user_output,
                Config {
                    frame_limit: usize::MAX,
                    supervision: &[],
                    clock_readers: &[0],
                    endpoints: &[endpoint(1, 0, Some(1)), endpoint(0, 1, None)],
                },
            )
        }
        .unwrap();
        assert!(
            reports
                .iter()
                .all(|report| report.exit == Some(Exit::Returned(0)))
        );
        assert_eq!(reports[0].wait_timeouts, 2);
        assert_eq!(reports[0].receives_blocked, 1);
        assert_eq!(reports[0].ipc_received, 1);
        drop(reports);
        reclaimed(memory, baseline, heap);
    }
    let reports = run(
        memory,
        &[program(11, 0), program(3, 0)],
        &[endpoint(1, 0, None)],
    );
    assert!(
        reports
            .iter()
            .all(|report| report.exit == Some(Exit::Returned(0)))
    );
    drop(reports);
    reclaimed(memory, baseline, heap);
    writeln!(console, "Cathedral Rust lab: IPC deadlines woke idle sessions; checked copies, late replies, terminal precedence and denied clock access passed").ok();
}
