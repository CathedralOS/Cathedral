//! Internal names include their supplying boundary. These never go on the wire.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(in crate::world) struct Name {
    pub value: u32,
    pub inherited: bool,
}
impl Name {
    pub const EMPTY: Self = Self::local(0);
    pub const fn local(value: u32) -> Self {
        Self {
            value,
            inherited: false,
        }
    }
    pub const fn upstream(value: u32) -> Self {
        Self {
            value,
            inherited: true,
        }
    }
    pub fn valid(self) -> bool {
        self.value != 0
    }
}
