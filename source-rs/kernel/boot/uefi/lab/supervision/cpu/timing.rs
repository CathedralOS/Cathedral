//! Observe a periodic task beside busy/yielding work, then under an inadequate
//! ceiling. Assertions prove progress/reclamation; printed spans are observations.
use super::super::*;

pub(super) fn exercise(memory: &mut PreparedMemory, console: &mut SerialPort) {
    let baseline = memory.frames.allocated();
    // SAFETY: Exclusive boot context, no active task session.
    let heap = unsafe { HEAP.used() };
    for (cap, yielding) in [(0, 0), (0, 1), (1_000, 0), (0, 2), (0, 3)] {
        writeln!(console, "Cathedral CPU timing: cap_us={cap} yielding_peer={yielding} period_ticks=8 work_us=1000 samples=16").ok();
        // SAFETY: Boot owns frames, immutable ELF bytes and entry machinery;
        // no other session/CPU can access them during this run.
        let reports = unsafe {
            user_tasks::run_configured(
                &mut memory.frames,
                &memory.layout,
                memory.image,
                &[program(30, 0), program(31, yielding)],
                user_output,
                Config {
                    executables: &[],
                    dynamic_tasks: 0,
                    channel_limits: &[],
                    cpu_limits: &[cap, 0],
                    memory: if yielding >= 2 {
                        &[cathedral_kernel_contracts::memory::Grant {
                            task: 1,
                            private_pages: 4 * cathedral_kernel_contracts::memory::MAX_PAGES,
                            shared_pages: 0,
                        }]
                    } else {
                        &[]
                    },
                    frame_limit: usize::MAX,
                    links: &[],
                    clock_readers: &[0, 1],
                    supervision: &[],
                    endpoints: &[],
                },
            )
        }
        .unwrap();
        assert!(
            reports
                .iter()
                .all(|report| report.exit == Some(Exit::Returned(0)))
        );
        assert!(reports[0].cpu.direct_us >= 16_000);
        assert!(reports[1].cpu.direct_us > 0);
        if yielding >= 2 {
            let pages = if yielding == 2 { 1 } else { 128 };
            assert!(reports[1].memory.allocated_pages >= 5 * pages);
            assert!(reports[1].memory.sealed_pages >= pages);
            assert!(reports[1].memory.unmapped_pages >= pages);
        }
        if cap != 0 {
            assert!(reports[0].cpu.throttles > 0);
        }
        writeln!(
            console,
            "Cathedral CPU timing: periodic={:?} peer={:?}",
            reports[0].cpu, reports[1].cpu
        )
        .ok();
        drop(reports);
        reclaimed(memory, baseline, heap);
    }
    writeln!(console, "Cathedral Rust lab: periodic timing under busy/yielding peers and CPU exhaustion reclaimed all state").ok();
}
