//! Boot supplies executable and authority bounds; restart decisions run in ring 3.
use crate::{diagnostics::user_output, heap::HEAP, memory::PreparedMemory};
use cathedral_contracts::user as abi;
use cathedral_core::{
    ipc::EndpointSpec,
    user_tasks::{self, Config, Executable, Exit, Program, Supervision},
};
use cathedral_uart_16550::SerialPort;
use core::fmt::Write;

static ELF: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/supervision.elf"));
mod links;
mod multiple;

pub fn exercise(memory: &mut PreparedMemory, console: &mut SerialPort) {
    let baseline = memory.frames.allocated();
    // SAFETY: Sole boot CPU, outside task execution.
    let heap = unsafe { HEAP.used() };
    let control = [endpoint(0, 1), endpoint(1, 0)];
    let reports = run(
        memory,
        &[program(0, 0), program(1, 0)],
        &control,
        program(2, 0),
        usize::MAX,
    );
    assert_eq!(reports[0].exit, Some(Exit::Returned(0)));
    assert_eq!(reports[1].exit, Some(Exit::Returned(0)));
    assert_eq!(reports[0].spawned, 32);
    assert_eq!(reports[0].reaped, 32);
    assert!(reports[0].waits_blocked > 0);
    assert_eq!(reports[1].ipc_received, 32 * 5); // Commands plus four echoes per child.
    assert!(matches!(reports[2].exit, Some(Exit::Fault(fault)) if fault.vector == 6));
    drop(reports);
    reclaimed(memory, baseline, heap);
    writeln!(console, "Cathedral Rust lab: userspace supervisor restarted 32 faulted services; live client reconnected with fresh grants").ok();

    for (owner, child) in [(5, 6), (9, 6), (8, 7)] {
        let reports = run(
            memory,
            &[program(owner, 0), program(4, 0)],
            &[endpoint(0, 1)],
            program(child, 0),
            usize::MAX,
        );
        if owner == 9 {
            assert!(matches!(reports[0].exit, Some(Exit::Fault(fault)) if fault.vector == 6));
        } else {
            assert_eq!(reports[0].exit, Some(Exit::Returned(0)));
        }
        assert_eq!(reports[1].exit, Some(Exit::Returned(0)));
        assert_eq!(reports[0].spawned, 1);
        assert_eq!(reports[0].reaped, 1);
        if owner == 5 || owner == 9 {
            assert_eq!(reports[0].cancelled, 1);
            assert_eq!(reports[2].exit, Some(Exit::Cancelled));
            assert_eq!(reports[2].receives_blocked, 1);
        } else {
            assert_eq!(reports[2].exit, Some(Exit::Returned(u64::MAX)));
        }
        drop(reports);
        reclaimed(memory, baseline, heap);
    }
    writeln!(console, "Cathedral Rust lab: supervision parent-exit cancellation and collected return status reclaimed all memory").ok();

    for budget in [0, 1, 10] {
        let reports = run(
            memory,
            &[program(3, abi::NO_MEMORY), program(4, 0)],
            &[endpoint(0, 1)],
            program(2, 0),
            budget,
        );
        assert!(
            reports[..2]
                .iter()
                .all(|report| report.exit == Some(Exit::Returned(0)))
        );
        assert_eq!(reports[0].spawned, 0);
        assert_eq!(reports[2].exit, None);
        drop(reports);
        reclaimed(memory, baseline, heap);
    }
    let malformed = Program {
        executable: Executable::Elf(b"invalid"),
        arguments: [2, 0],
    };
    let reports = run(
        memory,
        &[program(3, abi::BAD_EXECUTABLE), program(4, 0)],
        &[endpoint(0, 1)],
        malformed,
        usize::MAX,
    );
    assert!(
        reports[..2]
            .iter()
            .all(|report| report.exit == Some(Exit::Returned(0)))
    );
    assert_eq!(reports[0].spawned, 0);
    drop(reports);
    reclaimed(memory, baseline, heap);
    writeln!(console, "Cathedral Rust lab: supervision failed-spawn retries preserved live peers and memory baselines").ok();
    multiple::exercise(memory, console);
    links::exercise(memory, console);
}
fn program(role: u64, argument: u64) -> Program<'static> {
    Program {
        executable: Executable::Elf(ELF),
        arguments: [role, argument],
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
    child: Program<'_>,
    budget: usize,
) -> alloc::vec::Vec<user_tasks::Report> {
    // SAFETY: Sole CPU, IRQs off, boot-owned executable bytes and initialized entry paths.
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
                clock_readers: &[],
                supervision: &[Supervision {
                    disk: false,
                    keyboard: false,
                    framebuffer: None,
                    owner: 0,
                    peer: 1,
                    program: child,
                    frame_limit: budget,
                }],
            },
        )
    }
    .unwrap()
}
fn reclaimed(memory: &PreparedMemory, frames: usize, heap: usize) {
    assert_eq!(memory.frames.allocated(), frames);
    // SAFETY: All tasks and returned reports have been destroyed.
    assert_eq!(unsafe { HEAP.used() }, heap);
}
