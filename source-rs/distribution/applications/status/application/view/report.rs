//! Serial observations emitted only after a complete successful redraw.
use super::{State, digits};
use cathedral_session_protocol as session;
use cathedral_user_runtime::{Error, write};
pub(super) fn completed(state: &State, status: session::Status) -> Result<(), Error> {
    let mut message = *b"Cathedral: selection=0 toggles=0\n";
    message[21] = b'0' + state.selected;
    message[31] = b'0' + state.active;
    write(&message)?;
    let mut health = *b"Cathedral: health=00/00/00 last=0\n";
    digits(&mut health[18..20], status.display);
    digits(&mut health[21..23], status.input);
    digits(&mut health[24..26], status.application);
    health[32] = b'0' + status.last as u8;
    write(&health)?;
    let mut storage = *b"Cathedral: storage=00 saved=0000\n";
    digits(&mut storage[19..21], status.storage);
    digits(&mut storage[28..32], state.saved);
    write(&storage)
}
