// Hang recovery policy and independent progress observer, all in userspace.
use cathedral_kernel_contracts::user as abi;
use cathedral_user_runtime::{
    Error,
    connection::Client,
    ipc::Handle,
    server::{Server, SharedSlot},
    task::{Child, Port},
    time,
};

pub const ROUNDS: u64 = 8;

pub fn client() -> u64 {
    static mut CONTROL: SharedSlot = SharedSlot::empty();
    let slot = &raw mut CONTROL;
    let mut owner = Server::open_clients(0).unwrap();
    // SAFETY: One server exclusively owns this process-local slot.
    owner.enable_shared_control(unsafe { &mut *slot }).unwrap();
    let port = Port::bootstrap().unwrap();
    let mut previous: Option<(Handle, Handle)> = None;
    for round in 0..ROUNDS {
        let command = owner.next_request().unwrap();
        assert_eq!(command.len, 16);
        assert_eq!(word(&command.bytes[..8]), round);
        let stolen = Child::from_raw(word(&command.bytes[8..16]));
        assert_eq!(stolen.cancel(), Err(Error(abi::BAD_HANDLE as i64)));
        if let Some((send, receive)) = previous {
            assert_eq!(send.send(b"stale"), Err(Error(abi::BAD_HANDLE as i64)));
            assert_eq!(
                receive.receive(&mut [0; 64]),
                Err(Error(abi::BAD_HANDLE as i64))
            );
        }
        let pair = port.connect().unwrap();
        let mut service = Client::connect(pair, time::after(100).unwrap()).unwrap();
        let (message, len) = service.call_retained(&round.to_le_bytes()).unwrap();
        assert_eq!(len, 8);
        assert_eq!(word(&message[..8]), round);
        let (message, len) = service.call_retained(b"prepare").unwrap();
        assert_eq!(&message[..len], b"ready");
        owner.reply(&command, b"ready").unwrap();
        let wait = owner.next_request().unwrap();
        assert_eq!(&wait.bytes[..wait.len], b"await-closure");
        // Retain both this owner request and one service call until cancellation.
        // The service consumes "hang" but deliberately never replies to it.
        assert_eq!(
            service.call_retained(b"hang"),
            Err(Error(abi::PEER_CLOSED as i64))
        );
        owner.reply(&wait, b"closed").unwrap();
        previous = Some(pair);
    }
    0
}

pub fn service(round: u64) -> u64 {
    static mut CONTROL: SharedSlot = SharedSlot::empty();
    let slot = &raw mut CONTROL;
    let mut client = Server::open_clients(0).unwrap();
    // SAFETY: One server exclusively owns this incarnation's fresh slot.
    client.enable_shared_control(unsafe { &mut *slot }).unwrap();
    let request = client.next_request().unwrap();
    assert_eq!(request.len, 8);
    assert_eq!(word(&request.bytes[..8]), round);
    client.reply(&request, &request.bytes[..8]).unwrap();
    let request = client.next_request().unwrap();
    assert_eq!(&request.bytes[..request.len], b"prepare");
    client.reply(&request, b"ready").unwrap();
    let hung = client.next_request().unwrap();
    assert_eq!(&hung.bytes[..hung.len], b"hang");
    if round.is_multiple_of(2) {
        client.next_request().unwrap(); // Client deliberately sends nothing more.
        253
    } else {
        // SAFETY: Deliberate non-yielding userspace hang. Only timer preemption and
        // authorized kernel cancellation can stop this loop; no syscall is issued.
        unsafe {
            core::arch::asm!("2:", "pause", "jmp 2b", options(noreturn));
        }
    }
}

pub fn observer() -> u64 {
    static mut CONTROL: SharedSlot = SharedSlot::empty();
    let slot = &raw mut CONTROL;
    let mut owner = Server::open_clients(0).unwrap();
    // SAFETY: One server exclusively owns this process-local slot.
    owner.enable_shared_control(unsafe { &mut *slot }).unwrap();
    let mut previous = time::now().unwrap();
    for _ in 0..ROUNDS {
        let request = owner.next_request().unwrap();
        assert_eq!(request.len, 8);
        let deadline = word(&request.bytes[..8]);
        let start = time::now().unwrap();
        assert!(time::reached(start, previous));
        assert!(!time::reached(start, deadline));
        owner.reply(&request, &start.to_le_bytes()).unwrap();
        let mut iterations = 0u64;
        let end = loop {
            let now = time::now().unwrap();
            assert!(time::reached(now, previous));
            previous = now;
            iterations += 1;
            if time::reached(now, deadline) {
                break now;
            }
        };
        let mut report = [0; 16];
        report[..8].copy_from_slice(&end.to_le_bytes());
        report[8..].copy_from_slice(&iterations.to_le_bytes());
        let finish = owner.next_request().unwrap();
        assert_eq!(&finish.bytes[..finish.len], b"finish");
        owner.reply(&finish, &report).unwrap();
    }
    0
}
fn word(bytes: &[u8]) -> u64 {
    u64::from_le_bytes(bytes.try_into().unwrap())
}
