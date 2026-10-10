//! List the app's private saved records. Values come from verified persisted state.
use super::State;
use cathedral_boot_scene::Client;
use cathedral_user_runtime::Error;
pub(super) fn draw(client: &Client, state: &State) -> Result<(), Error> {
    client.text(720, 232, 2, 0x101827, b"SAVED RECORDS")?;
    if state.records == 1 {
        client.text(
            720,
            272,
            2,
            0x101827,
            if state.saved == 0 {
                b"1 EMPTY"
            } else {
                b"1 SCENE (LEGACY)"
            },
        )?;
    } else {
        let mut selection = *b"1 SELECTION 0";
        selection[12] = b'0' + state.selected;
        client.text(720, 272, 2, 0x101827, &selection)?;
        let mut toggles = *b"2 TOGGLES   0";
        toggles[12] = b'0' + state.active;
        client.text(720, 304, 2, 0x101827, &toggles)?;
    }
    Ok(())
}
