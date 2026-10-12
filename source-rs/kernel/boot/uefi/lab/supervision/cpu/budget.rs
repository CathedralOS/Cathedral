//! Measured ceilings under non-yielding work, yield storms, restart and all-idle refill.
use super::super::*;

pub(super) fn exercise(memory: &mut PreparedMemory, console: &mut SerialPort) {
    let baseline = memory.frames.allocated();
    // SAFETY: Exclusive boot context, no active session.
    let heap = unsafe { HEAP.used() };
    for argument in [0, 2, 258, u64::MAX] {
        let restarting = argument == u64::MAX;
        let launches = [if restarting { 27 } else { 26 }, 27, 27].map(|role| Supervision {
            owner: 0,
            peer: 0,
            program: program(role, 0),
            frame_limit: usize::MAX,
            framebuffer: None,
            keyboard: false,
            disk: false,
        });
        let allowance = if restarting { 2_000 } else { 20_000 };
        let limits = [allowance, 0, if restarting { 1_000 } else { 0 }, 0, 0];
        let start = cathedral_arch::cpu_time_us();
        // SAFETY: Boot owns frames, ELF bytes and entry machinery on the sole CPU.
        let reports = unsafe {
            user_tasks::run_configured(
                &mut memory.frames,
                &memory.layout,
                memory.image,
                &[
                    program(if restarting { 28 } else { 24 }, argument),
                    // Yield pressure needs a peer that also relinquishes partial
                    // turns; otherwise a pure yielder simply uses almost no CPU.
                    program(25, u64::from(argument == 258)),
                ],
                user_output,
                Config {
                    executables: &[],
                    dynamic_tasks: 0,
                    channel_limits: &[],
                    cpu_limits: &limits,
                    memory: &[],
                    frame_limit: usize::MAX,
                    links: &[],
                    clock_readers: &[0, 1],
                    supervision: &launches,
                    endpoints: &[endpoint(0, 1), endpoint(1, 0)],
                },
            )
        }
        .unwrap();
        let elapsed = cathedral_arch::cpu_time_us() - start;
        let owner = reports[0].cpu;
        let outside = reports[1].cpu;
        writeln!(console, "Cathedral CPU budget: mode={argument} elapsed_us={elapsed} owner={owner:?} outside_us={} child_us={}", outside.direct_us, reports[2].cpu.direct_us).ok();
        assert!(
            reports[..2]
                .iter()
                .all(|r| r.exit == Some(Exit::Returned(0)))
        );
        assert!(owner.throttles > 0 && owner.subtree_us > allowance);
        // Ceiling includes initial burst plus continuous refill. Allow one timer
        // quantum and a margin for unpreemptible entry/deferred work in this lab.
        assert!(owner.subtree_us <= allowance + elapsed * allowance / 100_000 + 30_000);
        // The owner also pays setup and teardown outside the peer's 128-tick
        // sampling window (substantial in debug builds), so compare neither
        // account's lifetime total as if both covered the same interval.
        assert!(outside.direct_us >= 500_000);
        if restarting {
            assert!(reports[0].spawned >= 2);
            assert!(reports[2].cpu.throttles > 0);
            assert!(reports[2].cpu.subtree_us <= 1_000 + elapsed / 100 + 30_000);
        } else {
            assert!(
                reports[2..(3 + (argument & 255) as usize)]
                    .iter()
                    .all(|r| r.cpu.direct_us > 0)
            );
        }
        drop(reports);
        reclaimed(memory, baseline, heap);
    }
    // With no independent runnable account, budget exhaustion must reach HLT and
    // timer refill must resume the process without any message or external task.
    // SAFETY: Same single-CPU session and frame ownership as above.
    let reports = unsafe {
        user_tasks::run_configured(
            &mut memory.frames,
            &memory.layout,
            memory.image,
            &[program(29, 0)],
            user_output,
            Config {
                executables: &[],
                dynamic_tasks: 0,
                channel_limits: &[],
                cpu_limits: &[1_000],
                memory: &[],
                frame_limit: usize::MAX,
                links: &[],
                clock_readers: &[0],
                supervision: &[],
                endpoints: &[],
            },
        )
    }
    .unwrap();
    assert_eq!(reports[0].exit, Some(Exit::Returned(0)));
    assert!(reports[0].cpu.throttles >= 2);
    drop(reports);
    reclaimed(memory, baseline, heap);
    writeln!(console, "Cathedral Rust lab: measured CPU ceilings, descendant charging, yield pressure, restart debt and idle refill passed").ok();
}
