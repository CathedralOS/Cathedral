//! Establish the display protocol before lending its only client to the root.
use super::*;
use cathedral_platform_contracts::display::{self, composition as c, pointer};
pub(super) fn request(pair: Pair, words: [u64; 6]) -> Result<[u64; 6], Error> {
    let deadline = time::after(READY_TICKS)?;
    pair.0.send(&display::encode(words))?;
    let (bytes, n) = receive(pair, deadline)?;
    let reply = display::decode(&bytes[..n]).ok_or(BAD_REPLY)?;
    if reply[0] != 0 {
        return Err(Error(reply[0] as i64));
    }
    Ok(reply)
}
pub(super) fn start(attempt: &mut Attempt, argument: u64) -> Result<(Pair, [u64; 2]), Error> {
    assert_eq!(
        Launch::at(0)?.executable().unwrap_err(),
        Error(abi::DENIED as i64)
    );
    let (send, input) = attempt.channel()?;
    let (output, recv) = attempt.channel()?;
    attempt.spawn(Stage::Display, argument, &[input, output])?;
    let pair = (send, recv);
    let info = request(pair, [display::INFO, 0, 0, 0, 0, 0])?;
    if !(1..=wire::MAX_EXTENT).contains(&info[1]) || !(1..=wire::MAX_EXTENT).contains(&info[2]) {
        return Err(BAD_REPLY);
    }
    request(pair, [c::BACKGROUND, BACKGROUND, 0, 0, 0, 0])?;
    request(pair, [pointer::ENABLE, 1, 0, 0, 0, 0])?;
    Ok((pair, [info[1], info[2]]))
}

#[cfg(feature = "shell")]
pub(super) fn failure_screen() {
    // No matrix/input/storage dependency. Use only the original display grant;
    // never draw through an owner-side framebuffer or resurrect a user session.
    let mut attempt = Attempt::new();
    let result = (|| {
        let (pair, [width, height]) = start(&mut attempt, 0)?;
        request(pair, [pointer::ENABLE, 0, 0, 0, 0, 0])?;
        for (text, y) in [
            (
                b"COULD NOT START".as_slice(),
                (height / 2).saturating_sub(24),
            ),
            (b"PLEASE RESTART".as_slice(), height / 2 + 12),
        ] {
            let x = width.saturating_sub(text.len() as u64 * 12) / 2;
            pair.0
                .send(&display::encode_text(x, y, 2, 0xffffff, text).unwrap())?;
            let (bytes, n) = receive(pair, time::after(READY_TICKS)?)?;
            if display::decode(&bytes[..n]) != Some([0; 6]) {
                return Err(BAD_REPLY);
            }
        }
        Ok::<_, Error>(pair)
    })();
    if let Ok(pair) = result {
        cathedral_user_runtime::write(b"Cathedral desktop: restart screen visible\n").unwrap();
        loop {
            match receive(pair, time::after(1000).unwrap()) {
                Err(Error(code)) if code == abi::TIMED_OUT as i64 => (),
                _ => break,
            }
        }
    }
    attempt.retire();
}
