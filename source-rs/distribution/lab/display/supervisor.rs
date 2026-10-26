//! Two provider incarnations, with retained shared coordination and observer work.
use cathedral_kernel_contracts::user as abi;
use cathedral_user_runtime::{
    Error,
    connection::Client,
    ipc::Handle,
    task::{Child, Launch, Outcome},
    time, write,
};
pub fn run() -> u64 {
    let launch = Launch::bootstrap().unwrap();
    let mut client = Client::connect(
        (Handle::bootstrap(0).unwrap(), Handle::bootstrap(1).unwrap()),
        time::after(300).unwrap(),
    )
    .unwrap();
    let mut observer = Client::connect(
        (Handle::bootstrap(2).unwrap(), Handle::bootstrap(3).unwrap()),
        time::after(300).unwrap(),
    )
    .unwrap();
    write(b"Cathedral display: shared coordination ready\n").unwrap();
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
        let (started, len) = observer.call_retained(&deadline.to_le_bytes()).unwrap();
        assert_eq!(len, 8);
        assert!(!time::reached(
            u64::from_le_bytes(started[..8].try_into().unwrap()),
            deadline
        ));
        let (reply, len) = client.call_retained(&child.raw().to_le_bytes()).unwrap();
        assert_eq!(&reply[..len], &[round as u8]);
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
        let (reply, len) = observer.call_retained(b"finish").unwrap();
        assert_eq!(len, 8);
        assert!(u64::from_le_bytes(reply[..8].try_into().unwrap()) > 1);
        stale = Some(child.raw());
    }
    let (reply, len) = client.call_retained(b"stop").unwrap();
    assert_eq!(&reply[..len], b"stopped");
    client.retire();
    observer.retire();
    await_reclamation();
    write(b"Cathedral display: shared coordination reclaimed\n").unwrap();
    0
}

fn await_reclamation() {
    // A final reply can wake us before the peer drops its reader and signal
    // grants. Retiring our endpoint is not proof that those charges have ended.
    let deadline = time::after(100).unwrap();
    loop {
        let used = cathedral_user_runtime::resources::budget().unwrap().used;
        let notifications = cathedral_user_runtime::notification::budget().unwrap();
        if [used.private_pages, used.reader_pages, used.channels] == [0; 3]
            && [notifications[1], notifications[3]] == [0; 2]
        {
            return;
        }
        assert!(
            !time::reached(time::now().unwrap(), deadline),
            "peer transport leaked"
        );
        cathedral_user_runtime::yield_now();
    }
}
