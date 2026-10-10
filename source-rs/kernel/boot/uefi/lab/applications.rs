//! Run independently compiled distribution code through the same user runtime.
use crate::{diagnostics::user_output, heap::HEAP, memory::PreparedMemory};
use cathedral_arch as arch;
use cathedral_core::user_tasks::{self, Executable, Exit, Program};
use cathedral_uart_16550::SerialPort;
use core::fmt::Write;

static EXECUTABLE: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/user.elf"));

pub fn exercise(memory: &mut PreparedMemory, console: &mut SerialPort) {
    let frames = memory.frames.allocated();
    // SAFETY: Boot CPU owns the initialized heap outside a task session.
    let heap = unsafe { HEAP.used() };
    let mut admission_frames = 0;
    for round in 0..2 {
        let statuses = [17 + round, 29 + round];
        let programs = statuses.map(|status| Program {
            executable: Executable::Elf(EXECUTABLE),
            arguments: [status, 0],
        });
        // SAFETY: Sole CPU, IRQs off, initialized entry paths and owned memory.
        let reports = unsafe {
            user_tasks::run(
                &mut memory.frames,
                &memory.layout,
                memory.image,
                &programs,
                user_output,
                usize::MAX,
            )
        }
        .unwrap();
        for (report, status) in reports.iter().zip(statuses) {
            assert_eq!(report.exit, Some(Exit::Returned(status)));
            assert_eq!(report.yields, 16);
            assert_eq!(report.writes, 4);
            assert_eq!(report.rejected, 0);
        }
        admission_frames = reports.iter().map(|report| report.frames).sum();
        drop(reports);
        assert_eq!(memory.frames.allocated(), frames);
        // SAFETY: Every task, context and returned report has been released.
        assert_eq!(unsafe { HEAP.used() }, heap);
        writeln!(console, "Cathedral Rust lab: ELF instances exited {statuses:?}; private data BSS and runtime writes passed").ok();
    }
    // Reject malformed input even after a valid peer has already been admitted.
    let mut malformed = [0u8; 64];
    malformed.copy_from_slice(&EXECUTABLE[..64]);
    malformed[0] = 0;
    let programs = [
        Program {
            executable: Executable::Elf(EXECUTABLE),
            arguments: [17, 0],
        },
        Program {
            executable: Executable::Elf(&malformed),
            arguments: [29, 0],
        },
    ];
    // SAFETY: Admission remains serialized; neither program may run on failure.
    let rejected = unsafe {
        user_tasks::run(
            &mut memory.frames,
            &memory.layout,
            memory.image,
            &programs,
            user_output,
            usize::MAX,
        )
    };
    assert!(matches!(
        rejected,
        Err(user_tasks::Error::Executable(
            user_tasks::elf::Error::Header
        ))
    ));
    assert_eq!(memory.frames.allocated(), frames);
    // SAFETY: Failed admission has returned all allocations.
    assert_eq!(unsafe { HEAP.used() }, heap);
    for budget in 0..admission_frames {
        let programs = [17, 29].map(|status| Program {
            executable: Executable::Elf(EXECUTABLE),
            arguments: [status, 0],
        });
        // SAFETY: Boot context owns mapper and inventory, with IRQs masked.
        let rejected = unsafe {
            user_tasks::run(
                &mut memory.frames,
                &memory.layout,
                memory.image,
                &programs,
                user_output,
                budget,
            )
        };
        assert!(matches!(
            rejected,
            Err(user_tasks::Error::Memory(arch::MemoryError::OutOfFrames))
        ));
        assert_eq!(memory.frames.allocated(), frames);
        // SAFETY: Failed admission has returned all allocations.
        assert_eq!(unsafe { HEAP.used() }, heap);
    }
    writeln!(console, "Cathedral Rust lab: ELF rejection and {admission_frames} admission failure boundaries reclaimed all memory").ok();
}
