//! Boot-selected mouse source. Connection identity, not payload, authorizes hardware updates.
use super::controller;
use cathedral_contracts::{display, pointer as wire, user as abi};
use cathedral_input_service::mouse::{Motion, Mouse};
use cathedral_user_runtime::{Error, keyboard, link::Link, time};
pub fn run() -> u64 {
    match serve() {
        Ok(()) => 0,
        Err(error) => panic!("mouse source: {error:?}"),
    }
}
fn serve() -> Result<(), Error> {
    controller::initialize_mouse()?;
    let link = Link::at(0)?;
    let (send, receive) = link.connect()?;
    let mut mouse = Mouse::default();
    send.send(&display::encode([wire::DEVICE, 0, 0, 0, 1, 0]))?;
    receive.receive_until(&mut [0; 64], time::after(200)?)?;
    cathedral_user_runtime::write(b"Cathedral pointer: mouse source ready\n")?;
    cathedral_user_runtime::ipc::Handle::bootstrap(1)?.send(b"ready")?;
    loop {
        let byte = match keyboard::read_until(time::after(25)?) {
            Ok(byte) => byte,
            Err(Error(code)) if code == abi::TIMED_OUT as i64 => continue,
            Err(error) => return Err(error),
        };
        let event = match byte {
            // Lab F5 exits this provider; the distribution replaces its source.
            0x3f => return Ok(()),
            keyboard::LOST => Some(mouse.reset()),
            // Reserved lab Escape cancels physical capture before app routing.
            1 => Some(Motion {
                dx: 0,
                dy: 0,
                buttons: 0,
                reset: true,
            }),
            byte if byte & keyboard::AUX != 0 => mouse.feed(byte as u8),
            _ => None,
        };
        if let Some(m) = event {
            send.send(&display::encode([
                wire::DEVICE,
                m.dx as i64 as u64,
                m.dy as i64 as u64,
                u64::from(m.buttons),
                u64::from(m.reset),
                0,
            ]))?;
            let mut bytes = [0; 64];
            let len = receive.receive_until(&mut bytes, time::after(200)?)?;
            if display::decode(&bytes[..len]).is_none_or(|reply| reply[0] != 0) {
                return Err(Error(abi::IO_ERROR as i64));
            }
        }
    }
}
