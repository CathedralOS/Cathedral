//! Render the status view through the display data link, then report the completed frame.
//! Client: distribution/libraries/boot-scene; protocol: contracts/display.rs;
//! peer: platform/services/display/main.rs -> surface.rs.
mod report;
use super::{connection::Connections, state::State};
use cathedral_contracts::user as abi;
use cathedral_user_runtime::{Error, time};
pub(super) fn render(state: &State, connections: &mut Connections) -> Result<(), Error> {
    for _ in 0..3 {
        let (send, receive) = connections.pair(cathedral_session_protocol::link::DISPLAY)?;
        let client = cathedral_boot_scene::Client::until(send, receive, time::after(100)?);
        let result = client.dimensions().and_then(|(w, h)| {
            client.draw_interactive(w, h, state.selected, state.active)?;
            client.text(64, 96, 2, 0xe8edf4, b"CATHEDRAL / STATUS")?;
            for (x, text) in [
                (80, &b"DISPLAY"[..]),
                (400, &b"INPUT"[..]),
                (720, &b"APPLICATION"[..]),
            ] {
                client.text(x, 184, 2, 0x101827, text)?;
            }
            client.text(64, 592, 2, 0xe8edf4, b"ARROWS SELECT / ENTER TOGGLE")?;
            let status = connections.status;
            let mut counters = *b"DISPLAY READY 00  INPUT READY 00  APP 00";
            digits(&mut counters[14..16], status.display);
            digits(&mut counters[30..32], status.input);
            digits(&mut counters[38..40], status.application);
            client.text(64, 632, 2, 0x59d9cc, &counters)?;
            client.text(
                64,
                664,
                2,
                0xe8edf4,
                match status.last {
                    1 => b"LAST RECOVERY: DISPLAY",
                    2 => b"LAST RECOVERY: INPUT",
                    3 => b"LAST RECOVERY: APPLICATION",
                    4 => b"LAST RECOVERY: STORAGE",
                    _ => b"LAST RECOVERY: NONE",
                },
            )?;
            let mut storage = *b"STORAGE READY 00  SAVED 0000";
            digits(&mut storage[14..16], status.storage);
            digits(&mut storage[24..28], state.saved);
            client.text(64, 696, 2, 0x59d9cc, &storage)
        });
        if result.is_ok() {
            return report::completed(state, connections.status);
        }
        connections.display = None;
        connections.refresh()?;
    }
    Err(Error(abi::IO_ERROR as i64))
}
fn digits(bytes: &mut [u8], value: u64) {
    let mut value = value.min(10u64.pow(bytes.len() as u32) - 1);
    for byte in bytes.iter_mut().rev() {
        *byte = b'0' + (value % 10) as u8;
        value /= 10;
    }
}
