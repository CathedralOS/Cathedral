use cathedral_contracts::user as abi;
use cathedral_user_runtime::{
    Error,
    ipc::Handle,
    task::{Child, Launch, Port},
};

pub fn run() -> u64 {
    assert_eq!(Launch::bootstrap().unwrap_err(), Error(abi::DENIED as i64));
    let commands = Handle::bootstrap(0).unwrap();
    let replies = Handle::bootstrap(1).unwrap();
    let port = Port::bootstrap().unwrap();
    let mut previous: Option<(Handle, Handle)> = None;
    for round in 0..32u64 {
        let mut command = [0; 64];
        assert_eq!(commands.receive(&mut command).unwrap(), 24);
        assert_eq!(u64::from_le_bytes(command[..8].try_into().unwrap()), round);
        let stolen = u64::from_le_bytes(command[8..16].try_into().unwrap());
        assert_eq!(
            Launch::from_raw(stolen).spawn(0).unwrap_err(),
            Error(abi::DENIED as i64)
        );
        let stolen = u64::from_le_bytes(command[16..24].try_into().unwrap());
        assert_eq!(
            Child::from_raw(stolen).wait(),
            Err(Error(abi::DENIED as i64))
        );
        // New service grants stay hidden until this client explicitly accepts.
        assert_eq!(
            Handle::bootstrap(2).unwrap_err(),
            Error(abi::BAD_HANDLE as i64)
        );
        if let Some((send, receive)) = previous {
            assert_eq!(send.send(b"stale"), Err(Error(abi::BAD_HANDLE as i64)));
            assert_eq!(
                receive.receive(&mut [0; 64]),
                Err(Error(abi::BAD_HANDLE as i64))
            );
        }
        let (send, receive) = port.connect().unwrap();
        assert_eq!(port.connect().unwrap_err(), Error(abi::BUSY as i64));
        for sequence in 0..4 {
            let mut message = [sequence; 64];
            message[..8].copy_from_slice(&round.to_le_bytes());
            send.send(&message).unwrap();
            let mut response = [0; 64];
            assert_eq!(receive.receive(&mut response).unwrap(), 64);
            assert_eq!(message, response);
        }
        send.send(b"crash").unwrap();
        assert_eq!(
            receive.receive(&mut [0; 64]),
            Err(Error(abi::PEER_CLOSED as i64))
        );
        assert_eq!(send.send(b"closed"), Err(Error(abi::PEER_CLOSED as i64)));
        replies.send(b"!").unwrap();
        previous = Some((send, receive));
    }
    0
}
