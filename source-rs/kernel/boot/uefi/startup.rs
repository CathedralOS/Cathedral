//! Host-selected ELF handoff. The stock executable/IPC route is documented in
//! source-rs/distribution/README.md and selected by distribution/profile.json.
//! Admit a supplied initial program and its bounded launch authority.
//! Executable selection and resource requests come from host composition.
use crate::console::Console as SerialPort;
use crate::{diagnostics::user_output, memory::PreparedMemory};
use cathedral_core::user_tasks::{self, Config, Executable, Program, Supervision};
use core::fmt::Write;

static INITIAL: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/initial.elf"));
struct InitialLaunch {
    elf: &'static [u8],
    framebuffer: bool,
    keyboard: bool,
    disk: bool,
    clock: bool,
    private_pages: usize,
    shared_pages: usize,
    argument: u64,
}
include!(concat!(env!("OUT_DIR"), "/startup_config.rs"));

pub fn run(memory: &mut PreparedMemory, console: &mut SerialPort) {
    let mut memory_grants = alloc::vec::Vec::new();
    memory_grants.push(cathedral_kernel_contracts::memory::Grant {
        task: 0,
        private_pages: INITIAL_PAGES[0],
        shared_pages: INITIAL_PAGES[1],
    });
    let mut launches = alloc::vec::Vec::new();
    let mut clock_readers = alloc::vec::Vec::new();
    if INITIAL_CLOCK {
        clock_readers.push(0);
    }
    for (index, grant) in LAUNCHES.iter().enumerate() {
        memory_grants.push(cathedral_kernel_contracts::memory::Grant {
            task: index + 1,
            private_pages: grant.private_pages,
            shared_pages: grant.shared_pages,
        });
        if grant.clock {
            clock_readers.push(index + 1);
        }
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
            disk: grant.disk,
            device: None,
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
        user_tasks::run_configured(
            &mut memory.frames,
            &memory.layout,
            memory.image,
            &[Program {
                executable: Executable::Elf(INITIAL),
                arguments: [0, 0],
            }],
            user_output,
            Config {
                executables: EXECUTABLES,
                dynamic_tasks: DYNAMIC_TASKS,
                links: LINKS,
                frame_limit: usize::MAX,
                endpoints: &[],
                memory: &memory_grants,
                channel_limits: CHANNEL_LIMITS,
                cpu_limits: CPU_LIMITS,
                clock_readers: &clock_readers,
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
