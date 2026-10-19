//! Remember the configured launcher's policy across bounded and explicit attempts.
use super::*;
use cathedral_matrix::world::Placement;
use cathedral_user_runtime::memory::Object;
#[derive(Clone, Copy)]
pub struct Fallback {
    pub image: cathedral_user_runtime::task::Executable,
    pub argument: u64,
}
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Phase {
    Running { automatic: bool },
    Reading { started: bool },
    NeedFallback,
    Fallback,
    Requested,
}
pub struct Launcher {
    pub plan: parent::LaunchRequest,
    pub task: u64,
    pub incarnation: u64,
    pub snapshot: Option<(Placement, [u64; 4])>,
    pub phase: Phase,
    pub result: Option<Result<Object, u64>>,
}
impl Launcher {
    pub fn new(plan: parent::LaunchRequest, task: u64, incarnation: u64) -> Self {
        Self {
            plan,
            task,
            incarnation,
            snapshot: None,
            phase: Phase::Running { automatic: true },
            result: None,
        }
    }
    /// Only a genuine launcher failure schedules recovery. A dead fallback,
    /// clean exit or explicit stop never acts as a retry request.
    pub fn retired(&mut self, failed: bool) -> bool {
        match self.phase {
            Phase::Running { automatic } if failed => {
                self.phase = if automatic {
                    Phase::Reading { started: false }
                } else {
                    Phase::NeedFallback
                };
                true
            }
            _ => false,
        }
    }
}
