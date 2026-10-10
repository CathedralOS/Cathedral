//! A four-child graph survives replacing either endpoint and complete teardown.
use cathedral_contracts::user as abi;
use cathedral_user_runtime::{
    Error,
    ipc::Handle,
    link::Link,
    server::Server,
    task::{Launch, Outcome, Port},
    time,
};
fn exchange(pair: (Handle, Handle), value: u8) {
    pair.0.send(&[value]).unwrap();
    let mut bytes = [0; 64];
    assert_eq!(pair.1.receive(&mut bytes).unwrap(), 1);
    assert_eq!(bytes[0], value);
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
    exchange(pairs[2], 0);
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
        children[index] = launches[index].spawn(generation as u64).unwrap();
        pairs[index] = ports[index].connect().unwrap();
        assert_eq!(retired.0.send(b"stale"), Err(Error(abi::BAD_HANDLE as i64)));
        for index in [0, 1, 3] {
            exchange(pairs[index], 7);
        }
        exchange(
            pairs[2],
            match index {
                2 => 0,
                3 => 3,
                _ => index as u8 + 1,
            },
        );
    }
    // Exit with all children live; kernel must reclaim their roots, queues and graph.
    0
}
pub fn provider() -> u64 {
    let mut server = Server::open().unwrap();
    loop {
        let request = server.next_request().unwrap();
        server
            .reply(&request, &request.bytes[..request.len])
            .unwrap();
    }
}
pub fn application() -> u64 {
    assert_eq!(Launch::at(0).unwrap_err(), Error(abi::DENIED as i64));
    let input = Handle::bootstrap(0).unwrap();
    let output = Handle::bootstrap(1).unwrap();
    let links = [
        Link::at(0).unwrap(),
        Link::at(1).unwrap(),
        Link::at(2).unwrap(),
    ];
    let mut pairs = links.map(|link| link.connect().unwrap());
    loop {
        let mut bytes = [0; 64];
        assert_eq!(input.receive(&mut bytes).unwrap(), 1);
        let command = bytes[0];
        if command != 0 {
            let index = usize::from(command - 1);
            assert_eq!(
                pairs[index].0.send(b"stale"),
                Err(Error(abi::BAD_HANDLE as i64))
            );
            pairs[index] = links[index].connect().unwrap();
        }
        for pair in pairs {
            pair.0.send(b"graph").unwrap();
            let length = pair
                .1
                .receive_until(&mut bytes, time::after(100).unwrap())
                .unwrap();
            assert_eq!(&bytes[..length], b"graph");
        }
        output.send(&[command]).unwrap();
    }
}
