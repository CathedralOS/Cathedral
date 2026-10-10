use super::{ROUNDS, probes, word};
use cathedral_contracts::user as abi;
use cathedral_user_runtime::{
    Error,
    ipc::Handle,
    task::{Child, Launch, Outcome},
    time,
};

pub fn run() -> u64 {
    probes::clock();
    let launch = Launch::bootstrap().unwrap();
    let commands = Handle::bootstrap(0).unwrap();
    let replies = Handle::bootstrap(1).unwrap();
    let observe = Handle::bootstrap(2).unwrap();
    let progress = Handle::bootstrap(3).unwrap();
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
        commands.send(&command).unwrap();
        let mut reply = [0; 64];
        assert_eq!(replies.receive(&mut reply).unwrap(), 5);
        let past = time::now().unwrap().wrapping_sub(1);
        assert_eq!(child.wait_until(past), Err(Error(abi::TIMED_OUT as i64)));
        assert_eq!(launch.spawn(0).unwrap_err(), Error(abi::BUSY as i64));

        let deadline = time::after(12).unwrap();
        observe.send(&deadline.to_le_bytes()).unwrap();
        assert_eq!(progress.receive(&mut reply).unwrap(), 8);
        assert!(!time::reached(word(&reply[..8]), deadline));
        assert_eq!(
            child.wait_until(deadline),
            Err(Error(abi::TIMED_OUT as i64))
        );
        assert!(time::reached(time::now().unwrap(), deadline));
        // Wait for independent progress before cancelling: this observation was
        // produced while the hung child was still alive, not after recovery.
        assert_eq!(progress.receive(&mut reply).unwrap(), 16);
        assert!(time::reached(word(&reply[..8]), deadline));
        assert!(word(&reply[8..16]) >= 2);
        child.cancel().unwrap();
        child.cancel().unwrap(); // Idempotent while the reaped outcome remains held.
        assert_eq!(child.wait().unwrap(), Outcome::Cancelled);
        assert_eq!(child.cancel(), Err(Error(abi::BAD_HANDLE as i64)));
        assert_eq!(replies.receive(&mut reply).unwrap(), 6);
        assert_eq!(&reply[..6], b"closed");
        stale = Some(child.raw());
    }
    0
}
