//! Configure through a short-lived boot stream; the root receives a separate
//! presentation pair. Copied positional labs retain the old staged exchange.
use super::*;
use cathedral_platform_contracts::display::{self, composition as c, pointer};
#[cfg(feature = "shell")]
pub(super) type Control = cathedral_user_runtime::connection::Client;
#[cfg(not(feature = "shell"))]
pub(super) type Control = Pair;

pub(super) fn exchange(control: &mut Control, bytes: &[u8]) -> Result<[u64; 6], Error> {
    exchange_until(control, bytes, time::after(READY_TICKS)?)
}
fn exchange_until(control: &mut Control, bytes: &[u8], deadline: u64) -> Result<[u64; 6], Error> {
    #[cfg(feature = "shell")]
    let (bytes, n) = control.call(bytes, deadline)?;
    #[cfg(not(feature = "shell"))]
    let (bytes, n) = {
        control.0.send(bytes)?;
        receive(*control, deadline)?
    };
    display::decode(&bytes[..n]).ok_or(BAD_REPLY)
}
fn request(control: &mut Control, words: [u64; 6]) -> Result<[u64; 6], Error> {
    let reply = exchange(control, &display::encode(words))?;
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
    let deadline = time::after(READY_TICKS)?;
    #[cfg(feature = "shell")]
    let mut control = Control::connect(Port::at(0)?.connect()?, deadline)?;
    #[cfg(not(feature = "shell"))]
    let mut control = pair;
    let info = exchange_until(
        &mut control,
        &display::encode([display::INFO, 0, 0, 0, 0, 0]),
        deadline,
    )?;
    if info[0] != 0 {
        return Err(Error(info[0] as i64));
    }
    if !(1..=wire::MAX_EXTENT).contains(&info[1]) || !(1..=wire::MAX_EXTENT).contains(&info[2]) {
        return Err(BAD_REPLY);
    }
    request(&mut control, [c::BACKGROUND, BACKGROUND, 0, 0, 0, 0])?;
    request(&mut control, [pointer::ENABLE, 1, 0, 0, 0, 0])?;
    #[cfg(feature = "bootstrap-probes")]
    for denied in [
        [c::MATRIX_BEGIN, 1, 0, 0, 0, 0],
        [pointer::DEVICE, 1, 0, 0, 0, 0],
    ] {
        if exchange(&mut control, &display::encode(denied))?[0] != abi::DENIED {
            return Err(BAD_REPLY);
        }
    }
    #[cfg(feature = "shell")]
    cathedral_user_runtime::write(b"Cathedral desktop: shared bootstrap display ready\n")?;
    attempt.display_control = Some(control);
    Ok((pair, [info[1], info[2]]))
}

#[cfg(feature = "shell")]
pub(super) fn failure_screen() {
    // No matrix/input/storage dependency. Use only the original display grant;
    // never draw through an owner-side framebuffer or resurrect a user session.
    let mut attempt = Attempt::new();
    let result = (|| {
        let (_, [width, height]) = start(&mut attempt, 0)?;
        let control = attempt.display_control.as_mut().unwrap();
        request(control, [pointer::ENABLE, 0, 0, 0, 0, 0])?;
        for (text, y) in [
            (
                b"COULD NOT START".as_slice(),
                (height / 2).saturating_sub(24),
            ),
            (b"PLEASE RESTART".as_slice(), height / 2 + 12),
        ] {
            let x = width.saturating_sub(text.len() as u64 * 12) / 2;
            if exchange(
                control,
                &display::encode_text(x, y, 2, 0xffffff, text).unwrap(),
            )? != [0; 6]
            {
                return Err(BAD_REPLY);
            }
        }
        Ok::<_, Error>(())
    })();
    if result.is_ok() {
        cathedral_user_runtime::write(b"Cathedral desktop: restart screen visible\n").unwrap();
        loop {
            let control = attempt.display_control.as_ref().unwrap();
            match cathedral_user_runtime::event::wait(
                &control.interests().unwrap()[..1],
                time::after(1000).unwrap(),
            ) {
                Err(Error(code)) if code == abi::TIMED_OUT as i64 => (),
                _ => break,
            }
        }
    }
    attempt.retire();
}
