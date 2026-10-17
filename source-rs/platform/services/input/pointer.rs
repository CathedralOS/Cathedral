//! Physical source coordinator: control/reconnect, device decode, authenticated delivery.
use super::controller;
use cathedral_input_service::decoder::{
    Decoder,
    mouse::{Motion, Mouse},
};
use cathedral_kernel_contracts::user as abi;
use cathedral_platform_contracts::display;
use cathedral_platform_contracts::display::interaction as session;
use cathedral_platform_contracts::display::pointer as wire;
use cathedral_platform_contracts::input;
use cathedral_user_runtime::{Error, ipc::Handle, keyboard, link::Link, time};
type Pair = (Handle, Handle);
pub fn run() -> u64 {
    match serve() {
        Ok(()) => 0,
        Err(error) => panic!("mouse source: {error:?}"),
    }
}
fn serve() -> Result<(), Error> {
    controller::initialize_mouse()?;
    let control = (Handle::bootstrap(0)?, Handle::bootstrap(1)?);
    let mut mouse = Mouse::default();
    let mut keys = Decoder::source();
    let mut connection = Some(connect(keys.held_keys())?);
    control.1.send(b"ready")?;
    loop {
        let mut packet = [0; 64];
        match control.0.receive_until(&mut packet, time::now()?) {
            Ok(len) => {
                if packet[..len] == [2] {
                    connection = Some(connect(keys.held_keys())?);
                    mouse.reset();
                    keys.reset();
                }
                control.1.send(if connection.is_some() {
                    b"ready"
                } else {
                    b"offline"
                })?;
            }
            Err(Error(code)) if code == abi::TIMED_OUT as i64 => (),
            Err(error) => return Err(error),
        }
        let byte = match keyboard::read_until(time::after(10)?) {
            Ok(byte) => byte,
            Err(Error(code)) if code == abi::TIMED_OUT as i64 => continue,
            Err(error) => return Err(error),
        };
        let words = match byte {
            0x3f => return Ok(()), // F5: lab source replacement.
            0x3e => Some([session::SESSION_CONTROL, 1, 0, 0, 0, 0]), // F4: compositor replacement.
            0x41 => Some([session::SESSION_CONTROL, 3, 0, 0, 0, 0]), // F7: app-owned synthetic click.
            0x40 => Some([session::SESSION_CONTROL, 2, 0, 0, 0, 0]), // F6: host replacement.
            keyboard::LOST => {
                cathedral_user_runtime::write(b"Cathedral pointer: raw input lost; cancelling\n")?;
                keys.reset();
                Some(motion(mouse.reset()))
            }
            1 => Some(motion(Motion {
                dx: 0,
                dy: 0,
                buttons: 0,
                reset: true,
            })),
            byte if byte & keyboard::AUX != 0 => mouse.feed(byte as u8).map(motion),
            byte if byte < 256 => keys.feed(byte as u8).map(|event| {
                [
                    session::KEY_SOURCE,
                    u64::from(event.key),
                    u64::from(event.state),
                    0,
                    0,
                    0,
                ]
            }),
            _ => None,
        };
        if let (Some(pair), Some(words)) = (connection, words)
            && publish(pair, words).is_err()
        {
            cathedral_user_runtime::write(b"Cathedral pointer: delivery failed; source offline\n")?;
            connection = None; // Keep serving control through compositor replacement.
        }
    }
}
fn connect(held: u64) -> Result<Pair, Error> {
    let pair = Link::at(0)?.connect()?;
    publish(pair, [wire::DEVICE, 0, 0, 0, 1, 0])?;
    // The physical reset above clears focus. Seed the replacement router's held
    // state before resetting keys, so typematic input cannot become a fresh press
    // if the user clicks a new target while still holding an old key.
    for key in input::UP..=input::ENTER {
        if held & (1 << (key - 1)) != 0 {
            publish(
                pair,
                [
                    session::KEY_SOURCE,
                    u64::from(key),
                    u64::from(input::PRESS),
                    0,
                    0,
                    0,
                ],
            )?;
        }
    }
    publish(
        pair,
        [session::KEY_SOURCE, u64::from(input::RESET), 0, 0, 0, 0],
    )?;
    cathedral_user_runtime::write(b"Cathedral pointer: mouse source ready\n")?;
    Ok(pair)
}
fn publish(pair: Pair, words: [u64; 6]) -> Result<(), Error> {
    pair.0.send(&display::encode(words))?;
    let mut bytes = [0; 64];
    let len = pair.1.receive_until(&mut bytes, time::after(200)?)?;
    if display::decode(&bytes[..len]).is_none_or(|reply| reply[0] != 0) {
        return Err(Error(abi::IO_ERROR as i64));
    }
    Ok(())
}
fn motion(m: Motion) -> [u64; 6] {
    [
        wire::DEVICE,
        m.dx as i64 as u64,
        m.dy as i64 as u64,
        u64::from(m.buttons),
        u64::from(m.reset),
        0,
    ]
}
