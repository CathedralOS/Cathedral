// A four-child graph survives replacing either endpoint and complete teardown.
use cathedral_kernel_contracts::user as abi;
use cathedral_user_runtime::{
    Error,
    connection::Client,
    ipc::Handle,
    link::Link,
    server::{Server, SharedSlot},
    task::{Launch, Outcome, Port},
    time,
};
fn exchange(client: &mut Client, value: u8) {
    let (bytes, len) = client.call_retained(&[value]).unwrap();
    assert_eq!(&bytes[..len], &[value]);
}
pub fn owner() -> u64 {
    assert_eq!(Link::at(0).unwrap_err(), Error(abi::DENIED as i64));
    let launches = [
        Launch::at(0).unwrap(),
        Launch::at(1).unwrap(),
        Launch::at(2).unwrap(),
        Launch::at(3).unwrap(),
    ];
    let ports = [
        Port::at(0).unwrap(),
        Port::at(1).unwrap(),
        Port::at(2).unwrap(),
        Port::at(3).unwrap(),
    ];
    let mut children = launches.map(|launch| launch.spawn(0).unwrap());
    let mut pairs = ports.map(|port| port.connect().unwrap());
    let mut clients = pairs.map(|pair| Client::connect(pair, time::after(100).unwrap()).unwrap());
    let mut names: [[u64; 2]; 4] =
        core::array::from_fn(|i| names::inspect(&mut clients[i], [0; 2]));
    exchange(&mut clients[2], 0);
    for generation in 1..=16 {
        let index = if generation % 2 == 1 {
            2
        } else {
            [0, 1, 3][(generation / 2) % 3]
        };
        let old = children[index];
        let retired = pairs[index];
        old.cancel().unwrap();
        assert_eq!(old.wait().unwrap(), Outcome::Cancelled);
        clients[index].retire();
        children[index] = launches[index].spawn(generation as u64).unwrap();
        pairs[index] = ports[index].connect().unwrap();
        clients[index] = Client::connect(pairs[index], time::after(100).unwrap()).unwrap();
        names[index] = names::inspect(&mut clients[index], names[index]);
        for other in 0..pairs.len() {
            assert_eq!(names::inspect(&mut clients[other], [0; 2]), names[other]);
        }
        assert_eq!(retired.0.send(b"stale"), Err(Error(abi::BAD_HANDLE as i64)));
        for index in [0, 1, 3] {
            exchange(&mut clients[index], 7);
        }
        exchange(
            &mut clients[2],
            match index {
                2 => 0,
                3 => 3,
                _ => index as u8 + 1,
            },
        );
    }
    cathedral_user_runtime::write(b"Cathedral: local clock/link names survive unrelated restarts and reject prior incarnations\n").unwrap();
    cathedral_user_runtime::write(
        b"Cathedral supervision: shared graph survived 16 app/provider replacements\n",
    )
    .unwrap();
    // Exit with all children live; kernel must reclaim their roots, queues and graph.
    // Do not close transport early through Rust drop and change that proof.
    core::mem::forget(clients);
    0
}
pub fn provider() -> u64 {
    static mut CONTROL: SharedSlot = SharedSlot::empty();
    static mut APPLICATION: SharedSlot = SharedSlot::empty();
    let control = &raw mut CONTROL;
    let application = &raw mut APPLICATION;
    let mut server = Server::open().unwrap();
    // SAFETY: One server exclusively owns these two distinct process-local slots.
    server
        .enable_shared_control(unsafe { &mut *control })
        .unwrap();
    // SAFETY: Distinct application slot, exclusively borrowed by this server.
    server
        .enable_shared(0, unsafe { &mut *application })
        .unwrap();
    loop {
        let request = server.next_request().unwrap();
        if request.control
            && let Some(reply) = names::probe(&request.bytes[..request.len])
        {
            server.reply(&request, &reply).unwrap();
            continue;
        }
        server
            .reply(&request, &request.bytes[..request.len])
            .unwrap();
    }
}
pub fn application() -> u64 {
    assert_eq!(Launch::at(0).unwrap_err(), Error(abi::DENIED as i64));
    static mut CONTROL: SharedSlot = SharedSlot::empty();
    let slot = &raw mut CONTROL;
    let mut owner = Server::open_clients(0).unwrap();
    // SAFETY: Single entry; only this server borrows its process-local slot.
    owner.enable_shared_control(unsafe { &mut *slot }).unwrap();
    let links = [
        Link::at(0).unwrap(),
        Link::at(1).unwrap(),
        Link::at(2).unwrap(),
    ];
    let mut connections: Option<[(Client, (Handle, Handle)); 3]> = None;
    loop {
        let request = owner.next_request().unwrap();
        if let Some(reply) = names::probe(&request.bytes[..request.len]) {
            owner.reply(&request, &reply).unwrap();
            continue;
        }
        assert_eq!(request.len, 1);
        let command = request.bytes[0];
        // Admission can run this app before the owner's last provider spawn.
        // The first control request is the explicit all-providers-live barrier.
        let clients = connections.get_or_insert_with(|| {
            links.map(|link| {
                let pair = link.connect().unwrap();
                (
                    Client::connect(pair, time::after(100).unwrap()).unwrap(),
                    pair,
                )
            })
        });
        if command != 0 {
            let index = usize::from(command - 1);
            assert_eq!(
                clients[index].1.0.send(b"stale"),
                Err(Error(abi::BAD_HANDLE as i64))
            );
            clients[index].0.retire();
            let pair = links[index].connect().unwrap();
            clients[index] = (
                Client::connect(pair, time::after(100).unwrap()).unwrap(),
                pair,
            );
        }
        for (client, _) in clients {
            let (bytes, length) = client.call(b"graph", time::after(100).unwrap()).unwrap();
            assert_eq!(&bytes[..length], b"graph");
        }
        owner.reply(&request, &[command]).unwrap();
    }
}
