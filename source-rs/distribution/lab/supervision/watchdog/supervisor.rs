use super::{ROUNDS, probes, word};
use cathedral_kernel_contracts::user as abi;
use cathedral_user_runtime::{
    Error,
    connection::Client,
    event,
    ipc::Handle,
    task::{Child, Launch, Outcome},
    time,
};

pub fn run() -> u64 {
    probes::clock();
    let launch = Launch::bootstrap().unwrap();
    let mut client = Client::connect(
        (Handle::bootstrap(0).unwrap(), Handle::bootstrap(1).unwrap()),
        time::after(200).unwrap(),
    )
    .unwrap();
    let mut observer = Client::connect(
        (Handle::bootstrap(2).unwrap(), Handle::bootstrap(3).unwrap()),
        time::after(200).unwrap(),
    )
    .unwrap();
    let mut stale = None;
    for round in 0..ROUNDS {
        let child = launch.spawn(round).unwrap();
        if let Some(ticket) = stale {
            assert_eq!(
                Child::from_raw(ticket).cancel(),
                Err(Error(abi::BAD_HANDLE as i64))
            );
        }
        let mut command = [0; 16];
        command[..8].copy_from_slice(&round.to_le_bytes());
        command[8..].copy_from_slice(&child.raw().to_le_bytes());
        let (reply, len) = client.call_retained(&command).unwrap();
        assert_eq!(&reply[..len], b"ready");
        client.begin(b"await-closure", None).unwrap();
        let past = time::now().unwrap().wrapping_sub(1);
        assert_eq!(child.wait_until(past), Err(Error(abi::TIMED_OUT as i64)));
        assert_eq!(launch.spawn(0).unwrap_err(), Error(abi::BUSY as i64));

        let deadline = time::after(12).unwrap();
        let (reply, len) = observer.call_retained(&deadline.to_le_bytes()).unwrap();
        assert_eq!(len, 8);
        assert!(!time::reached(word(&reply[..8]), deadline));
        assert_eq!(
            child.wait_until(deadline),
            Err(Error(abi::TIMED_OUT as i64))
        );
        assert!(time::reached(time::now().unwrap(), deadline));
        // Wait for independent progress before cancelling: this observation was
        // produced while the hung child was still alive, not after recovery.
        let (reply, len) = observer.call_retained(b"finish").unwrap();
        assert_eq!(len, 16);
        assert!(time::reached(word(&reply[..8]), deadline));
        assert!(word(&reply[8..16]) >= 2);
        child.cancel().unwrap();
        child.cancel().unwrap(); // Idempotent while the reaped outcome remains held.
        assert_eq!(child.wait().unwrap(), Outcome::Cancelled);
        assert_eq!(child.cancel(), Err(Error(abi::BAD_HANDLE as i64)));
        let deadline = time::after(100).unwrap();
        loop {
            if let Some((reply, len, object)) = client.poll().unwrap() {
                assert!(object.is_none());
                assert_eq!(&reply[..len], b"closed");
                break;
            }
            event::wait(&client.interests().unwrap(), deadline).unwrap();
        }
        stale = Some(child.raw());
    }
    client.retire();
    observer.retire();
    cathedral_user_runtime::write(
        b"Cathedral supervision: shared watchdog recovered 8 hung service calls\n",
    )
    .unwrap();
    0
}
