//! Guest composition, page-edit accounting and cleanup; no production rendering policy.
use crate::{diagnostics::user_output, heap::HEAP, memory::PreparedMemory};
use cathedral_contracts::memory::Grant;
use cathedral_core::{
    link::LinkSpec,
    user_tasks::{self, Config, Executable, Exit, MemoryReport, Program, Supervision},
};
use cathedral_uart_16550::SerialPort;
use core::fmt::Write;
static ELF: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/rendering.elf"));
pub fn exercise(memory: &mut PreparedMemory, console: &mut SerialPort) {
    let frames = memory.frames.allocated();
    // SAFETY: Sole CPU in boot context with no live tasks.
    let heap = unsafe { HEAP.used() };
    for _ in 0..2 {
        let launches = [1, 2, 3].map(|role| Supervision {
            owner: 0,
            peer: 0,
            disk: false,
            keyboard: false,
            framebuffer: None,
            program: program(role),
            frame_limit: usize::MAX,
        });
        // SAFETY: Exclusive boot custody and bounded ELF, explicit page and peer grants.
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
                    supervision: &launches,
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
                            private_pages: 1,
                            shared_pages: 4,
                        },
                        Grant {
                            task: 3,
                            private_pages: 1,
                            shared_pages: 0,
                        },
                    ],
                },
            )
        }
        .unwrap();
        assert!(
            reports[..3]
                .iter()
                .all(|r| r.exit == Some(Exit::Returned(0)))
        );
        assert!(matches!(reports[3].exit, Some(Exit::Fault(_))));
        assert_eq!((reports[0].spawned, reports[0].reaped), (3, 3));
        assert_eq!(
            reports[1].memory,
            MemoryReport {
                deferred_calls: 12,
                allocated_pages: 4,
                mapped_pages: 4,
                sealed_pages: 4,
                unmapped_pages: 4
            }
        );
        assert_eq!(
            reports[2].memory,
            MemoryReport {
                deferred_calls: 12,
                allocated_pages: 2,
                mapped_pages: 6,
                sealed_pages: 0,
                unmapped_pages: 6
            }
        );
        assert_eq!(
            reports[3].memory,
            MemoryReport {
                deferred_calls: 1,
                allocated_pages: 1,
                mapped_pages: 1,
                sealed_pages: 0,
                unmapped_pages: 1
            }
        );
        assert_eq!(reports.iter().map(|r| r.ipc_sent).sum::<usize>(), 4);
        assert_eq!(reports.iter().map(|r| r.ipc_received).sum::<usize>(), 4);
        writeln!(console, "Cathedral rendering lab: producer {:?}; consumer {:?}; guard {:?}; 4 IPC deliveries (2 frame, 2 startup)", reports[1].memory, reports[2].memory, reports[3].memory).ok();
        drop(reports);
        assert_eq!(memory.frames.allocated(), frames);
        // SAFETY: Session has retired every task and its mappings.
        assert_eq!(unsafe { HEAP.used() }, heap);
    }
    writeln!(console, "Cathedral Rust lab: rendering equivalence same-page escape page fault mapping counts and reclamation passed").ok();
}
fn program(role: u64) -> Program<'static> {
    Program {
        executable: Executable::Elf(ELF),
        arguments: [role, 0],
    }
}
