//! Adapt authenticated facts to Omega's router; no request identity or handles cross.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u64)]
pub enum Caller {
    Unknown = 0,
    Owner,
    Source,
    Child,
    Storage,
    Producer,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u64)]
pub enum Operation {
    Deny = 0,
    Producer,
    Acquire,
    Resolve,
    Pixel,
    Owner,
    RecoveryOwner,
    StorageOwner,
    LayoutOwner,
    Source,
    Storage,
    DenyNow,
    Launch,
    Close,
    Fund,
    AppEntry,
    Select,
    Layout,
    Status,
    SaveAvailability,
    RetrySaves,
    ManagedTask,
    RetryLauncher,
    Terminate,
    Present,
    Surface,
    World,
}
pub const CHANGED: u64 = 1;
pub const CLEAR_SLOT: u64 = 2;
pub const CLEAR_PAIR: u64 = 4;
pub const REPLAY_SLOT: u64 = 8;
pub const REPLAY_ALL: u64 = 16;
pub const REMEMBER_STORAGE: u64 = 32;
pub struct Request {
    pub caller: Caller,
    pub words: Option<[u64; 8]>,
    pub separate_producer: bool,
    pub storage_stream: bool,
}
impl Request {
    pub fn route(&self) -> Operation {
        operation(self.decide(0, 0, false))
    }
    pub fn owner_fallback(&self) -> Operation {
        operation(self.decide(1, 0, false))
    }
    pub fn owner_effects(&self, children: usize, success: bool) -> u64 {
        self.decide(2, children as u64, success)
    }
    pub fn attachment(&self) -> Operation {
        operation(self.decide(3, 0, false))
    }
    #[inline(never)]
    fn decide(&self, event: u64, children: u64, success: bool) -> u64 {
        let mut p = [0; 128];
        p[..8].copy_from_slice(&[
            15,
            event,
            self.caller as u64,
            self.words.is_some().into(),
            self.separate_producer.into(),
            self.storage_stream.into(),
            children,
            success.into(),
        ]);
        p[16..24].copy_from_slice(&self.words.unwrap_or([0; 8]));
        super::native::decide(&mut p)
    }
}
fn operation(value: u64) -> Operation {
    use Operation::*;
    match value {
        0 => Deny,
        1 => Producer,
        2 => Acquire,
        3 => Resolve,
        4 => Pixel,
        5 => Owner,
        6 => RecoveryOwner,
        7 => StorageOwner,
        8 => LayoutOwner,
        9 => Source,
        10 => Storage,
        11 => DenyNow,
        12 => Launch,
        13 => Close,
        14 => Fund,
        15 => AppEntry,
        16 => Select,
        17 => Layout,
        18 => Status,
        19 => SaveAvailability,
        20 => RetrySaves,
        21 => ManagedTask,
        22 => RetryLauncher,
        23 => Terminate,
        24 => Present,
        25 => Surface,
        26 => World,
        _ => unreachable!("Omega request operation"),
    }
}
