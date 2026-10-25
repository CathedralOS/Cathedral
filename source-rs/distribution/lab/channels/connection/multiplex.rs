//! Independent streams share dispatch, not authority, IDs or partial custody.
use super::*;
use cathedral_user_runtime::server::{Event, Request, Server, SharedSlot};

pub(super) fn run(launch: Launch) {
    // The ordinary root owns four mailbox writers before temporary publications.
    // Exercise both the new headroom and the still-bounded metadata allowance.
    {
        use cathedral_user_runtime::memory::{Private, shared::Writer};
        let writers: [Writer; 4] = core::array::from_fn(|_| Writer::allocate(1).unwrap());
        let scratch = Private::allocate(1).unwrap().freeze().unwrap();
        let remaining: [_; cathedral_kernel_contracts::memory::MAX_PRIVATE_REGIONS - 5] =
            core::array::from_fn(|_| Private::allocate(1).unwrap());
        assert!(matches!(Private::allocate(1),Err(Error(c)) if c == abi::NO_MEMORY as i64));
        drop((writers, scratch, remaining));
    }
    for mode in 71..=72 {
        let (a, ai) = ipc::channel().unwrap();
        let (ao, ar) = ipc::channel().unwrap();
        let (b, bi) = ipc::channel().unwrap();
        let (bo, br) = ipc::channel().unwrap();
        let child = launch.spawn_with(mode, &[ai, ao, bi, bo]).unwrap();
        let pair = Port::bootstrap().unwrap().connect().unwrap();
        let mut control = Client::connect(pair, time::after(300).unwrap()).unwrap();
        let mut first = Client::connect((a, ar), time::after(300).unwrap()).unwrap();
        if mode == 71 {
            let mut second = Client::connect((b, br), time::after(300).unwrap()).unwrap();
            // Equal transport IDs on distinct streams; responses deliberately reverse.
            first.begin(b"first", None).unwrap();
            second.begin(b"second", None).unwrap();
            assert_eq!(
                &control
                    .call(b"release", time::after(300).unwrap())
                    .unwrap()
                    .0[..2],
                b"ok"
            );
            let x = finish(&mut first);
            let y = finish(&mut second);
            assert_eq!(&x[..5], b"first");
            assert_eq!(&y[..6], b"second");
            second.retire();
        } else {
            b.send(wire::OPEN).unwrap();
            let mut bytes = [0; 64];
            assert_eq!(br.receive(&mut bytes).unwrap(), wire::ACK.len());
            // Incomplete setup on the second stream must not block either peer.
            assert_eq!(
                &first.call(b"first", time::after(300).unwrap()).unwrap().0[..5],
                b"first"
            );
            assert_eq!(
                &control
                    .call(b"wait-gone", time::after(600).unwrap())
                    .unwrap()
                    .0[..4],
                b"gone"
            );
            b.close().unwrap();
            br.close().unwrap();
        }
        for _ in 0..16 {
            assert_eq!(
                &first.call(b"alive", time::after(300).unwrap()).unwrap().0[..5],
                b"alive"
            );
        }
        assert_eq!(
            &control.call(b"stop", time::after(300).unwrap()).unwrap().0[..4],
            b"stop"
        );
        first.retire();
        control.retire();
        assert_eq!(
            child.wait_until(time::after(300).unwrap()).unwrap(),
            Outcome::Returned(0)
        );
    }
    cathedral_user_runtime::write(b"Cathedral shared multiplex: independent control/clients, reversed replies, partial expiry and surviving peer passed\n").unwrap();
}
fn finish(client: &mut Client) -> [u8; 64] {
    loop {
        if let Some((bytes, _, object)) = client.poll().unwrap() {
            assert!(object.is_none());
            return bytes;
        }
        event::wait(&client.interests().unwrap(), time::after(300).unwrap()).unwrap();
    }
}
pub(super) fn worker(mode: u64, env: Environment) -> u64 {
    // A full endpoint set cannot also reserve a notification. Refusal happens
    // before transport setup and leaves the same endpoints usable below.
    let mut full = Server::open_clients(abi::MAX_WAIT_HANDLES - 1).unwrap();
    full.install(0, (env.endpoint(0).unwrap(), env.endpoint(1).unwrap()))
        .unwrap();
    static mut REFUSED: SharedSlot = SharedSlot::empty();
    let refused = &raw mut REFUSED;
    // SAFETY: A distinct process-owned slot, offered only to this bounded probe.
    let refused = unsafe { &mut *refused };
    assert_eq!(
        full.enable_shared(0, refused),
        Err(Error(abi::INVALID_ARGUMENT as i64))
    );
    drop(full);
    let mut server = Server::open_clients(2).unwrap();
    static mut SLOTS: [SharedSlot; 3] = [const { SharedSlot::empty() }; 3];
    let slots = &raw mut SLOTS;
    // SAFETY: Single-entry process; each disjoint slot is lent only once.
    let [control, a, b] = unsafe { &mut *slots };
    server.enable_shared_control(control).unwrap();
    for (index, slot) in [a, b].into_iter().enumerate() {
        server
            .install(
                index,
                (
                    env.endpoint(index * 2).unwrap(),
                    env.endpoint(index * 2 + 1).unwrap(),
                ),
            )
            .unwrap();
        server.enable_shared(index, slot).unwrap();
    }
    let mut held: [Option<Request>; 2] = [None; 2];
    let mut release = None;
    let mut gone = false;
    loop {
        match server.next_event().unwrap() {
            Event::Request(r) if r.control => match &r.bytes[..r.len] {
                b"stop" => {
                    server.reply(&r, b"stop").unwrap();
                    return 0;
                }
                b"release" | b"wait-gone" => release = Some(r),
                _ => panic!("unexpected control"),
            },
            Event::Request(r) => {
                let index = r.client.unwrap();
                if mode == 71 && r.bytes[0] != b'a' {
                    assert_eq!(r.bytes[0], if index == 0 { b'f' } else { b's' });
                    held[index] = Some(r);
                } else {
                    server.reply(&r, &r.bytes[..r.len]).unwrap();
                }
            }
            Event::Disconnected { client, .. } => {
                assert_eq!(client, 1);
                gone = true;
            }
            _ => panic!("unexpected attachment"),
        }
        if let Some(r) = release {
            if mode == 71 && held.iter().all(Option::is_some) {
                for pending in held.iter_mut().rev() {
                    let p = pending.take().unwrap();
                    server.reply(&p, &p.bytes[..p.len]).unwrap();
                }
                server.reply(&r, b"ok").unwrap();
                release = None;
            } else if mode == 72 && gone {
                server.reply(&r, b"gone").unwrap();
                release = None;
            }
        }
    }
}
