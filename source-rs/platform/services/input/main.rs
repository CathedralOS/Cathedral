#![no_std]
#![no_main]
//! Physical-key provider: exclusive raw bytes in, normalized events over IPC out.
mod controller;
#[cfg(feature = "recovery-lab")]
mod recovery;
use cathedral_contracts::{input as wire, user as abi};
use cathedral_input_service::Decoder;
use cathedral_user_runtime::{Error, ipc::Handle, keyboard, server::Server, time};
cathedral_user_runtime::entry!(main);

fn main(_mode: u64, _generation: u64) -> u64 {
    #[cfg(feature = "recovery-lab")]
    recovery::startup(_mode, _generation);
    if serve().is_ok() { 0 } else { 1 }
}
fn serve() -> Result<(), Error> {
    controller::initialize()?;
    let mut server = Server::open()?;
    let mut decoder = Decoder::default();
    Handle::bootstrap(1)?.send(&wire::Event::reset().encode())?;
    loop {
        let request = match server.next_request() {
            Ok(request) => request,
            Err(Error(code)) if code == abi::PEER_CLOSED as i64 => return Ok(()),
            Err(error) => return Err(error),
        };
        let bytes = &request.bytes[..request.len];
        #[cfg(feature = "recovery-lab")]
        if request.control {
            recovery::request(bytes, Handle::bootstrap(0)?, Handle::bootstrap(1)?);
        }
        if bytes == wire::HEALTH && request.control {
            server.reply(&request, &wire::Event::idle().encode())?;
            continue;
        }
        if bytes != wire::NEXT {
            server.reply(&request, &[0xff, 0])?;
            continue;
        }
        let deadline = time::after(25)?;
        let event = loop {
            if time::reached(time::now()?, deadline) {
                break wire::Event::idle();
            }
            let byte = match keyboard::read_until(deadline) {
                Ok(byte) => byte,
                Err(Error(code)) if code == abi::TIMED_OUT as i64 => break wire::Event::idle(),
                Err(error) => return Err(error),
            };
            match byte {
                keyboard::LOST => break decoder.reset(),
                byte => {
                    if let Some(event) = decoder.feed(byte as u8) {
                        break event;
                    }
                }
            }
        };
        server.reply(&request, &event.encode())?;
    }
}
