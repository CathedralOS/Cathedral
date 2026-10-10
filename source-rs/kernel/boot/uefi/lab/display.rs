//! Compose two independent user executables and verify hardware grant recovery.
mod probes;
use crate::{diagnostics::user_output, heap::HEAP, memory::PreparedMemory};
use cathedral_contracts::display::USER_ADDRESS;
use cathedral_core::{
    ipc::EndpointSpec,
    users::{self, Config, Executable, Exit, Program, Supervision},
};
use cathedral_uart_16550::SerialPort;
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
        users::run_configured(
            &mut memory.frames,
            &memory.layout,
            memory.image,
            &programs,
            user_output,
            Config {
                links: &[],
                frame_limit: usize::MAX,
                endpoints: &endpoints,
                clock_readers: &[0, 2],
                supervision: &[Supervision {
                    disk: false,
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
    assert_eq!(reports[0].wait_timeouts, 0);
    assert_eq!(reports[2].ipc_sent, 4);
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
