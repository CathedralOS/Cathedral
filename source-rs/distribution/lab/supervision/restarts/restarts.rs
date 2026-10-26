// Repeated service faults preserve a live client; invalid waits never consume outcomes.
use cathedral_kernel_contracts::user as abi;
use cathedral_user_runtime::{
    Error,
    connection::Client,
    event,
    ipc::Handle,
    task::{Child, Launch, Outcome},
    time, write,
};

pub fn run() -> u64 {
    let launch = Launch::bootstrap().unwrap();
    let mut client = Client::connect(
        (Handle::bootstrap(0).unwrap(), Handle::bootstrap(1).unwrap()),
        time::after(200).unwrap(),
    )
    .unwrap();
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
        super::failures::bad_wait(child);
        let mut command = [0; 24];
        command[..8].copy_from_slice(&round.to_le_bytes());
        command[8..16].copy_from_slice(&launch.raw().to_le_bytes());
        command[16..].copy_from_slice(&child.raw().to_le_bytes());
        // Publish without waiting for the client reply: task wait must still
        // block until the service faults while the independent client progresses.
        client.begin(&command, None).unwrap();
        assert!(matches!(
            child.wait().unwrap(),
            Outcome::Fault {
                vector: 6,
                error: 0,
                ..
            }
        ));
        assert_eq!(child.wait(), Err(Error(abi::BAD_HANDLE as i64)));
        let deadline = time::after(100).unwrap();
        loop {
            if let Some((bytes, len, object)) = client.poll().unwrap() {
                assert!(object.is_none());
                assert_eq!(&bytes[..len], b"!");
                break;
            }
            event::wait(&client.interests().unwrap(), deadline).unwrap();
        }
        previous = Some(child.raw());
    }
    client.retire();
    write(b"Cathedral supervision: 32 shared service incarnations completed\n").unwrap();
    0
}
