use cathedral_kernel_contracts::user as abi;
use cathedral_user_runtime::{
    Error,
    connection::Client,
    ipc::Handle,
    server::{Server, SharedSlot},
    task::{Child, Launch, Port},
    time,
};

pub fn run() -> u64 {
    assert_eq!(Launch::bootstrap().unwrap_err(), Error(abi::DENIED as i64));
    static mut CONTROL: SharedSlot = SharedSlot::empty();
    let slot = &raw mut CONTROL;
    let mut owner = Server::open_clients(0).unwrap();
    // SAFETY: One entry in this process; this server exclusively owns its slot.
    owner.enable_shared_control(unsafe { &mut *slot }).unwrap();
    let port = Port::bootstrap().unwrap();
    let mut previous: Option<(Handle, Handle)> = None;
    for round in 0..32u64 {
        let request = owner.next_request().unwrap();
        assert_eq!(request.len, 24);
        let command = request.bytes;
        assert_eq!(u64::from_le_bytes(command[..8].try_into().unwrap()), round);
        let stolen = u64::from_le_bytes(command[8..16].try_into().unwrap());
        assert_eq!(
            Launch::from_raw(stolen).spawn(0).unwrap_err(),
            Error(abi::BAD_HANDLE as i64)
        );
        let stolen = u64::from_le_bytes(command[16..24].try_into().unwrap());
        assert_eq!(
            Child::from_raw(stolen).wait(),
            Err(Error(abi::BAD_HANDLE as i64))
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
        let baseline = transport_usage();
        let pair = port.connect().unwrap();
        assert_eq!(port.connect().unwrap_err(), Error(abi::BUSY as i64));
        let mut service = Client::connect(pair, time::after(100).unwrap()).unwrap();
        for sequence in 0..4 {
            let mut message = [sequence; 64];
            message[..8].copy_from_slice(&round.to_le_bytes());
            let (response, len) = service.call_retained(&message).unwrap();
            assert_eq!(len, 64);
            assert_eq!(message, response);
        }
        assert_eq!(
            service.call_retained(b"crash"),
            Err(Error(abi::PEER_CLOSED as i64))
        );
        // Terminal failure retires endpoint/backing custody. No new request can
        // reuse that connection, even after a replacement service is admitted.
        assert_eq!(
            service.begin(b"closed", None),
            Err(Error(abi::PEER_CLOSED as i64))
        );
        await_reclamation(baseline);
        owner.reply(&request, b"!").unwrap();
        previous = Some(pair);
    }
    0
}

fn transport_usage() -> [u64; 5] {
    let used = cathedral_user_runtime::resources::budget().unwrap().used;
    let notifications = cathedral_user_runtime::notification::budget().unwrap();
    [
        used.private_pages,
        used.reader_pages,
        used.channels,
        notifications[1],
        notifications[3],
    ]
}

fn await_reclamation(baseline: [u64; 5]) {
    // Observing closure can precede teardown's final reader release. Wait for
    // actual discharged custody, never refund on the mere fault notification.
    let deadline = time::after(100).unwrap();
    while transport_usage() != baseline {
        assert!(
            !time::reached(time::now().unwrap(), deadline),
            "faulted transport leaked"
        );
        cathedral_user_runtime::yield_now();
    }
}
