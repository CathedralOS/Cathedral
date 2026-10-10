use cathedral_contracts::user as abi;
use cathedral_user_runtime::{
    Error,
    ipc::Handle,
    task::{Child, Launch, Outcome},
};

pub fn run() -> u64 {
    let launch = Launch::bootstrap().unwrap();
    let commands = Handle::bootstrap(0).unwrap();
    let replies = Handle::bootstrap(1).unwrap();
    assert_eq!(
        Child::from_raw(0).wait(),
        Err(Error(abi::BAD_HANDLE as i64))
    );
    let mut previous = None;
    for round in 0..32u64 {
        let child = launch.spawn(round).unwrap();
        assert_eq!(launch.spawn(0).unwrap_err(), Error(abi::BUSY as i64));
        if let Some(stale) = previous {
            assert_eq!(
                Child::from_raw(stale).wait(),
                Err(Error(abi::BAD_HANDLE as i64))
            );
        }
        // Bad destinations must neither block nor consume the child's outcome.
        super::probes::bad_wait(child);
        let mut command = [0; 24];
        command[..8].copy_from_slice(&round.to_le_bytes());
        command[8..16].copy_from_slice(&launch.raw().to_le_bytes());
        command[16..].copy_from_slice(&child.raw().to_le_bytes());
        commands.send(&command).unwrap();
        assert!(matches!(
            child.wait().unwrap(),
            Outcome::Fault {
                vector: 6,
                error: 0,
                ..
            }
        ));
        assert_eq!(child.wait(), Err(Error(abi::BAD_HANDLE as i64)));
        assert_eq!(replies.receive(&mut [0; 64]).unwrap(), 1);
        previous = Some(child.raw());
    }
    0
}
