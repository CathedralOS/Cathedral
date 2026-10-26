// Compose two independent user executables and verify hardware grant recovery.
use crate::console::Console as SerialPort;
use crate::{diagnostics::user_output, heap::HEAP, memory::PreparedMemory};
use cathedral_core::{
    ipc::EndpointSpec,
    user_tasks::{self, Config, Executable, Exit, Program, Supervision},
};
use cathedral_kernel_contracts::framebuffer::USER_ADDRESS;
use core::fmt::Write;
static CLIENT: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/display-lab.elf"));
static SERVICE: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/display-service.elf"));

pub fn exercise(memory: &mut PreparedMemory, console: &mut SerialPort) {
    let Some(framebuffer) = memory.framebuffer else {
        writeln!(
            console,
            "Cathedral Rust lab: display unavailable (no supported linear GOP mode)"
        )
        .ok();
        #[cfg(feature = "smoke-test")]
        panic!("display smoke requires supported GOP");
        #[cfg(not(feature = "smoke-test"))]
        return;
    };
    writeln!(
        console,
        "Cathedral Rust lab: GOP {}x{} stride={} bytes={} format={}",
        framebuffer.width,
        framebuffer.height,
        framebuffer.stride,
        framebuffer.bytes,
        framebuffer.format
    )
    .ok();
    let frames = memory.frames.allocated();
    // SAFETY: Exclusive boot CPU, no active session.
    let heap = unsafe { HEAP.used() };
    let programs = [program(0), program(1), program(2), program(3)];
    let endpoints = [(0, 1), (1, 0), (0, 2), (2, 0)].map(|(sender, receiver)| EndpointSpec {
        sender,
        receiver,
        revoker: None,
    });
    // SAFETY: Sole CPU, owned images/entry paths, IRQs off. Firmware relinquished
    // the reserved display aperture; only the supervised service may map it.
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
                endpoints: &endpoints,
                // Two coordination streams at the supervisor, one upstream
                // plus one provider stream at its client, and one at each peer.
                memory: &[(0, 2), (1, 2), (2, 1), (4, 1)].map(|(task, pages)| {
                    cathedral_kernel_contracts::memory::Grant {
                        task,
                        private_pages: pages,
                        shared_pages: pages,
                    }
                }),
                channel_limits: &[2, 2, 1, 0, 1],
                cpu_limits: &[],
                clock_readers: &[0, 1, 2, 4],
                supervision: &[Supervision {
                    disk: false,
                    device: None,
                    keyboard: false,
                    owner: 0,
                    peer: 1,
                    program: Program {
                        executable: Executable::Elf(SERVICE),
                        arguments: [9, 0],
                    },
                    frame_limit: usize::MAX,
                    framebuffer: Some(framebuffer),
                }],
            },
        )
    }
    .unwrap();
    assert!(
        reports[..3]
            .iter()
            .all(|report| report.exit == Some(Exit::Returned(0)))
    );
    assert!(
        matches!(reports[3].exit, Some(Exit::Fault(fault)) if fault.vector == 14 && fault.address == USER_ADDRESS && fault.error == 4)
    );
    assert_eq!(
        (reports[0].spawned, reports[0].reaped, reports[0].cancelled),
        (2, 2, 1)
    );
    // Shared transport uses maintenance waits while retaining the exact call,
    // so its aggregate wait_timeouts/ipc_sent counters no longer measure the
    // old payload exchange. Guest assertions still require provider completion
    // before the same deadline, both observer replies and transport reclamation.
    assert_eq!(reports[4].exit, Some(Exit::Cancelled));
    drop(reports);
    assert_eq!(memory.frames.allocated(), frames);
    // SAFETY: All task contexts and reports have been dropped.
    assert_eq!(unsafe { HEAP.used() }, heap);
    writeln!(console, "Cathedral Rust lab: display service faulted and restarted; pattern redrawn; observer progressed; ungranted mapping fault contained; all task memory reclaimed").ok();
    probes::exercise(memory, console);
}
fn program(role: u64) -> Program<'static> {
    Program {
        executable: Executable::Elf(CLIENT),
        arguments: [role, 0],
    }
}
