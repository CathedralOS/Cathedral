// Boot supplies executable and authority bounds; restart decisions run in ring 3.
use crate::console::Console as SerialPort;
use crate::{diagnostics::user_output, heap::HEAP, memory::PreparedMemory};
use cathedral_core::{
    ipc::EndpointSpec,
    user_tasks::{self, Config, Executable, Exit, Program, Supervision},
};
use cathedral_kernel_contracts::user as abi;
use core::fmt::Write;

static ELF: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/supervision.elf"));

pub fn exercise(memory: &mut PreparedMemory, console: &mut SerialPort) {
    lifecycle(memory, console);
    multiple::exercise(memory, console);
    links::exercise(memory, console);
    cpu::exercise(memory, console);
    watchdog::exercise(memory, console);
}
fn lifecycle(memory: &mut PreparedMemory, console: &mut SerialPort) {
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
    // The guest checks all 32 commands and 128 exact shared echoes. Copied-IPC
    // counters now describe capability setup, not application delivery.
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
    let launches = [
        Supervision {
            disk: false,
            device: None,
            keyboard: false,
            framebuffer: None,
            owner: 0,
            peer: 1,
            program: Program {
                executable: child.executable,
                arguments: child.arguments,
            },
            frame_limit: budget,
        },
        Supervision {
            disk: false,
            device: None,
            keyboard: false,
            framebuffer: None,
            owner: 0,
            peer: 0,
            program: child,
            frame_limit: budget,
        },
    ];
    // Failure probes also propose a launch loan. It must survive ELF and partial
    // frame-admission failure just like their initial endpoint bundle.
    let launch_count = if programs[0].arguments[0] == 3 { 2 } else { 1 };
    let shared_restarts = programs[0].arguments[0] == 0;
    let transport_memory =
        [(0, 1), (1, 2), (2, 1)].map(|(task, pages)| cathedral_kernel_contracts::memory::Grant {
            task,
            private_pages: pages,
            shared_pages: pages,
        });
    // SAFETY: Sole CPU, IRQs off, boot-owned executable bytes and initialized entry paths.
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
                memory: if shared_restarts {
                    &transport_memory
                } else {
                    &[]
                },
                // Failure probes retain two proposed channels; other lifecycle
                // owners fund one completion gate. Receivers allocate neither.
                channel_limits: if shared_restarts {
                    &[1, 2, 1]
                } else if launch_count == 2 {
                    &[2, 0, 0, 0]
                } else {
                    &[1, 0, 0]
                },
                cpu_limits: &[],
                clock_readers: if shared_restarts { &[0, 1, 2] } else { &[] },
                supervision: &launches[..launch_count],
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
