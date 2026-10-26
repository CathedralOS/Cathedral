// Compare real preemptions with one and three busy descendants of the same owner.
use super::*;

pub(super) fn exercise(memory: &mut PreparedMemory, console: &mut SerialPort) {
    let baseline = memory.frames.allocated();
    // SAFETY: No task session is active; boot owns allocator access.
    let heap = unsafe { HEAP.used() };
    for descendants in [0, 2] {
        let launches = [26, 27, 27].map(|role| Supervision {
            owner: 0,
            peer: 0,
            program: program(role, 0),
            frame_limit: usize::MAX,
            framebuffer: None,
            keyboard: false,
            disk: false,
            device: None,
        });
        // SAFETY: Boot-owned ELFs, exclusive CPU/kernel root, initialized timer/entry paths.
        let reports = unsafe {
            user_tasks::run_configured(
                &mut memory.frames,
                &memory.layout,
                memory.image,
                &[program(24, descendants), program(25, 0)],
                user_output,
                Config {
                    executables: &[],
                    dynamic_tasks: 0,
                    // Owner start/sample gates, peer completion, branch readiness
                    // and two descendant starts. Workers only accept wait rights.
                    channel_limits: &[2, 1, 3, 0, 0],
                    cpu_limits: &[],
                    memory: &[],
                    frame_limit: usize::MAX,
                    links: &[],
                    clock_readers: &[1],
                    supervision: &launches,
                    endpoints: &[endpoint(0, 1), endpoint(1, 0)],
                },
            )
        }
        .unwrap();
        assert!(
            reports[..2]
                .iter()
                .all(|r| r.exit == Some(Exit::Returned(0)))
        );
        assert_eq!(reports[2].exit, Some(Exit::Cancelled));
        let outside = reports[1].preemptions;
        let branch: [usize; 3] = core::array::from_fn(|i| reports[i + 2].preemptions);
        let subtree = branch.iter().sum();
        writeln!(
            console,
            "Cathedral CPU: descendants={descendants} outside={outside} subtree={branch:?}"
        )
        .ok();
        assert!(outside >= 48);
        assert!(outside.abs_diff(subtree) <= 12); // Startup/teardown can cross a timer tick.
        assert!(branch[..descendants as usize + 1].iter().all(|&n| n >= 16));
        assert_eq!(reports[0].spawned, 1);
        assert_eq!(reports[2].spawned, descendants as usize);
        drop(reports);
        reclaimed(memory, baseline, heap);
    }
    writeln!(console, "Cathedral Rust lab: hierarchical CPU turns preserved outside progress under subtree growth; loan return and full reclamation passed").ok();
    budget::exercise(memory, console);
    timing::exercise(memory, console);
}
