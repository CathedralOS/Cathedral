//! Launch the same userspace peers through the existing supervisor and sharing graph.
use super::processes::{program, reclaimed};
use crate::{console::Console, diagnostics::user_output, memory::PreparedMemory};
use cathedral_core::{
    ipc::link::LinkSpec,
    user_tasks::{self, Config, Exit, Supervision},
};
use core::fmt::Write;
pub(super) fn exercise(
    memory: &mut PreparedMemory,
    console: &mut Console,
    baseline: usize,
    heap: usize,
) {
    for role in [6, 8] {
        let launches = [role, 7].map(|role| Supervision {
            owner: 0,
            peer: 0,
            disk: false,
            device: None,
            keyboard: false,
            framebuffer: None,
            program: program(role, 0),
            frame_limit: usize::MAX,
        });
        let grants = [1, 2].map(|task| cathedral_kernel_contracts::memory::Grant {
            task,
            private_pages: 2,
            shared_pages: 2,
        });
        // SAFETY: Sole boot CPU outside a session, owned live layout and reclaiming allocator.
        let reports = unsafe {
            user_tasks::run_configured(
                &mut memory.frames,
                &memory.layout,
                memory.image,
                &[program(9, u64::from(role == 8))],
                user_output,
                Config {
                    executables: &[],
                    dynamic_tasks: 0,
                    links: &[LinkSpec {
                        client: 1,
                        service: 2,
                    }],
                    frame_limit: usize::MAX,
                    endpoints: &[],
                    supervision: &launches,
                    memory: &grants,
                    channel_limits: &[],
                    cpu_limits: &[],
                    clock_readers: &[],
                },
            )
        }
        .unwrap();
        assert_eq!(reports[0].exit, Some(Exit::Returned(0)));
        assert_eq!(reports[0].spawned, 2);
        assert_eq!(reports[0].reaped, 2);
        if role == 6 {
            assert_eq!(reports[1].exit, Some(Exit::Returned(0)));
        } else {
            assert!(
                matches!(reports[1].exit,Some(Exit::Fault(f)) if f.vector==0x24 && f.address==cathedral_kernel_contracts::memory::USER_BASE)
            );
        }
        assert_eq!(reports[2].exit, Some(Exit::Returned(0)));
        assert_eq!(reports[1].memory.sealed_pages, 2);
        assert_eq!(reports[2].memory.mapped_pages, 2);
        drop(reports);
        reclaimed(memory, baseline, heap);
    }
    writeln!(console,"Cathedral Rust lab: ARM64 sealed pages rejected writes; reader survived producer exit and fault").ok();
}
