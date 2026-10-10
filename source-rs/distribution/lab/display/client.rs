use cathedral_contracts::{display as wire, user as abi};
use cathedral_user_runtime::{
    Error,
    ipc::Handle,
    task::{Child, Port},
};

pub fn run() -> u64 {
    let commands = Handle::bootstrap(0).unwrap();
    let status = Handle::bootstrap(1).unwrap();
    let port = Port::bootstrap().unwrap();
    let mut stale: Option<(Handle, Handle)> = None;
    for round in 0..2 {
        let mut bytes = [0; 64];
        assert_eq!(commands.receive(&mut bytes).unwrap(), 8);
        let stolen = Child::from_raw(u64::from_le_bytes(bytes[..8].try_into().unwrap()));
        assert_eq!(stolen.cancel(), Err(Error(abi::DENIED as i64)));
        if let Some((send, receive)) = stale {
            assert_eq!(send.send(b"stale"), Err(Error(abi::BAD_HANDLE as i64)));
            assert_eq!(
                receive.receive(&mut bytes),
                Err(Error(abi::BAD_HANDLE as i64))
            );
        }
        let (send, receive) = port.connect().unwrap();
        let info = call(send, receive, [wire::INFO, 0, 0, 0, 0, 0]);
        assert_eq!(info[0], 0);
        let (w, h) = (info[1], info[2]);
        assert!(w >= 640 && h >= 480);
        send.send(b"malformed").unwrap();
        assert_eq!(receive.receive(&mut bytes).unwrap(), wire::REQUEST_BYTES);
        assert_eq!(
            wire::decode(&bytes[..wire::REQUEST_BYTES]).unwrap()[0],
            abi::INVALID_ARGUMENT
        );
        assert_eq!(
            call(send, receive, [wire::RECT, u64::MAX, 0, 2, 1, 0])[0],
            abi::INVALID_ARGUMENT
        );
        cathedral_boot_scene::draw(send, receive, w, h).unwrap();
        send.send(&wire::encode([wire::INFO, 0, 0, 0, 0, 0]))
            .unwrap();
        if round == 0 {
            assert_eq!(
                receive.receive(&mut bytes),
                Err(Error(abi::PEER_CLOSED as i64))
            );
        } else {
            assert_eq!(receive.receive(&mut bytes).unwrap(), wire::REQUEST_BYTES);
            assert_eq!(wire::decode(&bytes[..wire::REQUEST_BYTES]).unwrap(), info);
        }
        status.send(&[round]).unwrap();
        stale = Some((send, receive));
    }
    assert_eq!(commands.receive(&mut [0; 64]).unwrap(), 4);
    0
}
fn call(send: Handle, receive: Handle, words: [u64; 6]) -> [u64; 6] {
    send.send(&wire::encode(words)).unwrap();
    let mut bytes = [0; 64];
    assert_eq!(receive.receive(&mut bytes).unwrap(), wire::REQUEST_BYTES);
    wire::decode(&bytes[..wire::REQUEST_BYTES]).unwrap()
}
