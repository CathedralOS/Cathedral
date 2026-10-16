//! Bounded session experiment: focus authority and independently owned child scenes.
//! Six-word packets use display::encode. Tokens are checked against the caller's
//! connection, ancestor grants and incarnations; numeric IDs alone grant nothing.
pub const FOCUS_GRANT: u64 = 48; // control: client,key_mask,may_delegate,0,0 -> token
pub const FOCUS_DELEGATE: u64 = 49; // host: token,child,key_mask,may_delegate,0 -> child token
pub const FOCUS_SELECT: u64 = 50; // app/host: token,target,0,0,0 (within active subtree)
pub const FOCUS_REVOKE: u64 = 51; // app/host: token,0,0,0,0
pub const KEY_SOURCE: u64 = 52; // source only: key,state,0,0,0; key0 resets source
pub const FOCUS_STATE: u64 = 53; // control: all zero -> active client+1, token
pub const HOST_GRANT: u64 = 54; // control: host,child,0,0,0 -> placement token
pub const HOST_PLACE: u64 = 55; // host: token,x,y,width,height
pub const HOST_REVOKE: u64 = 56; // host: token,0,0,0,0
pub const SESSION_CONTROL: u64 = 57; // source: F4/F6/F7 lab request -> control polls action
pub const KEY_MASK: u64 = 0x1f; // Up, Down, Left, Right, Enter. No global shortcuts.
pub const KEY_EVENT: u64 = 4;
pub const FOCUS_LOST: u64 = 5;
pub const KEY_STREAM: u64 = 2;
pub fn key_event(key: u8, state: u8, token: u64) -> [u64; 6] {
    [0, KEY_STREAM, KEY_EVENT, key as u64, state as u64, token]
}
pub fn focus_lost(token: u64) -> [u64; 6] {
    [0, KEY_STREAM, FOCUS_LOST, 0, 0, token]
}
