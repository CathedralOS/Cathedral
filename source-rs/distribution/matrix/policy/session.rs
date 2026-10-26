//! Private bridge to Omega's stock service orchestration; facts carry no handles.
pub const BUSY: u64 = 1;
pub const READY_LAUNCH: u64 = 2;
pub const REFRESH_READY: u64 = 4;
pub const PUBLISHING: u64 = 8;
pub const DEFERRED_READY: u64 = 16;
pub const DEFERRED_EMPTY: u64 = 32;
pub const STORING: u64 = 64;
pub const CAN_POLL: u64 = 128;
pub const HAS_WAKE: u64 = 256;
pub const EVENT_BLOCKED: u64 = 512;
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u64)]
pub enum Command {
    Interaction = 1,
    Refresh,
    Storage,
    AdmitReady,
    Binding,
    Presentation,
    Selection,
    LaunchScene,
    RefreshRetained,
    FinishChildren,
    Flush,
    RestoreInput,
    Cursors,
    Prune,
    Launch,
    Recover,
    Startup,
    Poll,
    Wait,
    QueryLoss,
    Cancel,
    Defer,
    LegacyRefresh,
    Disconnect,
    ProducerMemory,
    StorageMemory,
    ImageMemory,
    Request,
    Invalid,
}
#[derive(Default)]
pub struct State {
    phase: u64,
    observations: u64,
}
impl State {
    pub fn observations(&self) -> u64 {
        self.observations
    }
    pub fn command(&mut self, flags: u64, event: u64) -> Option<Command> {
        use Command::*;
        Some(match self.apply(0, flags, event) {
            0 => return None,
            1 => Interaction,
            2 => Refresh,
            3 => Storage,
            4 => AdmitReady,
            5 => Binding,
            6 => Presentation,
            7 => Selection,
            8 => LaunchScene,
            9 => RefreshRetained,
            10 => FinishChildren,
            11 => Flush,
            12 => RestoreInput,
            13 => Cursors,
            14 => Prune,
            15 => Launch,
            16 => Recover,
            17 => Startup,
            18 => Poll,
            19 => Wait,
            20 => QueryLoss,
            21 => Cancel,
            22 => Defer,
            23 => LegacyRefresh,
            24 => Disconnect,
            25 => ProducerMemory,
            26 => StorageMemory,
            27 => ImageMemory,
            28 => Request,
            29 => Invalid,
            _ => unreachable!(),
        })
    }
    pub fn complete(&mut self, command: Command, result: bool) -> bool {
        self.apply(1, command as u64, result.into()) == 1
    }
    #[inline(never)]
    fn apply(&mut self, event: u64, a: u64, b: u64) -> u64 {
        let mut p = [0; 128];
        p[..4].copy_from_slice(&[14, event, a, b]);
        p[16] = self.phase;
        p[17] = self.observations;
        let result = super::native::decide(&mut p);
        self.phase = p[16];
        self.observations = p[17];
        result
    }
}
