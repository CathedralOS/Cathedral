#![no_std]
#![no_main]
//! Physical-key provider: exclusive raw bytes in, normalized events over IPC out.
mod controller;
use cathedral_contracts::{input as wire, user as abi};
use cathedral_input_service::Decoder;
use cathedral_user_runtime::{Error, ipc::Handle, keyboard};
cathedral_user_runtime::entry!(main);

fn main(_: u64, _: u64) -> u64 {
    if serve().is_ok() { 0 } else { 1 }
}
fn serve() -> Result<(), Error> {
    controller::initialize()?;
    let request = Handle::bootstrap(0)?;
    let reply = Handle::bootstrap(1)?;
    let mut decoder = Decoder::default();
    reply.send(&wire::Event::reset().encode())?;
    loop {
        let mut bytes = [0; 64];
        let length = match request.receive(&mut bytes) {
            Ok(length) => length,
            Err(Error(code)) if code == abi::PEER_CLOSED as i64 => return Ok(()),
            Err(error) => return Err(error),
        };
        if bytes[..length] != wire::NEXT {
            return Err(Error(abi::INVALID_ARGUMENT as i64));
        }
        let event = loop {
            match keyboard::read(true)? {
                keyboard::LOST => break decoder.reset(),
                byte => {
                    if let Some(event) = decoder.feed(byte as u8) {
                        break event;
                    }
                }
            }
        };
        reply.send(&event.encode())?;
    }
}
