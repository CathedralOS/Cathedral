//! Admit a supplied initial program and its bounded launch authority.
//! Executable selection and resource requests come from host composition.
use crate::{diagnostics::user_output, memory::PreparedMemory};
use cathedral_core::users::{self, Config, Executable, Program, Supervision};
use cathedral_uart_16550::SerialPort;
use core::fmt::Write;

static INITIAL: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/initial.elf"));
struct InitialLaunch {
    elf: &'static [u8],
    framebuffer: bool,
    keyboard: bool,
    argument: u64,
}
include!(concat!(env!("OUT_DIR"), "/startup_config.rs"));

pub fn run(memory: &mut PreparedMemory, console: &mut SerialPort) {
    let mut launches = alloc::vec::Vec::new();
    for grant in LAUNCHES {
        let framebuffer = if grant.framebuffer {
            let Some(framebuffer) = memory.framebuffer else {
                writeln!(
                    console,
                    "Cathedral kernel: initial program requires unavailable framebuffer"
                )
                .ok();
                return;
            };
            Some(framebuffer)
        } else {
            None
        };
        launches.push(Supervision {
            owner: 0,
            peer: 0,
            framebuffer,
            keyboard: grant.keyboard,
            program: Program {
                executable: Executable::Elf(grant.elf),
                arguments: [grant.argument, 0],
            },
            frame_limit: usize::MAX,
        });
    }
    writeln!(
        console,
        "Cathedral kernel: starting supplied initial program"
    )
    .ok();
    // SAFETY: Sole boot CPU, IRQs off, owned image and entry paths; allocator is
    // reclaiming. Optional device aperture is reserved and held exclusively.
    let result = unsafe {
        users::run_configured(
            &mut memory.frames,
            &memory.layout,
            memory.image,
            &[Program {
                executable: Executable::Elf(INITIAL),
                arguments: [0, 0],
            }],
            user_output,
            Config {
                frame_limit: usize::MAX,
                endpoints: &[],
                clock_readers: &[],
                supervision: &launches,
            },
        )
    };
    match result {
        Ok(reports) => {
            writeln!(
                console,
                "Cathedral kernel: initial program stopped: {:?}",
                reports[0].exit
            )
            .ok();
        }
        Err(error) => {
            writeln!(
                console,
                "Cathedral kernel: initial program admission failed: {error:?}"
            )
            .ok();
        }
    }
}
