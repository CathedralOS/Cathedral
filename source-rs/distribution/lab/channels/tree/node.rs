//! Identical supervisor at each depth. No PID, global service lookup or depth API.
use super::*;
use cathedral_user_runtime::task::{Child, Port};

struct Running {
    child: Child,
    send: Handle,
    recv: Handle,
    old: Launch,
}
impl Running {
    fn start(count: u64, audit: Handle) -> Self {
        let launch = Launch::at(0).unwrap();
        let loans = [Launch::at(1).ok()];
        let (send, input) = ipc::channel().unwrap();
        let (output, recv) = ipc::channel().unwrap();
        let child = if count == 2 {
            launch.spawn_tree(count - 1, &[input, output, audit], &[loans[0].unwrap()])
        } else {
            launch.spawn_tree(0, &[input, output, audit], &[])
        }
        .unwrap();
        assert_eq!(&receive(recv).0[..5], b"ready");
        Self {
            child,
            send,
            recv,
            old: launch,
        }
    }
    fn stop(self) {
        self.child.cancel().unwrap();
        assert_eq!(self.child.wait().unwrap(), Outcome::Cancelled);
        assert_eq!(self.send.send(b"dead"), Err(Error(abi::PEER_CLOSED as i64)));
        self.send.close().unwrap();
        self.recv.close().unwrap();
        // This launch stayed here; deeper loans return with fresh names.
        assert_eq!(Launch::at(0).unwrap().raw(), self.old.raw());
    }
}
pub fn run(count: u64, environment: Environment) -> u64 {
    assert!(count <= 2);
    assert_eq!(environment.len(), 3);
    assert!(Launch::at(count).is_err());
    assert!(Port::at(count).is_err());
    let input = environment.endpoint(0).unwrap();
    let output = environment.endpoint(1).unwrap();
    let audit = environment.endpoint(2).unwrap();
    let mut running = if count == 0 {
        audit.send(b"leaf").unwrap();
        None
    } else {
        Some(Running::start(count, audit))
    };
    output.send(b"ready").unwrap();
    loop {
        let mut command = [0; 64];
        let (length, attachment) = input.receive_with(&mut command).unwrap();
        match &command[..length] {
            b"return" => return 70, // Kernel must reclaim our live descendants.
            b"stop" => {
                running.take().unwrap().stop();
                output.send(b"stopped").unwrap();
            }
            b"restart" => {
                running.take().unwrap().stop();
                running = Some(Running::start(count, attachment.unwrap()));
                output.send(b"ready").unwrap();
            }
            _ => panic!("unknown tree command"),
        }
    }
}
