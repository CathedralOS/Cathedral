//! Hang recovery policy and independent progress observer, all in userspace.
pub mod probes;
pub mod supervisor;
use cathedral_contracts::user as abi;
use cathedral_user_runtime::{
    Error,
    ipc::Handle,
    task::{Child, Port},
    time,
};

pub const ROUNDS: u64 = 8;

pub fn client() -> u64 {
    assert_eq!(time::now(), Err(Error(abi::DENIED as i64)));
    let command = Handle::bootstrap(0).unwrap();
    let status = Handle::bootstrap(1).unwrap();
    let port = Port::bootstrap().unwrap();
    let mut previous: Option<(Handle, Handle)> = None;
    for round in 0..ROUNDS {
        let mut message = [0; 64];
        assert_eq!(command.receive(&mut message).unwrap(), 16);
        assert_eq!(word(&message[..8]), round);
        let stolen = Child::from_raw(word(&message[8..16]));
        assert_eq!(stolen.cancel(), Err(Error(abi::DENIED as i64)));
        if let Some((send, receive)) = previous {
            assert_eq!(send.send(b"stale"), Err(Error(abi::BAD_HANDLE as i64)));
            assert_eq!(
                receive.receive(&mut message),
                Err(Error(abi::BAD_HANDLE as i64))
            );
        }
        let (send, receive) = port.connect().unwrap();
        send.send(&round.to_le_bytes()).unwrap();
        assert_eq!(receive.receive(&mut message).unwrap(), 8);
        assert_eq!(word(&message[..8]), round);
        send.send(b"hang").unwrap();
        assert_eq!(receive.receive(&mut message).unwrap(), 5);
        assert_eq!(&message[..5], b"ready");
        status.send(b"ready").unwrap();
        assert_eq!(
            receive.receive(&mut message),
            Err(Error(abi::PEER_CLOSED as i64))
        );
        status.send(b"closed").unwrap();
        previous = Some((send, receive));
    }
    0
}

pub fn service(round: u64) -> u64 {
    assert_eq!(time::now(), Err(Error(abi::DENIED as i64)));
    let input = Handle::bootstrap(0).unwrap();
    let output = Handle::bootstrap(1).unwrap();
    let mut message = [0; 64];
    assert_eq!(input.receive(&mut message).unwrap(), 8);
    assert_eq!(word(&message[..8]), round);
    output.send(&message[..8]).unwrap();
    assert_eq!(input.receive(&mut message).unwrap(), 4);
    output.send(b"ready").unwrap();
    if round.is_multiple_of(2) {
        input.receive(&mut message).unwrap(); // Client deliberately sends nothing more.
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
    let commands = Handle::bootstrap(0).unwrap();
    let reports = Handle::bootstrap(1).unwrap();
    let mut previous = time::now().unwrap();
    for _ in 0..ROUNDS {
        let mut message = [0; 64];
        assert_eq!(commands.receive(&mut message).unwrap(), 8);
        let deadline = word(&message[..8]);
        let start = time::now().unwrap();
        assert!(time::reached(start, previous));
        assert!(!time::reached(start, deadline));
        reports.send(&start.to_le_bytes()).unwrap();
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
        reports.send(&report).unwrap();
    }
    0
}
fn word(bytes: &[u8]) -> u64 {
    u64::from_le_bytes(bytes.try_into().unwrap())
}
