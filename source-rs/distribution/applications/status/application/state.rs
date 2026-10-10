//! App-owned schema and navigation. No IPC, device access or drawing.
use cathedral_contracts::{input, storage::Record, user as abi};
use cathedral_user_runtime::Error;
#[derive(Default)]
pub(super) struct State {
    pub selected: u8,
    pub active: u8,
    pub saved: u64,
    pub records: u8,
}
impl State {
    pub fn restore(record: Record) -> Result<Self, Error> {
        match record.payload() {
            [] if record.generation == 0 => Ok(Self::default()),
            [1, selected @ 0..=2, active @ 0..=7] => Ok(Self {
                selected: *selected,
                active: *active,
                saved: record.generation,
                records: 0,
            }),
            _ => Err(Error(abi::IO_ERROR as i64)),
        }
    }
    pub fn update(&mut self, event: input::Event) -> bool {
        if event.state == input::RELEASE {
            return false;
        }
        match event.key {
            input::LEFT | input::UP => self.selected = (self.selected + 2) % 3,
            input::RIGHT | input::DOWN => self.selected = (self.selected + 1) % 3,
            input::ENTER if event.state == input::PRESS => self.active ^= 1 << self.selected,
            _ => return false,
        }
        true
    }
}
