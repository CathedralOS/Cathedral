use cathedral_contracts::user as abi;
use cathedral_user_runtime::{
    Error,
    ipc::Handle,
    task::{Child, Launch, Outcome},
    time,
};
pub fn run() -> u64 {
    let launch = Launch::bootstrap().unwrap();
    let commands = Handle::bootstrap(0).unwrap();
    let replies = Handle::bootstrap(1).unwrap();
    let observer = Handle::bootstrap(2).unwrap();
    let progress = Handle::bootstrap(3).unwrap();
    let mut stale = None;
    for round in 0..2 {
        let child = launch.spawn(round).unwrap();
        if let Some(ticket) = stale {
            assert_eq!(
                Child::from_raw(ticket).cancel(),
                Err(Error(abi::BAD_HANDLE as i64))
            );
        }
        let deadline = time::after(200).unwrap();
        observer.send(&deadline.to_le_bytes()).unwrap();
        let mut bytes = [0; 64];
        assert_eq!(progress.receive(&mut bytes).unwrap(), 8);
        commands.send(&child.raw().to_le_bytes()).unwrap();
        assert_eq!(replies.receive(&mut bytes).unwrap(), 1);
        assert_eq!(bytes[0], round as u8);
        if round == 0 {
            assert!(matches!(
                child.wait_until(deadline).unwrap(),
                Outcome::Fault { vector: 6, .. }
            ));
        } else {
            child.cancel().unwrap();
            assert_eq!(child.wait().unwrap(), Outcome::Cancelled);
        }
        assert!(!time::reached(time::now().unwrap(), deadline));
        assert_eq!(progress.receive(&mut bytes).unwrap(), 8);
        assert!(u64::from_le_bytes(bytes[..8].try_into().unwrap()) > 1);
        stale = Some(child.raw());
    }
    commands.send(b"stop").unwrap();
    0
}
