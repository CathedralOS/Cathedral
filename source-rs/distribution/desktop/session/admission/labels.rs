//! Static labels used only by the catalogue lab compositions.
use super::*;
use cathedral_platform_contracts::display;
pub(super) fn paint(pair: Pair) -> Result<(), Error> {
    let labels: &[(u64, u64, &[u8])] = if cfg!(feature = "shell") {
        &[] // The shell puts its headings inside the phase-specific chooser.
    } else if cfg!(feature = "notes") {
        &[
            (80, 60, b"PRIVATE NOTES"),
            (134, 140, b"NESTED NOTE"),
            (500, 140, b"SIBLING NOTE"),
            (80, 400, b"F1 SAVE OR CHECK"),
            (80, 432, b"F2 RELOAD"),
            (80, 464, b"F3 COPY F4 NEXT"),
            (500, 400, b"GREEN SAVED"),
            (500, 432, b"AMBER DRAFT"),
            (500, 464, b"RED SAVE ERROR"),
        ]
    } else {
        &[
            (80, 60, b"MATRIX SESSION".as_slice()),
            (134, 140, b"NESTED APP"),
            (500, 140, b"SIBLING APP"),
            (80, 400, b"CLICK THEN ENTER"),
            (80, 440, b"RED: UNCONFIRMED"),
            (80, 472, b"ENTER TO RELOAD"),
        ]
    };
    for &(x, y, text) in labels {
        pair.0
            .send(&display::encode_text(x, y, 2, 0xffffff, text).unwrap())?;
        let (bytes, n) = receive(pair, time::after(READY_TICKS)?)?;
        if display::decode(&bytes[..n]) != Some([0; 6]) {
            return Err(BAD_REPLY);
        }
    }
    Ok(())
}
