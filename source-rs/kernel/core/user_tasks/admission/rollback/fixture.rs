//! Shared inactive roots, retained IPC rights and exact refund assertions.
use super::*;

pub(super) struct Fixture<'a> {
    pub(super) session: alloc::boxed::Box<Session>,
    pub(super) source: Frames<'a>,
    pub(super) baseline: usize,
    pub(super) admitted: usize,
    pub(super) producer: u64,
}

#[inline(never)]
pub(super) unsafe fn setup<'a>(
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
    let (mut session, _) = startup::prepare(2, &config, baseline, |_| true).unwrap();
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

pub(super) unsafe fn discard_draft(f: &mut Fixture<'_>) {
    loop {
        let before = f.source.allocated();
        // SAFETY: Private draft has never run, and retains its frame/source custody.
        let done = unsafe { abort_step(&mut f.session, &mut f.source, 0, 8) };
        assert!(before - f.source.allocated() <= 8);
        if done {
            break;
        }
        unpublished(f);
    }
}

pub(super) unsafe fn reclaim(f: &mut Fixture<'_>) {
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

pub(super) fn unpublished(f: &Fixture<'_>) {
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
pub(super) fn restored(f: &Fixture<'_>) {
    assert!(f.session.launches[0].draft.is_none());
    assert_eq!(f.source.allocated(), f.admitted);
    assert_eq!(f.session.memory.funding.snapshot(2)[1], 0);
}
pub(super) unsafe fn finish(mut f: Fixture<'_>) {
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

pub(super) fn executable() -> alloc::vec::Vec<u8> {
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
