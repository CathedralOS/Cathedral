// Real Rust clients stress an Omega service with independent clients and owner.
use cathedral_kernel_contracts::user as abi;
use cathedral_platform_contracts::{connection as wire, matrix};
use cathedral_user_runtime::{
    connection::Client,
    ipc,
    memory::Private,
    notification, resources,
    task::{Launch, Outcome, Port},
    time,
};

pub(super) fn run() {
    for mode in 200..=212 {
        let budget = resources::budget().unwrap();
        let physical = resources::physical_budget().unwrap();
        let bells = notification::budget().unwrap();
        scenario(mode);
        assert_eq!(resources::budget().unwrap(), budget, "service mode {mode}");
        assert_eq!(resources::physical_budget().unwrap(), physical);
        assert_eq!(notification::budget().unwrap(), bells);
    }
    cathedral_user_runtime::write(b"Omega multi-client service PASS\n").unwrap();
}
fn scenario(mode: u64) {
    let (send_a, input_a) = ipc::channel().unwrap();
    let (output_a, recv_a) = ipc::channel().unwrap();
    let (send_b, input_b) = ipc::channel().unwrap();
    let (output_b, recv_b) = ipc::channel().unwrap();
    let child = Launch::at(1)
        .unwrap()
        .spawn_with(mode, &[input_a, output_a, input_b, output_b])
        .unwrap();
    // OPEN without its reader: B and the parent must still finish setup and work.
    let mut raw = None;
    let mut held = None;
    let mut backing = None;
    if mode == 200 {
        send_a.send(wire::OPEN).unwrap();
    } else if mode == 201 || mode == 210 {
        let mut client = Client::connect((send_a, recv_a), time::after(300).unwrap()).unwrap();
        if mode == 201 {
            let page = Private::allocate(1).unwrap().freeze().unwrap();
            client
                .begin(&matrix::encode([10, 0, 0, 0, 0, 0, 0, 0]), Some(&page))
                .unwrap();
            backing = Some(page);
        } else {
            for value in 1..=16 {
                image(&mut client, 1, value);
            }
        }
        held = Some(client);
    } else {
        raw = Some(raw::Peer::open(send_a, recv_a));
    }
    let mut other = Client::connect((send_b, recv_b), time::after(300).unwrap()).unwrap();
    let mut owner = Client::connect(
        Port::at(1).unwrap().connect().unwrap(),
        time::after(300).unwrap(),
    )
    .unwrap();
    if let Some(peer) = &mut raw {
        let page = Private::allocate(1).unwrap().freeze().unwrap();
        match mode {
            202 => peer.grant(1, &page),
            203 => peer.frame(1, wire::Attachment::Memory),
            204 => {
                peer.grant(2, &page);
                peer.frame(1, wire::Attachment::Memory);
            }
            205 => {
                peer.grant(1, &page);
                observed(&mut owner, |v| v[5] != 0);
                peer.grant(1, &page);
            }
            206 => {
                peer.grant(1, &page);
                peer.frame(1, wire::Attachment::None);
            }
            207 => {
                peer.send.send(b"no copied request fallback").unwrap();
            }
            208 => peer.notification(),
            209 => peer.grant(0, &page),
            211 => {
                peer.frame(1, wire::Attachment::Memory);
                observed(&mut owner, |v| v[6] == 1);
                peer.grant(1, &page);
                peer.reply(1);
            }
            212 => {
                peer.grant(1, &page);
                observed(&mut owner, |v| v[5] != 0);
                peer.frame(1, wire::Attachment::Memory);
                peer.reply(1);
            }
            _ => unreachable!(),
        }
        backing = Some(page);
    }
    if mode >= 211 {
        drop(raw.take());
    }
    if mode == 201 {
        observed(&mut owner, |v| v[2] != 0 && v[1] != 0);
    }
    // Same request IDs on distinct clients, independent data, and owner progress.
    for value in 17..=24 {
        image(&mut other, 1, value);
        let state = status(&mut owner, false);
        assert!(state[3] != 0 && state[4] != 0);
        if mode == 201 {
            assert_ne!(state[1], 0);
        }
    }
    if let Some(client) = &mut held {
        client.retire();
    }
    let expected = if mode == 200 || mode == 202 || mode == 203 {
        abi::TIMED_OUT
    } else if mode == 201 || mode >= 210 {
        abi::PEER_CLOSED
    } else {
        abi::IO_ERROR
    };
    let retired = observed(&mut owner, |v| v[0] != 0);
    assert_eq!(retired[0], expected, "retirement mode {mode}");
    assert_eq!(retired[1], 0);
    assert_eq!(retired[2], 0);
    drop(raw);
    drop(held);
    drop(backing);
    if mode == 200 {
        send_a.close().unwrap();
        recv_a.close().unwrap();
    }
    // Only five reader pages are funded. B + owner use two; this three-page
    // transfer cannot succeed if A's partial/held reader leaked while alive.
    for value in 25..=28 {
        image(&mut other, 3, value);
    }
    status(&mut owner, true);
    assert_eq!(
        child.wait_until(time::after(300).unwrap()).unwrap(),
        Outcome::Returned(0)
    );
    other.retire();
    owner.retire();
}
fn image(client: &mut Client, pages: usize, value: u8) {
    let mut backing = Private::allocate(pages).unwrap();
    backing.bytes_mut().fill(value);
    let object = backing.freeze().unwrap();
    let bytes = matrix::encode([11, value as u64, pages as u64, 0, 0, 0, 0, 0]);
    let (reply, n, received) = client
        .call_memory(&bytes, Some(&object), time::after(300).unwrap())
        .unwrap();
    assert_eq!(n, 64);
    assert_eq!(reply, bytes);
    drop(object);
    let mut received = received.expect("sealed reply backing");
    assert!(received.map().unwrap().iter().all(|&byte| byte == value));
}
fn status(owner: &mut Client, stop: bool) -> [u64; 8] {
    let bytes = matrix::encode([if stop { 99 } else { 0 }, 0, 0, 0, 0, 0, 0, 0]);
    let (reply, n) = owner.call(&bytes, time::after(300).unwrap()).unwrap();
    matrix::decode(&reply[..n]).unwrap()
}
fn observed(owner: &mut Client, check: impl Fn([u64; 8]) -> bool) -> [u64; 8] {
    let deadline = time::after(600).unwrap();
    loop {
        let state = status(owner, false);
        if check(state) {
            return state;
        }
        assert!(
            !time::reached(time::now().unwrap(), deadline),
            "service observation expired: {state:?}"
        );
        let _ = cathedral_user_runtime::event::wait(&[], time::after(1).unwrap());
    }
}
