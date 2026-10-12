//! Root fixture lends two budgets through three identical recursive processes.
//! node.rs owns the per-level algorithm; the root observes only its direct child.
mod node;
mod probes;
use super::*;
use cathedral_user_runtime::task::{Environment, Launch, Outcome, Port};
pub use node::run as worker;

pub fn run() {
    for round in 0..16 {
        let launch = Launch::at(1).unwrap();
        let loans = [Launch::at(2).unwrap(), Launch::at(3).unwrap()];
        let withheld = Launch::at(4).unwrap();
        let sibling = withheld.spawn(1).unwrap();
        let (_, sibling_events) = Port::at(4).unwrap().connect().unwrap();
        assert_eq!(&receive(sibling_events).0[..6], b"parked");
        let (send, input) = ipc::channel().unwrap();
        let (output, recv) = ipc::channel().unwrap();
        let (audit, witness) = ipc::channel().unwrap();
        probes::check(launch);
        for (grants, error) in [
            ([loans[0], loans[0]], abi::INVALID_ARGUMENT),
            ([loans[0], Launch::from_raw(0)], abi::BAD_HANDLE),
            ([loans[0], launch], abi::DENIED),
            ([loans[0], withheld], abi::BUSY),
        ] {
            assert_eq!(
                launch
                    .spawn_tree(2, &[input, output, audit], &grants)
                    .unwrap_err(),
                Error(error as i64)
            );
            send.send(b"retained").unwrap();
            assert_eq!(input.receive(&mut [0; 64]).unwrap(), 8);
            assert_eq!(Launch::at(2).unwrap().raw(), loans[0].raw());
        }
        let child = launch
            .spawn_tree(2, &[input, output, audit], &loans)
            .unwrap();
        assert_eq!(Launch::at(2).unwrap().raw(), withheld.raw());
        assert!(Launch::at(3).is_err());
        for old in loans {
            assert_eq!(old.spawn(0).unwrap_err(), Error(abi::BAD_HANDLE as i64));
        }
        assert_eq!(&receive(recv).0[..5], b"ready");
        assert_eq!(&receive(witness).0[..4], b"leaf");
        match round % 4 {
            1 => send.send(b"return").unwrap(),
            2 => {
                send.send(b"stop").unwrap();
                assert_eq!(&receive(recv).0[..7], b"stopped");
                child.cancel().unwrap();
            }
            3 => {
                let (audit, next_witness) = ipc::channel().unwrap();
                send.send_with(b"restart", audit).unwrap();
                assert_eq!(&receive(recv).0[..5], b"ready");
                assert_eq!(&receive(next_witness).0[..4], b"leaf");
                child.cancel().unwrap();
                assert_eq!(
                    next_witness.receive(&mut [0; 64]),
                    Err(Error(abi::PEER_CLOSED as i64))
                );
                next_witness.close().unwrap();
            }
            _ => child.cancel().unwrap(),
        }
        assert_eq!(
            child.wait_until(time::after(300).unwrap()).unwrap(),
            if round % 4 == 1 {
                Outcome::Returned(70)
            } else {
                Outcome::Cancelled
            }
        );
        // The witness's only sender was passed all the way to the deepest leaf.
        assert_eq!(
            witness.receive(&mut [0; 64]),
            Err(Error(abi::PEER_CLOSED as i64))
        );
        assert_eq!(send.send(b"dead"), Err(Error(abi::PEER_CLOSED as i64)));
        assert_eq!(
            recv.receive(&mut [0; 64]),
            Err(Error(abi::PEER_CLOSED as i64))
        );
        for (index, old) in loans.into_iter().enumerate() {
            assert_ne!(Launch::at(index as u64 + 2).unwrap().raw(), old.raw());
            assert_eq!(old.spawn(0).unwrap_err(), Error(abi::BAD_HANDLE as i64));
        }
        assert_eq!(Launch::at(4).unwrap().raw(), withheld.raw());
        assert_eq!(
            sibling.wait_until(time::after(0).unwrap()),
            Err(Error(abi::TIMED_OUT as i64))
        );
        sibling.cancel().unwrap();
        assert_eq!(sibling.wait().unwrap(), Outcome::Cancelled);
        for handle in [send, recv, witness] {
            handle.close().unwrap();
        }
    }
    cathedral_user_runtime::write(b"Cathedral IPC: recursive launch loans, subtree cancellation and fresh-budget restart passed\n").unwrap();
}
