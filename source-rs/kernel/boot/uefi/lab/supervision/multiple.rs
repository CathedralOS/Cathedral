//! Runtime integration: independently restart siblings, then reclaim the session.
use super::*;

pub(super) fn exercise(memory: &mut PreparedMemory, console: &mut SerialPort) {
    let baseline = memory.frames.allocated();
    // SAFETY: Boot owns the heap outside any task session.
    let heap = unsafe { HEAP.used() };
    for (role, budget, keyboard, spawned) in [
        (16, usize::MAX, false, 18),
        (18, 0, false, 1),
        (19, usize::MAX, true, 2),
    ] {
        let grants = [
            Supervision {
                owner: 0,
                peer: 0,
                program: program(17, 0),
                frame_limit: usize::MAX,
                framebuffer: None,
                keyboard: false,
            },
            Supervision {
                owner: 0,
                peer: 0,
                program: program(if keyboard { 20 } else { 17 }, 0),
                frame_limit: budget,
                framebuffer: None,
                keyboard,
            },
        ];
        // SAFETY: Sole boot CPU, IRQs off, valid image and exclusive device grant.
        let reports = unsafe {
            users::run_configured(
                &mut memory.frames,
                &memory.layout,
                memory.image,
                &[program(role, 0)],
                user_output,
                Config {
                    frame_limit: usize::MAX,
                    endpoints: &[],
                    clock_readers: &[],
                    supervision: &grants,
                },
            )
        }
        .unwrap();
        assert_eq!(reports[0].exit, Some(Exit::Returned(0)));
        assert_eq!(reports[0].spawned, spawned);
        assert_eq!(reports[0].reaped, spawned);
        assert_eq!(reports[0].cancelled, spawned);
        if role == 19 {
            // Parent close wakes the echo reader with PEER_CLOSED before it is
            // cancelled. The raw keyboard reader remains blocked until cancellation.
            assert!(reports[0].cancelled_blocked >= 1);
            assert_eq!(reports[2].keyboard_reads_blocked, 1);
        }
        assert_eq!(reports[1].exit, Some(Exit::Cancelled));
        assert_eq!(
            reports[2].exit,
            if role == 18 {
                None
            } else {
                Some(Exit::Cancelled)
            }
        );
        drop(reports);
        reclaimed(memory, baseline, heap);
    }
    let duplicate = [true, true].map(|keyboard| Supervision {
        owner: 0,
        peer: 0,
        program: program(20, 0),
        frame_limit: usize::MAX,
        framebuffer: None,
        keyboard,
    });
    // SAFETY: No execution is expected: duplicate exclusive grants must fail preflight.
    let rejected = unsafe {
        users::run_configured(
            &mut memory.frames,
            &memory.layout,
            memory.image,
            &[program(19, 0)],
            user_output,
            Config {
                frame_limit: usize::MAX,
                endpoints: &[],
                clock_readers: &[],
                supervision: &duplicate,
            },
        )
    };
    assert!(matches!(rejected, Err(users::Error::InvalidEndpoints)));
    drop(rejected);
    reclaimed(memory, baseline, heap);
    writeln!(console, "Cathedral Rust lab: multiple launch grants preserved sibling IPC across 16 restarts; failed admission and keyboard-wait cancellation reclaimed all memory").ok();
}
