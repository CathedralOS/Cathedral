//! Host-supplied driver ELF; kernel asserts authority, IRQ and reclamation outcomes.
use crate::{console::Console, diagnostics::user_output, memory::PreparedMemory};
use cathedral_core::user_tasks::{self, Config, Executable, Exit, Program, Supervision};
use core::fmt::Write;
static ELF: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/devices.elf"));
pub fn exercise(memory: &mut PreparedMemory, console: &mut Console) {
    let baseline = memory.frames.allocated();
    // SAFETY: Outside a session, sole boot CPU owns heap.
    let heap = unsafe { crate::heap::HEAP.used() };
    for _ in 0..2 {
        let programs = [Program {
            executable: Executable::Elf(ELF),
            arguments: [0, 0],
        }];
        let launch = [Supervision {
            device: Some(cathedral_arch::board::DeviceId::AuxiliarySerial),
            disk: false,
            keyboard: false,
            framebuffer: None,
            owner: 0,
            peer: 0,
            program: Program {
                executable: Executable::Elf(ELF),
                arguments: [1, 0],
            },
            frame_limit: usize::MAX,
        }];
        // SAFETY: Sole CPU, IRQs off, exclusive COM2/IRQ3; harness instantiates UART.
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
                    channel_limits: &[],
                    cpu_limits: &[],
                    memory: &[],
                    frame_limit: usize::MAX,
                    links: &[],
                    endpoints: &[],
                    supervision: &launch,
                    clock_readers: &[0, 1],
                },
            )
        }
        .unwrap();
        assert_eq!(reports[0].exit, Some(Exit::Returned(0)));
        assert_eq!(reports[0].spawned, 5);
        assert_eq!(reports[0].reaped, 5);
        assert_eq!(reports[0].cancelled_blocked, 1);
        assert_eq!(reports[1].exit, Some(Exit::Returned(0)));
        assert!(reports[1].device_interrupts >= 16);
        assert!(reports[1].readiness_blocked >= 2);
        assert!(reports[1].wait_timeouts >= 2);
        drop(reports);
        assert_eq!(memory.frames.allocated(), baseline);
        // SAFETY: Session ended; all device/process state reclaimed.
        assert_eq!(unsafe { crate::heap::HEAP.used() }, heap);
    }
    writeln!(console, "Cathedral Rust lab: device capabilities, masked IRQ delivery and driver replacement reclaimed all state").ok();
}
