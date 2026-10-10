//! Boot-level experiments: ring transitions, bad requests, isolation and cleanup.
use crate::{diagnostics::user_output, heap::HEAP, memory::PreparedMemory};
use cathedral_arch as arch;
use cathedral_core::user_tasks::{self, Executable, Exit, Program};
use cathedral_uart_16550::SerialPort;
use core::fmt::Write;

pub fn exercise(memory: &mut PreparedMemory, console: &mut SerialPort) {
    let frames = memory.frames.allocated();
    // SAFETY: Sole boot CPU, outside a task session.
    let heap = unsafe { HEAP.used() };
    let code = arch::user_probe();
    let mut admission_frames = 0;
    for mode in 1..=10 {
        let probe_address = if mode == 9 {
            memory.layout.root_address()
        } else {
            memory.layout.heap_start
        };
        let programs = [
            Program {
                executable: Executable::Probe(code),
                arguments: [mode, probe_address],
            },
            Program {
                executable: Executable::Probe(code),
                arguments: [0, memory.layout.heap_start],
            },
        ];
        // SAFETY: Initialized single-CPU machine, owned frame inventory and layout,
        // IRQs off; output does not allocate, suspend or retain the supplied slice.
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
        let Exit::Fault(fault) = reports[0].exit.expect("missing task outcome") else {
            panic!("offender escaped: {:?}", reports[0]);
        };
        let (vector, error, address) = match mode {
            1 => (14, 5, Some(memory.layout.heap_start)),
            2 | 3 | 10 => (13, 0, None),
            4 => (14, 7, Some(arch::USER_CODE)),
            5 => (14, 21, Some(arch::USER_STACK_TOP - 8)),
            6 => (14, 6, Some(arch::USER_STACK - 8)),
            7 => (13, (48 << 3) | 2, None),
            8 => (6, 0, None),
            9 => (14, 4, Some(memory.layout.root_address())),
            _ => unreachable!(),
        };
        assert_eq!(fault.vector, vector);
        assert_eq!(fault.error, error);
        if let Some(address) = address {
            assert_eq!(fault.address, address);
        }
        if mode == 5 {
            assert_eq!(fault.instruction, arch::USER_STACK_TOP - 8);
        } else {
            assert!((arch::USER_CODE..arch::USER_DATA).contains(&fault.instruction));
        }
        assert_eq!(reports[1].exit, Some(Exit::Returned(0)));
        assert_eq!(reports[0].completion_order, 1);
        assert_eq!(reports[1].completion_order, 2);
        for report in &reports {
            assert!(report.preemptions > 0, "no user preemption: {report:?}");
            assert_eq!(report.yields, 1);
            assert_eq!(report.rejected, 6);
        }
        assert_eq!(reports[1].writes, 3);
        admission_frames = reports.iter().map(|report| report.frames).sum();
        writeln!(console, "Cathedral Rust lab: user fault contained mode={mode} vector={vector} error={error:#x}; peer exited after fault").ok();
        drop(reports);
        assert_eq!(memory.frames.allocated(), frames);
        // SAFETY: Session and its returned reports have been destroyed.
        assert_eq!(unsafe { HEAP.used() }, heap);
    }
    writeln!(
        console,
        "Cathedral Rust lab: ring3 private memory preemption and checked syscalls passed"
    )
    .ok();
    // Fail at every frame request, including admission of the second task after
    // the first already owns a complete root. Neither payload may run on failure.
    for budget in 0..admission_frames {
        let programs = [
            Program {
                executable: Executable::Probe(code),
                arguments: [0, memory.layout.heap_start],
            },
            Program {
                executable: Executable::Probe(code),
                arguments: [0, memory.layout.heap_start],
            },
        ];
        // SAFETY: Exclusive boot context; failures must leave no published context.
        let result = unsafe {
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
            result,
            Err(user_tasks::Error::Memory(arch::MemoryError::OutOfFrames))
        ));
        assert_eq!(memory.frames.allocated(), frames);
        // SAFETY: Failed admission has returned all heap allocations.
        assert_eq!(unsafe { HEAP.used() }, heap);
    }
    writeln!(console, "Cathedral Rust lab: user admission rollback passed {admission_frames} frame boundaries; heap and frames reclaimed").ok();
}
