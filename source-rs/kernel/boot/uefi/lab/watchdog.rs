//! Verify userspace timeout/recovery and unrelated progress under a spinning peer.
use crate::{diagnostics::user_output, heap::HEAP, memory::PreparedMemory};
use cathedral_core::{
    ipc::EndpointSpec,
    user_tasks::{self, Config, Executable, Exit, Program, Supervision},
};
use cathedral_uart_16550::SerialPort;
use core::fmt::Write;

static ELF: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/supervision.elf"));

pub fn exercise(memory: &mut PreparedMemory, console: &mut SerialPort) {
    let frames = memory.frames.allocated();
    // SAFETY: Exclusive boot CPU, no active task session.
    let heap = unsafe { HEAP.used() };
    let endpoints = [
        endpoint(0, 1),
        endpoint(1, 0),
        endpoint(0, 2),
        endpoint(2, 0),
    ];
    let reports = run(
        memory,
        &[program(10), program(11), program(13)],
        &endpoints,
        12,
        &[0, 2],
    );
    assert!(
        reports[..3]
            .iter()
            .all(|report| report.exit == Some(Exit::Returned(0)))
    );
    let owner = &reports[0];
    assert_eq!((owner.spawned, owner.reaped, owner.cancelled), (8, 8, 8));
    assert_eq!((owner.cancelled_ready, owner.cancelled_blocked), (4, 4));
    assert_eq!(owner.wait_timeouts, 16);
    assert!(owner.waits_blocked >= 8);
    assert_eq!(reports[3].exit, Some(Exit::Cancelled));
    assert!(reports[3].preemptions >= 2); // Last service is a syscall-free spin loop.
    assert_eq!(reports[2].ipc_sent, 16);
    drop(reports);
    reclaimed(memory, frames, heap);
    writeln!(console, "Cathedral Rust lab: deadlines recovered 4 silent and 4 spinning services; independent observer progressed; all memory reclaimed").ok();

    let reports = run(
        memory,
        &[program(14), program(4)],
        &[endpoint(0, 1)],
        7,
        &[0],
    );
    assert!(
        reports[..2]
            .iter()
            .all(|report| report.exit == Some(Exit::Returned(0)))
    );
    assert_eq!(reports[0].wait_timeouts, 0);
    assert_eq!(reports[0].cancelled, 0);
    assert_eq!(reports[0].reaped, 2);
    drop(reports);
    reclaimed(memory, frames, heap);
    writeln!(console, "Cathedral Rust lab: clock grants copy checks and deadline completion/cancellation precedence passed").ok();

    let reports = run(
        memory,
        &[program(15), program(4)],
        &[endpoint(0, 1)],
        6,
        &[0],
    );
    assert!(
        reports[..2]
            .iter()
            .all(|report| report.exit == Some(Exit::Returned(0)))
    );
    assert_eq!(reports[0].wait_timeouts, 1);
    assert_eq!(reports[0].waits_blocked, 1);
    assert_eq!(reports[0].cancelled_blocked, 1);
    assert_eq!(reports[1].receives_blocked, 2);
    assert_eq!(reports[2].receives_blocked, 1);
    drop(reports);
    reclaimed(memory, frames, heap);
    writeln!(console, "Cathedral Rust lab: deadline woke an idle session with every user task blocked; all memory reclaimed").ok();
}
fn program(role: u64) -> Program<'static> {
    Program {
        executable: Executable::Elf(ELF),
        arguments: [role, 0],
    }
}
fn endpoint(sender: usize, receiver: usize) -> EndpointSpec {
    EndpointSpec {
        sender,
        receiver,
        revoker: None,
    }
}
fn run(
    memory: &mut PreparedMemory,
    programs: &[Program<'_>],
    endpoints: &[EndpointSpec],
    role: u64,
    clocks: &[usize],
) -> alloc::vec::Vec<user_tasks::Report> {
    // SAFETY: Sole boot CPU with initialized entry paths, IRQs off, owned images.
    unsafe {
        user_tasks::run_configured(
            &mut memory.frames,
            &memory.layout,
            memory.image,
            programs,
            user_output,
            Config {
                links: &[],
                frame_limit: usize::MAX,
                endpoints,
                memory: &[],
                clock_readers: clocks,
                supervision: &[Supervision {
                    disk: false,
                    keyboard: false,
                    framebuffer: None,
                    owner: 0,
                    peer: 1,
                    program: program(role),
                    frame_limit: usize::MAX,
                }],
            },
        )
    }
    .unwrap()
}
fn reclaimed(memory: &PreparedMemory, frames: usize, heap: usize) {
    assert_eq!(memory.frames.allocated(), frames);
    // SAFETY: Session and returned reports are gone.
    assert_eq!(unsafe { HEAP.used() }, heap);
}
