//! Three independent processes exchange one physical object through ordinary IPC.
use crate::{console::Console, diagnostics::user_output, memory::PreparedMemory};
use cathedral_core::{
    ipc::EndpointSpec,
    user_tasks::{self, Config, Executable, Exit, Program},
};
use core::fmt::Write;
static ELF: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/shared-memory.elf"));
pub fn exercise(memory: &mut PreparedMemory, console: &mut Console) {
    let baseline = memory.frames.allocated();
    // SAFETY: Outside any session; sole boot CPU owns the heap.
    let heap = unsafe { crate::heap::HEAP.used() };
    for mode in [0, 1, 2, 0] {
        let programs = [0, 1, 2].map(|role| Program {
            executable: Executable::Elf(ELF),
            arguments: [role, mode],
        });
        let grants = [0, 1, 2].map(|task| cathedral_kernel_contracts::memory::Grant {
            task,
            private_pages: 1,
            shared_pages: 2,
        });
        // SAFETY: Single boot CPU, owned mappings and reclaiming physical allocator.
        let reports = unsafe {
            user_tasks::run_configured(
                &mut memory.frames,
                &memory.layout,
                memory.image,
                &programs,
                user_output,
                Config {
                    executables: &[],
                    dynamic_tasks: 0,
                    links: &[],
                    frame_limit: usize::MAX,
                    endpoints: &[
                        EndpointSpec {
                            sender: 0,
                            receiver: 1,
                            revoker: None,
                        },
                        EndpointSpec {
                            sender: 1,
                            receiver: 2,
                            revoker: None,
                        },
                    ],
                    supervision: &[],
                    memory: &grants,
                    channel_limits: &[],
                    cpu_limits: &[],
                    clock_readers: &[],
                },
            )
        }
        .unwrap();
        assert_eq!(reports[0].exit, Some(Exit::Returned(0)));
        assert_eq!(reports[1].exit, Some(Exit::Returned(0)));
        if mode != 1 {
            assert!(reports[0].completion_order < reports[1].completion_order);
            assert!(reports[1].completion_order < reports[2].completion_order);
        }
        assert_eq!(
            reports[1].memory.mapped_pages, 0,
            "forwarder must never map pixel bytes"
        );
        if mode == 2 {
            let Some(Exit::Fault(fault)) = reports[2].exit else {
                panic!("reader must fault");
            };
            assert_eq!(
                fault.address,
                cathedral_kernel_contracts::memory::address(
                    cathedral_kernel_contracts::memory::MAX_PRIVATE_REGIONS
                )
            );
            #[cfg(target_arch = "x86_64")]
            assert!(fault.vector == 14 && fault.error & 7 == 7);
            #[cfg(target_arch = "aarch64")]
            assert!(fault.vector == 0x24 && fault.error & 0x7f == 0x4f);
        } else {
            assert_eq!(reports[2].exit, Some(Exit::Returned(0)));
        }
        assert_eq!(
            reports
                .iter()
                .map(|r| r.memory.allocated_pages)
                .sum::<usize>(),
            1
        );
        drop(reports);
        assert_eq!(memory.frames.allocated(), baseline);
        // SAFETY: All processes returned and all session storage was reclaimed.
        assert_eq!(unsafe { crate::heap::HEAP.used() }, heap);
    }
    writeln!(console, "Cathedral Rust lab: shared objects forwarded without mappings; lifetime and reclamation passed").ok();
}
