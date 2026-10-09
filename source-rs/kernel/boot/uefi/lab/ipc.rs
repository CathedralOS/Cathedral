//! Lab composition: grants and assertions, with all executable behavior in distro ELF.
use crate::{diagnostics::user_output, heap::HEAP, memory::PreparedMemory};
use cathedral_core::{
    ipc::EndpointSpec,
    users::{self, Config, Executable, Exit, Program},
};
use cathedral_uart_16550::SerialPort;
use core::fmt::Write;

static EXECUTABLE: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/ipc.elf"));

pub fn exercise(memory: &mut PreparedMemory, console: &mut SerialPort) {
    let baseline = memory.frames.allocated();
    // SAFETY: Boot CPU owns initialized heap, outside any task session.
    let heap = unsafe { HEAP.used() };
    let mut stale = 0;
    for round in 0..2 {
        let programs = [0, 1].map(|role| program(role, stale));
        let endpoints = [endpoint(1, 0, None), endpoint(0, 1, None)];
        let reports = run(memory, &programs, &endpoints);
        assert_eq!(reports[0].exit, Some(Exit::Returned(0)));
        let Some(Exit::Returned(ticket)) = reports[1].exit else {
            panic!("IPC client failed")
        };
        assert!(ticket > 0 && (ticket as i64) > 0 && ticket != stale);
        stale = ticket;
        assert!(
            reports
                .iter()
                .map(|report| report.receives_blocked)
                .sum::<usize>()
                > 0
        );
        for report in &reports {
            assert_eq!(report.ipc_sent, 32);
            assert_eq!(report.ipc_received, 32);
        }
        drop(reports);
        reclaimed(memory, baseline, heap);
        writeln!(console, "Cathedral Rust lab: IPC echo round={round} 32 exchanges; bound rights and stale handles passed").ok();
    }
    for (name, roles, specs, fault) in [
        ("peer exit", [2, 3], [endpoint(1, 0, None)], false),
        ("peer fault", [2, 4], [endpoint(1, 0, None)], true),
        ("revocation", [5, 6], [endpoint(1, 0, Some(1))], false),
    ] {
        let reports = run(memory, &roles.map(|role| program(role, 0)), &specs);
        assert_eq!(reports[0].exit, Some(Exit::Returned(0)));
        assert_eq!(reports[0].receives_blocked, 1);
        if fault {
            assert!(matches!(reports[1].exit, Some(Exit::Fault(fault)) if fault.vector == 6));
        } else {
            assert_eq!(reports[1].exit, Some(Exit::Returned(0)));
        }
        drop(reports);
        reclaimed(memory, baseline, heap);
        writeln!(
            console,
            "Cathedral Rust lab: IPC {name} woke blocked receiver and reclaimed all memory"
        )
        .ok();
    }
    let reports = run(
        memory,
        &[program(7, 0), program(8, 0)],
        &[endpoint(0, 1, None), endpoint(0, 1, None)],
    );
    assert!(
        reports
            .iter()
            .all(|report| report.exit == Some(Exit::Returned(0)))
    );
    assert!(reports[1].rejected >= 9);
    drop(reports);
    reclaimed(memory, baseline, heap);
    writeln!(console, "Cathedral Rust lab: IPC backpressure and checked copyout preserved queued message; all memory reclaimed").ok();
}
fn program(role: u64, stale: u64) -> Program<'static> {
    Program {
        executable: Executable::Elf(EXECUTABLE),
        arguments: [role, stale],
    }
}
fn endpoint(sender: usize, receiver: usize, revoker: Option<usize>) -> EndpointSpec {
    EndpointSpec {
        sender,
        receiver,
        revoker,
    }
}
fn run(
    memory: &mut PreparedMemory,
    programs: &[Program<'_>],
    endpoints: &[EndpointSpec],
) -> alloc::vec::Vec<users::Report> {
    // SAFETY: Sole CPU, IRQs off, initialized entry paths/heap and owned memory.
    unsafe {
        users::run_configured(
            &mut memory.frames,
            &memory.layout,
            memory.image,
            programs,
            user_output,
            Config {
                frame_limit: usize::MAX,
                endpoints,
                supervision: &[],
                clock_readers: &[],
            },
        )
    }
    .unwrap()
}
fn reclaimed(memory: &PreparedMemory, frames: usize, heap: usize) {
    assert_eq!(memory.frames.allocated(), frames);
    // SAFETY: Session and returned reports are gone; heap is exclusively owned.
    assert_eq!(unsafe { HEAP.used() }, heap);
}
