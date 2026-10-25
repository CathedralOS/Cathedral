//! Shared admission, preemption, IPC, memory-object lifetime and failed-admission rollback.
use crate::{console::Console, diagnostics::user_output, memory::PreparedMemory};
use cathedral_arch as arch;
use cathedral_core::{
    ipc::EndpointSpec,
    user_tasks::{self, Config, Executable, Exit, Program},
};
use cathedral_kernel_contracts::memory::MAX_PAGES;
use core::fmt::Write;
static ELF: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/arm64.elf"));
pub(super) fn exchange(
    memory: &mut PreparedMemory,
    console: &mut Console,
    baseline: usize,
    heap: usize,
) -> usize {
    let endpoints = endpoints();

    let mut admission = 0;
    for round in 0..2 {
        let programs = [program(0, 0), program(1, 0)];
        let reports = run(memory, &programs, &endpoints, usize::MAX).unwrap();
        for (index, report) in reports.iter().enumerate() {
            assert_eq!(report.exit, Some(Exit::Returned(0)));
            // One shared-view and two notification introductions precede the
            // 32 echoes. The client moves/uses a local endpoint (two messages)
            // and separately delegates/accepts a notification (one message).
            let messages = if index == 1 { 38 } else { 35 };
            assert_eq!(report.ipc_sent, messages);
            assert_eq!(report.ipc_received, messages);
        }
        assert!(reports.iter().any(|r| r.receives_blocked > 0));
        admission = reports.iter().map(|r| r.frames).sum();
        drop(reports);
        reclaimed(memory, baseline, heap);
        writeln!(console,"Cathedral Rust lab: ARM64 IPC round={round}; 32 echoes and 32 notification round trips, local rights, stale handles, reclamation passed").ok();
    }
    admission
}

pub(super) fn isolation(
    memory: &mut PreparedMemory,
    console: &mut Console,
    baseline: usize,
    heap: usize,
) {
    let reports = run(
        memory,
        &[program(3, 0x5a), program(3, 0xa5)],
        &[],
        usize::MAX,
    )
    .unwrap();
    for report in &reports {
        assert_eq!(report.exit, Some(Exit::Returned(0)));
        assert!(
            report.preemptions > 0,
            "timer did not switch EL0 peers: {report:?}"
        );
        assert_eq!(report.memory.allocated_pages, MAX_PAGES);
    }
    drop(reports);
    reclaimed(memory, baseline, heap);
    writeln!(
        console,
        "Cathedral Rust lab: ARM64 timer preempted both private EL0 peers after incremental maximum-size allocations"
    )
    .ok();
    for (role, address, ec) in [
        (2, memory.layout.heap_start, 0x24),
        (2, memory.layout.root_address(), 0x24),
        (2, 0x0900_0000, 0x24),
        (4, user_tasks::address_space::USER_CODE, 0x24),
        (2, user_tasks::address_space::USER_STACK - 8, 0x24),
        (5, user_tasks::address_space::USER_STACK, 0x20),
    ] {
        let reports = run(
            memory,
            &[program(role, address), program(3, 0)],
            &[],
            usize::MAX,
        )
        .unwrap();
        let Some(Exit::Fault(fault)) = reports[0].exit else {
            panic!("offender escaped: {:?}", reports[0]);
        };
        assert_eq!(fault.vector, ec);
        assert_eq!(fault.address, address);
        assert_eq!(reports[1].exit, Some(Exit::Returned(0)));
        assert_eq!(reports[1].memory.allocated_pages, MAX_PAGES);
        drop(reports);
        reclaimed(memory, baseline, heap);
        writeln!(console,"Cathedral Rust lab: ARM64 fault contained role={role} address={address:#x}; peer and private pages survived").ok();
    }
}

pub(super) fn rollback(
    memory: &mut PreparedMemory,
    console: &mut Console,
    baseline: usize,
    heap: usize,
    admission: usize,
) {
    let endpoints = endpoints();
    let mut wrong_cpu = [0; 64];
    wrong_cpu.copy_from_slice(&ELF[..64]);
    wrong_cpu[18..20].copy_from_slice(&62u16.to_le_bytes());
    assert!(matches!(
        user_tasks::admission::elf::parse(&wrong_cpu),
        Err(user_tasks::admission::elf::Error::Header)
    ));
    // SAFETY: Boot owns the live layout and reclaiming allocator outside a session.
    unsafe {
        user_tasks::exercise_memory_rollback(
            &mut memory.frames,
            &memory.layout,
            memory.image,
            memory.framebuffer,
            user_output,
        );
    }
    reclaimed(memory, baseline, heap);
    for budget in 0..admission {
        let result = run(memory, &[program(0, 0), program(1, 0)], &endpoints, budget);
        assert!(matches!(
            result,
            Err(user_tasks::Error::Memory(arch::MemoryError::OutOfFrames))
        ));
        drop(result);
        reclaimed(memory, baseline, heap);
    }
    writeln!(
        console,
        "Cathedral Rust lab: ARM64 admission rollback passed {admission} frame boundaries"
    )
    .ok();
}
pub(super) fn program(role: u64, address: u64) -> Program<'static> {
    Program {
        executable: Executable::Elf(ELF),
        arguments: [role, address],
    }
}
fn run(
    memory: &mut PreparedMemory,
    programs: &[Program<'_>],
    endpoints: &[EndpointSpec],
    budget: usize,
) -> Result<alloc::vec::Vec<user_tasks::Report>, user_tasks::Error> {
    let grants = [0, 1].map(|task| cathedral_kernel_contracts::memory::Grant {
        task,
        private_pages: MAX_PAGES,
        shared_pages: 2,
    });
    // SAFETY: Sole CPU, IRQs masked, owned initialized layout and reclaiming allocator.
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
                frame_limit: budget,
                endpoints,
                supervision: &[],
                memory: &grants,
                channel_limits: &[],
                cpu_limits: &[],
                clock_readers: &[0, 1],
            },
        )
    }
}
pub(super) fn reclaimed(memory: &PreparedMemory, frames: usize, heap: usize) {
    assert_eq!(memory.frames.allocated(), frames);
    // SAFETY: Session and reports destroyed, sole heap owner.
    assert_eq!(unsafe { crate::heap::HEAP.used() }, heap);
}

fn endpoints() -> [EndpointSpec; 2] {
    [
        EndpointSpec {
            sender: 1,
            receiver: 0,
            revoker: None,
        },
        EndpointSpec {
            sender: 0,
            receiver: 1,
            revoker: None,
        },
    ]
}
