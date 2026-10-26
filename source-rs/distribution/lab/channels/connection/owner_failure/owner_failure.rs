// Owner transport failure must not retire independent admitted service clients.
use super::*;
use cathedral_user_runtime::{memory::shared::Writer, notification};

pub(super) fn run(launch: Launch) {
    resources::assign(
        launch,
        Limits {
            private_pages: 4,
            reader_pages: 4,
            channels: 8,
        },
    )
    .unwrap();
    let baseline = resources::budget().unwrap();
    let physical = resources::physical_budget().unwrap();
    let notifications = notification::budget().unwrap();
    for failure in 0..9 {
        scenario(launch, failure);
        assert_eq!(resources::budget().unwrap(), baseline);
        assert_eq!(resources::physical_budget().unwrap(), physical);
        assert_eq!(notification::budget().unwrap(), notifications);
    }
    cathedral_user_runtime::write(b"Cathedral shared owner isolation: setup, expiry, pending replies, replacement, independent service and exact refunds passed\n").unwrap();
}

fn call(client: &mut Client, message: &[u8]) -> ([u8; 64], usize) {
    client.call(message, time::after(600).unwrap()).unwrap()
}

fn scenario(launch: Launch, failure: usize) {
    let (send, input) = ipc::channel().unwrap();
    let (output, receive) = ipc::channel().unwrap();
    let child = launch.spawn_with(103, &[input, output]).unwrap();
    let owner = Port::bootstrap().unwrap().connect().unwrap();
    let mut data = Client::connect((send, receive), time::after(300).unwrap()).unwrap();
    assert_eq!(call(&mut data, b"state").0[0], 1);
    match failure {
        0 => {
            owner.0.close().unwrap();
            owner.1.close().unwrap();
        }
        1..=4 => {
            owner
                .0
                .send(if failure == 1 {
                    wire::OPEN_PIPELINE
                } else {
                    wire::OPEN
                })
                .unwrap();
            let mut bytes = [0; 64];
            if failure != 1 {
                let n = owner
                    .1
                    .receive_until(&mut bytes, time::after(300).unwrap())
                    .unwrap();
                assert_eq!(&bytes[..n], wire::ACK);
                assert_eq!(call(&mut data, b"state").0[0], 1);
                if failure == 2 {
                    owner.0.send(b"not a writer grant").unwrap();
                } else {
                    let writer = Writer::allocate(1).unwrap();
                    writer.delegate(owner.0).unwrap();
                    let n = owner
                        .1
                        .receive_until(&mut bytes, time::after(300).unwrap())
                        .unwrap();
                    assert_eq!(&bytes[..n], wire::ACK);
                    // The ACK proves the receiver retained the backing before
                    // a malformed second grant or setup expiry retires it.
                    if failure == 3 {
                        owner.0.send(b"not a notification").unwrap();
                    }
                    assert_eq!(
                        owner.1.receive_until(&mut bytes, time::after(600).unwrap()),
                        Err(Error(abi::PEER_CLOSED as i64))
                    );
                    drop(writer);
                }
            }
            if failure <= 2 {
                assert_eq!(
                    owner.1.receive_until(&mut bytes, time::after(600).unwrap()),
                    Err(Error(abi::PEER_CLOSED as i64))
                );
            }
            owner.0.close().unwrap();
            owner.1.close().unwrap();
        }
        5..=7 => {
            let mut control = Client::connect(owner, time::after(300).unwrap()).unwrap();
            assert_eq!(&call(&mut control, b"echo").0[..4], b"echo");
            if failure >= 6 {
                control
                    .begin(if failure == 6 { b"hold" } else { b"hold-reply" }, None)
                    .unwrap();
                // Two independently scheduled streams: wait for actual acceptance.
                let until = time::after(300).unwrap();
                while call(&mut data, b"held").0[0] == 0 {
                    assert!(!time::reached(time::now().unwrap(), until));
                }
            }
            control.retire();
        }
        8 => {
            owner.0.send(b"hold-reply").unwrap();
            let until = time::after(300).unwrap();
            while call(&mut data, b"held").0[0] == 0 {
                assert!(!time::reached(time::now().unwrap(), until));
            }
            owner.0.close().unwrap();
            owner.1.close().unwrap();
        }
        _ => unreachable!(),
    }
    if failure >= 7 {
        assert_eq!(&call(&mut data, b"reply-now").0[..9], b"reply-now");
    }
    let until = time::after(300).unwrap();
    while call(&mut data, b"state").0[0] != 0 {
        assert!(!time::reached(time::now().unwrap(), until));
    }
    for _ in 0..8 {
        assert_eq!(&call(&mut data, b"echo").0[..4], b"echo");
    }
    assert_eq!(&call(&mut data, b"finish").0[..6], b"finish");
    assert_eq!(
        child.wait_until(time::after(300).unwrap()).unwrap(),
        Outcome::Returned(0)
    );
    data.retire();
}
