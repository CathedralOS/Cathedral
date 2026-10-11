//! Reap and replace an optional hostile peer while ordinary desktop work proceeds.
use super::*;
use crate::pressure::{FAULT, SPIN, STATUS};
pub(super) struct Pressure {
    child: Child,
    pair: Pair,
    polls: u64,
    rounds: u64,
    spinning: bool,
}
impl Pressure {
    pub(super) fn start() -> Option<Self> {
        let launch = match Launch::at(5) {
            Ok(launch) => launch,
            Err(Error(code)) if code == abi::BAD_HANDLE as i64 || code == abi::DENIED as i64 => {
                return None;
            }
            Err(error) => panic!("pressure launch: {error:?}"),
        };
        let child = launch.spawn(0).unwrap();
        let pair = connect(5);
        marker(b"Cathedral pressure: active\n");
        Some(Self {
            child,
            pair,
            polls: 0,
            rounds: 0,
            spinning: false,
        })
    }
    pub(super) fn tick(&mut self) {
        if self.spinning {
            assert_eq!(
                self.child.wait_until(time::now().unwrap()),
                Err(Error(abi::TIMED_OUT as i64))
            );
            self.child.cancel().unwrap();
            assert_eq!(self.child.wait().unwrap(), Outcome::Cancelled);
            marker(b"Cathedral pressure: spinning peer cancelled and reaped\n");
            self.restart();
            return;
        }
        self.polls += 1;
        let command = if self.polls < 4 {
            STATUS
        } else if self.rounds.is_multiple_of(2) {
            SPIN
        } else {
            FAULT
        };
        let stats = call(self.pair, command, 0);
        assert_eq!(stats[0], 0);
        assert!(stats[1] > 0 && stats[2] == stats[1] && stats[3] == 1);
        if command == STATUS {
            return;
        }
        assert!(stats[4] > 3 && stats[5] > 0); // Requests consumed despite unread replies; sender also hit backpressure.
        marker(b"Cathedral pressure: page/object limits and IPC backpressure verified\n");
        if command == SPIN {
            self.spinning = true; // Keep it alive through another full input/health cycle.
        } else {
            assert!(matches!(
                self.child.wait_until(time::after(100).unwrap()).unwrap(),
                Outcome::Fault { vector: 6, .. }
            ));
            marker(b"Cathedral pressure: faulted peer reaped\n");
            self.restart();
        }
    }
    fn restart(&mut self) {
        self.child = Launch::at(5).unwrap().spawn(0).unwrap();
        self.pair = connect(5);
        self.polls = 0;
        self.rounds += 1;
        self.spinning = false;
        marker(b"Cathedral pressure: replacement admitted\n");
    }
}
