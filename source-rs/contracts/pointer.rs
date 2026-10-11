//! Experimental logical pointers; independent of keyboard focus and production seats.
//! All packets use display's six-word encoding. Coordinates on delivery are signed
//! two's-complement app-local values (capture can move outside the app's scope).
pub const SOURCE_MODE: u64 = 0x4d4f5553;
pub const DEVICE: u64 = 32; // source-only: dx,dy,buttons,reset,0
pub const NEXT: u64 = 33; // client poll: all unused fields zero
pub const CAPTURE: u64 = 34; // client: pointer, enable,0,0,0
pub const DRIVE: u64 = 35; // client: lease,x,y,buttons,0; local clamped coordinates
pub const ENABLE: u64 = 36; // control: 1/0,0,0,0,0
pub const LEASE: u64 = 37; // control: client,1/0,0,0,0; reply carries lease epoch
pub const STATE: u64 = 38; // control: pointer,0,0,0,0; reply x,y,buttons,capture+1,lease
pub const MOTION: u64 = 1;
pub const BUTTONS: u64 = 2;
pub const CANCEL: u64 = 3;
pub const NONE: u64 = 0;
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Event {
    pub pointer: u64,
    pub kind: u64,
    pub x: i64,
    pub y: i64,
    pub buttons: u8,
}
impl Event {
    pub fn words(self) -> [u64; 6] {
        [
            0,
            self.pointer,
            self.kind,
            self.x as u64,
            self.y as u64,
            u64::from(self.buttons),
        ]
    }
    pub fn decode(words: [u64; 6]) -> Option<Self> {
        if words[0] != 0 || words[1] > 1 || words[2] > CANCEL || words[5] > 7 {
            return None;
        }
        Some(Self {
            pointer: words[1],
            kind: words[2],
            x: words[3] as i64,
            y: words[4] as i64,
            buttons: words[5] as u8,
        })
    }
}
