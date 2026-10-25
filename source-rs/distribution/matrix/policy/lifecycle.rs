//! Pure close/loss policy bridge. Rust retains all endpoints and task identities.
//! Decisions: source/distribution/matrix/lifecycle/main.omg; transport never enters Omega.
use cathedral_platform_contracts::matrix::Packet;
#[derive(Debug, PartialEq, Eq)]
pub enum Close {
    Refuse,
    Stop,
    AlreadyStopped,
}
pub struct Survey {
    state: [u64; 2],
}
impl Survey {
    pub fn begin(busy: [bool; 4]) -> Option<Self> {
        let flags = busy
            .iter()
            .enumerate()
            .fold(0, |mask, (i, b)| mask | (u64::from(*b) << i));
        let (result, state) = decide(3, flags, 0, [0; 2], [0; 8]);
        (result == 1).then_some(Self { state })
    }
    pub fn sent(&mut self, slot: usize, sent: bool) {
        let (result, state) = decide(4, slot as u64, u64::from(sent), self.state, [0; 8]);
        assert_eq!(result, 1, "valid survey slot");
        self.state = state;
    }
    pub fn answer(&mut self, slot: usize, answer: Result<Packet, u64>) {
        let (result, state) = decide(
            5,
            slot as u64,
            u64::from(answer.is_ok()),
            self.state,
            answer.unwrap_or([0; 8]),
        );
        assert_eq!(result, 1, "one answer per pending survey slot");
        self.state = state;
    }
    pub fn finish(&self, recovery: bool) -> Option<bool> {
        match decide(6, u64::from(recovery), 0, self.state, [0; 8]).0 {
            0 => None,
            1 => Some(false),
            2 => Some(true),
            _ => unreachable!("survey policy result"),
        }
    }
}
pub fn close(retired: bool, answer: Result<Packet, u64>) -> Close {
    match decide(
        7,
        u64::from(retired),
        u64::from(answer.is_ok()),
        [0; 2],
        answer.unwrap_or([0; 8]),
    )
    .0
    {
        1 => Close::Refuse,
        2 => Close::Stop,
        3 => Close::AlreadyStopped,
        _ => unreachable!("close policy result"),
    }
}
pub fn wait(is_close: bool, teardown_busy: bool) -> bool {
    decide(
        8,
        u64::from(is_close),
        u64::from(teardown_busy),
        [0; 2],
        [0; 8],
    )
    .0 == 1
}
// Keep the temporary native packet out of the caller's already deep IPC stack.
#[inline(never)]
fn decide(op: u64, first: u64, second: u64, state: [u64; 2], answer: Packet) -> (u64, [u64; 2]) {
    let mut p = [0; 128];
    p[..3].copy_from_slice(&[op, first, second]);
    p[16..18].copy_from_slice(&state);
    p[24..32].copy_from_slice(&answer);
    let result = super::native::decide(&mut p);
    (result, [p[16], p[17]])
}
