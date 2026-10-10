//! Real frame rollback and supervised ring-3 page-object exercises.
use crate::{diagnostics::user_output, heap::HEAP, memory::PreparedMemory};
use cathedral_contracts::memory::Grant;
use cathedral_core::{
    link::LinkSpec,
    user_tasks::{self, Config, Executable, Exit, Program, Supervision},
};
use cathedral_uart_16550::SerialPort;
use core::fmt::Write;
static ELF: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/memory.elf"));
pub fn exercise(memory: &mut PreparedMemory, console: &mut SerialPort) {
    let baseline = memory.frames.allocated();
    // SAFETY: Sole CPU in boot context, no active tasks.
    let heap = unsafe { HEAP.used() };
    // SAFETY: Boot-owned live image/layout and reclaiming frame allocator.
    unsafe {
        user_tasks::exercise_memory_rollback(&mut memory.frames, &memory.layout, memory.image)
    };
    assert_eq!(memory.frames.allocated(), baseline);
    // SAFETY: No task owns heap state now.
    assert_eq!(unsafe { HEAP.used() }, heap);
    writeln!(
        console,
        "Cathedral Rust lab: runtime page allocation rolled back at every frame boundary"
    )
    .ok();
    for _ in 0..2 {
        let grants = [1, 2].map(|role| Supervision {
            owner: 0,
            peer: 0,
            disk: false,
            keyboard: false,
            framebuffer: None,
            program: program(role),
            frame_limit: usize::MAX,
        });
        // SAFETY: Exclusive boot custody, bounded ELF and explicit peer/page grants.
        let reports = unsafe {
            user_tasks::run_configured(
                &mut memory.frames,
                &memory.layout,
                memory.image,
                &[program(0)],
                user_output,
                Config {
                    frame_limit: usize::MAX,
                    endpoints: &[],
                    clock_readers: &[],
                    supervision: &grants,
                    links: &[LinkSpec {
                        client: 1,
                        service: 2,
                    }],
                    memory: &[
                        Grant {
                            task: 1,
                            private_pages: 4,
                            shared_pages: 0,
                        },
                        Grant {
                            task: 2,
                            private_pages: 0,
                            shared_pages: 4,
                        },
                    ],
                },
            )
        }
        .unwrap();
        assert_eq!(reports[0].exit, Some(Exit::Returned(0)));
        assert_eq!(reports[0].spawned, reports[0].reaped);
        assert!(reports[0].spawned >= 28);
        assert!(
            reports[1..]
                .iter()
                .all(|report| report.exit == Some(Exit::Cancelled))
        );
        drop(reports);
        assert_eq!(memory.frames.allocated(), baseline);
        // SAFETY: Session completely retired; no live allocator users.
        assert_eq!(unsafe { HEAP.used() }, heap);
    }
    writeln!(console, "Cathedral Rust lab: private budgets zeroing shared leases NX write faults stale handles and peer death passed; all memory reclaimed").ok();
}
fn program(role: u64) -> Program<'static> {
    Program {
        executable: Executable::Elf(ELF),
        arguments: [role, 0],
    }
}
