//! Verify userspace timeout/recovery and unrelated progress under a spinning peer.
use crate::console::Console as SerialPort;
use crate::{diagnostics::user_output, heap::HEAP, memory::PreparedMemory};
use cathedral_core::{
    ipc::EndpointSpec,
    user_tasks::{self, Config, Executable, Exit, Program, Supervision},
};
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
        &[0, 1, 2, 3],
    );
    assert!(
        reports[..3]
            .iter()
            .all(|report| report.exit == Some(Exit::Returned(0)))
    );
    let owner = &reports[0];
    assert_eq!((owner.spawned, owner.reaped, owner.cancelled), (8, 8, 8));
    assert_eq!((owner.cancelled_ready, owner.cancelled_blocked), (4, 4));
    // Guest checks both task-wait timeouts per round. Shared transport also has
    // maintenance deadlines; aggregate timeout counts may include those waits.
    assert!(owner.wait_timeouts >= 16);
    assert!(owner.waits_blocked >= 8);
    assert_eq!(reports[3].exit, Some(Exit::Cancelled));
    assert!(reports[3].preemptions >= 2); // Last service is a syscall-free spin loop.
    // Both observer replies per round are checked in userspace. Copied-send
    // counters now count grant setup, not these shared application messages.
    drop(reports);
    reclaimed(memory, frames, heap);
    writeln!(console, "Cathedral Rust lab: deadlines recovered 4 silent and 4 spinning services; independent observer progressed; all memory reclaimed").ok();

    let reports = run(
        memory,
        &[program(32), program(4)],
        &[endpoint(0, 1)],
        33,
        &[],
    );
    assert!(reports.iter().all(|r| r.exit == Some(Exit::Returned(0))));
    assert_eq!(
        (reports[0].spawned, reports[0].reaped, reports[0].cancelled),
        (1, 1, 0)
    );
    assert_eq!(reports[0].wait_timeouts, 0);
    drop(reports);
    reclaimed(memory, frames, heap);
    writeln!(console, "Cathedral Rust lab: clockless initial and child tasks refused timed operations without consuming notification signals").ok();

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
    // The gate's grant receive waits through the idle interval. After its signal,
    // the sender may have exited, so observing closure need not block again.
    assert!(reports[1].receives_blocked >= 1);
    assert_eq!(reports[1].ipc_received, 1);
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
    let shared = role == 12;
    let transport_memory = [(0, 2), (1, 2), (2, 1), (3, 1)].map(|(task, pages)| {
        cathedral_kernel_contracts::memory::Grant {
            task,
            private_pages: pages,
            shared_pages: pages,
        }
    });
    // SAFETY: Sole boot CPU with initialized entry paths, IRQs off, owned images.
    unsafe {
        user_tasks::run_configured(
            &mut memory.frames,
            &memory.layout,
            memory.image,
            programs,
            user_output,
            Config {
                executables: &[],
                dynamic_tasks: 0,
                links: &[],
                frame_limit: usize::MAX,
                endpoints,
                memory: if shared { &transport_memory } else { &[] },
                channel_limits: if shared {
                    &[2, 2, 1, 1]
                } else if role == 33 {
                    &[1, 0, 1]
                } else {
                    &[1, 0, 0]
                },
                cpu_limits: &[],
                clock_readers: clocks,
                supervision: &[Supervision {
                    disk: false,
                    device: None,
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
