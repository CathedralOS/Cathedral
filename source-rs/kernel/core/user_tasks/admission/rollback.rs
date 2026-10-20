//! Real-frame staged admission regressions, shared by the x86 and ARM64 smoke labs.
use super::*;
use crate::user_tasks::address_space::{USER_CODE, USER_DATA};
use crate::user_tasks::execution::deferred::Work;
use crate::user_tasks::{Config, Supervision, lifecycle, startup};
use crate::{
    ipc::{EndpointSpec, Right},
    memory::FrameAllocator,
    scheduler::TaskState,
};
use cathedral_kernel_contracts::{memory as wire, user as abi};

struct Fixture<'a> {
    session: alloc::boxed::Box<Session>,
    source: Frames<'a>,
    baseline: usize,
    admitted: usize,
    producer: u64,
}

/// # Safety
/// Sole boot CPU, inactive user machinery, initialized heap and live kernel layout.
#[inline(never)]
pub(in crate::user_tasks) unsafe fn exercise(
    frames: &mut FrameAllocator,
    layout: &arch::BootLayout,
    image: arch::ImageRange,
    output: fn(&[u8]) -> bool,
) {
    output(b"staged admission: physical rollback\n");
    let bytes = executable();
    let program = Program {
        executable: Executable::Elf(&bytes),
        arguments: [0, 7],
    };
    // SAFETY: No fixture root is ever executed; all frames are privately held here.
    unsafe {
        physical(frames, layout, image, &program, output);
        output(b"staged admission: interrupted authority\n");
        for death in 0..3 {
            interrupted(frames, layout, image, &program, death);
        }
        output(b"staged admission: abandoned source\n");
        abandoned_object(frames, layout, image, &program);
        output(b"staged admission: rollback, cancellation and source reclamation passed\n");
    }
}

#[inline(never)]
unsafe fn setup<'a>(
    frames: &'a mut FrameAllocator,
    layout: &arch::BootLayout,
    image: arch::ImageRange,
    program: &Program<'_>,
) -> Fixture<'a> {
    let baseline = frames.allocated();
    let launches = [Supervision {
        owner: 0,
        peer: 1,
        disk: false,
        device: None,
        keyboard: false,
        framebuffer: None,
        program: Program {
            executable: program.executable,
            arguments: program.arguments,
        },
        frame_limit: usize::MAX,
    }];
    let config = Config {
        executables: &[],
        dynamic_tasks: 0,
        frame_limit: usize::MAX,
        endpoints: &[EndpointSpec {
            sender: 0,
            receiver: 1,
            revoker: Some(1),
        }],
        links: &[],
        supervision: &launches,
        channel_limits: &[],
        cpu_limits: &[],
        clock_readers: &[],
        memory: &[
            wire::Grant {
                task: 0,
                private_pages: 2,
                shared_pages: 0,
            },
            wire::Grant {
                task: 2,
                private_pages: 1,
                shared_pages: 0,
            },
        ],
    };
    let (session, _) = startup::prepare(2, &config, baseline, |_| true).unwrap();
    // Keep callers' retained sessions off the boot stack: returning or retiring
    // several by-value Session temporaries can exhaust either ISA's guarded stack.
    let mut session = alloc::boxed::Box::new(session);
    let mut source = Frames::new(frames, session.memory.funding.capacity(), usize::MAX).unwrap();
    let initial = Program {
        executable: Executable::Probe(&[0xcc]),
        arguments: [0, 0],
    };
    // SAFETY: Inactive machine, serialized initial construction.
    unsafe {
        admit(&mut session, layout, image, &initial, &mut source, 0).unwrap();
        admit(&mut session, layout, image, &initial, &mut source, 1).unwrap();
    }
    let grant = session.launches[0].model.launch(0).unwrap();
    session.launches[0].model.request(0, grant, 7).unwrap();
    let [send, receive] = session.ipc.create(0).unwrap();
    let transport = session.ipc.handle(0, 0).unwrap();
    session
        .ipc
        .send_cap(0, transport, b"producer", Some(send))
        .unwrap();
    let transport = session.ipc.handle(1, 0).unwrap();
    let producer = session
        .ipc
        .receive_cap(1, transport, 64, true)
        .unwrap()
        .unwrap()
        .attachment
        .unwrap();
    session.launches[0].initial[0] = receive;
    session.launches[0].initial_count = 1;
    let admitted = source.allocated();
    Fixture {
        session,
        source,
        baseline,
        admitted,
        producer,
    }
}

#[inline(never)]
unsafe fn physical(
    frames: &mut FrameAllocator,
    layout: &arch::BootLayout,
    image: arch::ImageRange,
    program: &Program<'_>,
    output: fn(&[u8]) -> bool,
) {
    // SAFETY: Exclusive machine/frame custody for this entire fixture.
    let mut f = unsafe { setup(frames, layout, image, program) };
    let mut steps = 0;
    loop {
        steps += 1;
        // SAFETY: Immutable ELF retained; child remains outside runnable context storage.
        let ready =
            unsafe { advance(&mut f.session, layout, image, program, &mut f.source, 0) }.unwrap();
        unpublished(&f);
        if ready {
            break;
        }
        assert!(steps < 64);
    }
    assert!(steps > 8); // 64 image pages + stack, not just a tiny probe.
    let required = f.source.allocated() - f.admitted;
    // SAFETY: Even a complete draft is private until final authority commit.
    unsafe {
        abort(&mut f.session, &mut f.source, 0);
    }
    restored(&f);
    for limit in 0..required {
        f.session.launches[0].frame_limit = limit;
        loop {
            // SAFETY: Same inactive roots; deterministic failure at every frame boundary.
            match unsafe { advance(&mut f.session, layout, image, program, &mut f.source, 0) } {
                Ok(false) => unpublished(&f),
                Ok(true) => panic!("admission exceeded total frame limit"),
                Err(Error::Memory(arch::MemoryError::OutOfFrames)) => break,
                Err(other) => panic!("unexpected refusal: {other:?}"),
            }
        }
        restored(&f);
    }
    output(b"staged admission: frame refusals passed\n");
    f.session.launches[0].frame_limit = usize::MAX;
    for boundary in 1..=steps {
        for _ in 0..boundary {
            // SAFETY: Retained source, same private child root.
            unsafe { advance(&mut f.session, layout, image, program, &mut f.source, 0) }.unwrap();
            unpublished(&f);
        }
        // SAFETY: Exercise withdrawal at every coherent stage, including fully built.
        unsafe {
            abort(&mut f.session, &mut f.source, 0);
        }
        restored(&f);
    }
    output(b"staged admission: withdrawals passed\n");
    let old = f.session.launches[0].initial[0];
    while f.session.launches[0].model.pending().is_some() {
        f.session.deferred.remove(Work::Admission, 0);
        // SAFETY: Exercise the actual lifecycle commit with inactive parent/peer roots.
        unsafe {
            lifecycle::spawn(&mut f.session, &mut f.source, layout, image, program, 0);
        }
    }
    assert_eq!(f.session.tasks[0].report.spawned, 1);
    assert_eq!(f.session.scheduler.states()[2], TaskState::Ready);
    assert_eq!(
        f.session.ipc.check(0, old, Right::Receive),
        Err(abi::BAD_HANDLE)
    );
    let space = f.session.tasks[2].space.as_ref().unwrap();
    let mut byte = [0];
    // SAFETY: Live inactive committed root; copies validate the complete ranges.
    unsafe {
        assert!(space.copy_from_user(USER_CODE, &mut byte));
        assert_eq!(byte, [0xc3]);
        for page in 0..63 {
            let mut tail = [1; 32];
            assert!(space.copy_from_user(USER_DATA + page * 4096 + 4064, &mut tail));
            assert_eq!(tail, [0; 32]);
        }
    }
    assert!(!space.writable_range(USER_CODE, 1));
    assert!(space.writable_range(USER_DATA, 1));
    f.session.scheduler.cancel(0);
    f.session.tasks[0].report.exit = Some(crate::user_tasks::Exit::Cancelled);
    lifecycle::close(&mut f.session, 0);
    let mut batches = 0;
    while f.session.deferred.reclaiming() {
        let before = f.source.allocated();
        // SAFETY: Production cancellation withdraws the child before the first yield.
        unsafe {
            lifecycle::reap(&mut f.session, &mut f.source);
        }
        batches += 1;
        assert!(batches < 256);
        assert!(before - f.source.allocated() <= 8);
        assert_eq!(f.session.scheduler.states()[1], TaskState::Ready);
        assert_eq!(
            f.session.ipc.send(1, f.producer, b"late"),
            Err(abi::PEER_CLOSED)
        );
        if f.session.deferred.reclaiming() {
            // Neither the parent nor child slot/outcome is reusable mid-wave.
            assert_eq!(f.session.scheduler.states()[0], TaskState::Exited);
            assert_eq!(f.session.scheduler.states()[2], TaskState::Exited);
            assert_eq!(f.session.tasks[0].report.reaped, 0);
        }
    }
    assert!(batches > 8);
    assert_eq!(f.session.scheduler.states()[0], TaskState::Vacant);
    assert_eq!(f.session.scheduler.states()[2], TaskState::Vacant);
    assert_eq!(f.session.tasks[0].report.reaped, 1);
    // SAFETY: No fixture context was ever activated.
    unsafe {
        finish(f);
    }
}

#[inline(never)]
unsafe fn interrupted(
    frames: &mut FrameAllocator,
    layout: &arch::BootLayout,
    image: arch::ImageRange,
    program: &Program<'_>,
    death: usize,
) {
    // SAFETY: New isolated fixture with explicit parent and separate peer.
    let mut f = unsafe { setup(frames, layout, image, program) };
    for _ in 0..3 {
        f.session.deferred.remove(Work::Admission, 0);
        // SAFETY: Start construction through the production lifecycle, retaining authority.
        unsafe {
            lifecycle::spawn(&mut f.session, &mut f.source, layout, image, program, 0);
        }
        unpublished(&f);
    }
    let receive = f.session.launches[0].initial[0];
    if death < 2 {
        f.session.scheduler.cancel(death);
        f.session.tasks[death].report.exit = Some(crate::user_tasks::Exit::Cancelled);
        lifecycle::close(&mut f.session, death);
        // SAFETY: Exited fixture task cannot resume; exercise owner-return cleanup too.
        unsafe {
            reclaim(&mut f);
        }
    } else {
        // A receive right with queued data is no longer movable. The independent
        // producer can make it busy while its owner waits for child admission.
        f.session.ipc.send(1, f.producer, b"arrived").unwrap();
    }
    if death != 0 {
        f.session.deferred.remove(Work::Admission, 0);
        // SAFETY: A dead peer/busy initial right must refuse and abort the draft.
        unsafe {
            lifecycle::spawn(&mut f.session, &mut f.source, layout, image, program, 0);
        }
        assert_eq!(f.session.tasks[0].report.rejected, 1);
    }
    if death == 2 {
        let message = f.session.ipc.receive(0, receive, 64).unwrap().unwrap();
        assert_eq!(&message.bytes[..message.len], b"arrived");
    }
    assert!(f.session.launches[0].draft.is_none());
    assert!(f.session.tasks[2].space.is_none());
    assert_eq!(f.session.scheduler.states()[2], TaskState::Vacant);
    assert_eq!(f.session.memory.funding.snapshot(2)[1], 0);
    assert_eq!(f.session.tasks[0].report.spawned, 0);
    // SAFETY: Remaining initial task(s) are inactive and retained here.
    unsafe {
        finish(f);
    }
}

// An object-backed executable carries a separate kernel reference. If its last
// owner dies mid-build, both the draft and that source reference must disappear.
#[inline(never)]
unsafe fn abandoned_object(
    frames: &mut FrameAllocator,
    layout: &arch::BootLayout,
    image: arch::ImageRange,
    program: &Program<'_>,
) {
    // SAFETY: New inactive fixture and exclusive bank custody.
    let mut f = unsafe { setup(frames, layout, image, program) };
    f.session.tasks[0].memory_call = Some([abi::MEMORY_ALLOCATE, 2, 0]);
    f.session.deferred.insert(Work::Memory, 0);
    // SAFETY: Prepared inactive root; process owns frame/mapping edits.
    unsafe {
        crate::user_tasks::memory::process(&mut f.session, &mut f.source);
    }
    assert!(f.session.tasks[0].memory_call.is_none());
    let region = f
        .session
        .memory
        .model
        .entries
        .iter()
        .position(|entry| entry.owner == Some(0))
        .unwrap();
    let handle = f.session.memory.model.handle(0, region);
    let address = wire::address(f.session.memory.model.mapping(0, region));
    let Executable::Elf(bytes) = program.executable else {
        unreachable!()
    };
    for (page, chunk) in bytes.chunks(wire::PAGE_BYTES).enumerate() {
        // SAFETY: Each copy fits one complete writable page of this allocation.
        assert!(unsafe {
            f.session.tasks[0]
                .space
                .as_ref()
                .unwrap()
                .copy_to_user(address + (page * wire::PAGE_BYTES) as u64, chunk)
        });
    }
    f.session.tasks[0].memory_call = Some([abi::MEMORY_FREEZE, handle, 0]);
    f.session.deferred.insert(Work::Memory, 0);
    // SAFETY: No references into backing survive sealing.
    unsafe {
        crate::user_tasks::memory::process(&mut f.session, &mut f.source);
    }
    f.session.memory.hold(region);
    f.session.launches[0].code = Some(pool::Code::Object { region, role: 0 });
    let pages = [
        f.session.memory.frames_of(region)[0],
        f.session.memory.frames_of(region)[1],
    ];
    let program = Program {
        executable: Executable::Object(&pages),
        arguments: [0, 7],
    };
    for _ in 0..3 {
        f.session.deferred.remove(Work::Admission, 0);
        // SAFETY: Kernel reference retains immutable source pages across these steps.
        unsafe {
            lifecycle::spawn(&mut f.session, &mut f.source, layout, image, &program, 0);
        }
    }
    assert!(f.session.launches[0].draft.is_some());
    f.session.scheduler.cancel(0);
    f.session.tasks[0].report.exit = Some(crate::user_tasks::Exit::Cancelled);
    lifecycle::close(&mut f.session, 0);
    // SAFETY: Owner cannot resume; abort draft before settling its source reference.
    unsafe {
        reclaim(&mut f);
        crate::user_tasks::memory::process(&mut f.session, &mut f.source);
    }
    assert!(f.session.launches[0].draft.is_none());
    assert!(f.session.launches[0].code.is_none());
    assert_eq!(f.session.memory.frames(), 0);
    assert_eq!(f.session.memory.funding.snapshot(2)[1], 0);
    // SAFETY: Only the inactive initial peer remains.
    unsafe {
        finish(f);
    }
}

unsafe fn reclaim(f: &mut Fixture<'_>) {
    let mut steps = 0;
    while f.session.deferred.reclaiming() {
        steps += 1;
        assert!(steps < 256);
        // SAFETY: Fixture contexts never execute; retain all custody through collection.
        unsafe {
            lifecycle::reap(&mut f.session, &mut f.source);
        }
    }
}

fn unpublished(f: &Fixture<'_>) {
    assert!(f.session.tasks[2].space.is_none());
    assert_eq!(f.session.scheduler.states()[2], TaskState::Vacant);
    assert_eq!(f.session.ipc.handle(2, 0), Err(abi::BAD_HANDLE));
    assert_eq!(f.session.tasks[0].report.spawned, 0);
    let initial = f.session.launches[0].initial[0];
    assert!(f.session.ipc.check(0, initial, Right::Receive).is_ok());
    assert_eq!(
        f.session.memory.funding.snapshot(2)[1],
        f.source.allocated() - f.admitted
    );
}
fn restored(f: &Fixture<'_>) {
    assert!(f.session.launches[0].draft.is_none());
    assert_eq!(f.source.allocated(), f.admitted);
    assert_eq!(f.session.memory.funding.snapshot(2)[1], 0);
}
unsafe fn finish(mut f: Fixture<'_>) {
    assert!(f.session.launches[0].draft.is_none());
    for slot in 0..f.session.tasks.len() {
        if f.session.tasks[slot].space.is_some() {
            // SAFETY: Fixture roots never execute or gain borrowed page objects.
            unsafe {
                retire(&mut f.session, slot, &mut f.source);
            }
        }
    }
    assert_eq!(f.source.allocated(), f.baseline);
    assert_eq!(f.session.memory.funding.charged(), 0);
}

fn executable() -> alloc::vec::Vec<u8> {
    let mut b = alloc::vec![0; 8192];
    b[..7].copy_from_slice(b"\x7fELF\x02\x01\x01");
    for (at, value) in [
        (16, 2),
        (18, arch::ELF_MACHINE),
        (52, 64),
        (54, 56),
        (56, 2),
    ] {
        b[at..at + 2].copy_from_slice(&value.to_le_bytes());
    }
    b[20..24].copy_from_slice(&1u32.to_le_bytes());
    b[24..32].copy_from_slice(&USER_CODE.to_le_bytes());
    b[32..40].copy_from_slice(&64u64.to_le_bytes());
    for (i, flags, offset, address, file, memory) in [
        (0, 5u32, 4096u64, USER_CODE, 1u64, 4096u64),
        (1, 6u32, 8192, USER_DATA, 0, 63 * 4096),
    ] {
        let at = 64 + i * 56;
        b[at..at + 4].copy_from_slice(&1u32.to_le_bytes());
        b[at + 4..at + 8].copy_from_slice(&flags.to_le_bytes());
        for (field, value) in [
            (8, offset),
            (16, address),
            (32, file),
            (40, memory),
            (48, 4096),
        ] {
            b[at + field..at + field + 8].copy_from_slice(&value.to_le_bytes());
        }
    }
    b[4096] = 0xc3;
    b
}
