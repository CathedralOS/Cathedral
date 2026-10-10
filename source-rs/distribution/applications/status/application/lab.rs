//! Test-only checks run from the app's actual unprivileged principal.
use super::connection::Connections;
use cathedral_contracts::user as abi;
use cathedral_user_runtime::{
    Error, disk, display, keyboard,
    task::{Child, Launch, Port},
    time, write,
};
pub fn authority(connections: &mut Connections) -> Result<(), Error> {
    assert_eq!(Launch::at(0).unwrap_err(), Error(abi::DENIED as i64));
    assert_eq!(Port::at(0).unwrap_err(), Error(abi::DENIED as i64));
    assert_eq!(Child::from_raw(0).cancel(), Err(Error(abi::DENIED as i64)));
    assert_eq!(keyboard::read(false), Err(Error(abi::DENIED as i64)));
    assert_eq!(keyboard::write(false, 0xf4), Err(Error(abi::DENIED as i64)));
    assert_eq!(display::mapping(), Err(Error(abi::DENIED as i64)));
    assert_eq!(disk::read(8), Err(Error(abi::DENIED as i64)));
    assert_eq!(disk::write(8, 6), Err(Error(abi::DENIED as i64)));
    assert_eq!(
        disk::transfer(&mut [0; 512], false),
        Err(Error(abi::DENIED as i64))
    );
    storage(connections)?;
    for index in 0..2 {
        let (send, receive) = connections.pair(index)?;
        send.send(&[0xf0, 1])?; // Provider fault handler must not accept data-channel requests.
        let mut bytes = [0; 64];
        let length = receive.receive_until(&mut bytes, time::after(100)?)?;
        if index == 0 {
            assert_eq!(
                cathedral_contracts::display::decode(&bytes[..length]).unwrap()[0],
                abi::INVALID_ARGUMENT
            );
        } else {
            assert_eq!(&bytes[..length], &[0xff, 0]);
        }
    }
    write(b"Cathedral: application authority isolated\n")
}
fn storage(connections: &mut Connections) -> Result<(), Error> {
    use cathedral_contracts::storage as wire;
    let before = super::storage::load(connections)?;
    let (send, receive) = connections.pair(2)?;
    for (request, error) in [
        (wire::request(wire::READ, 2, 0, &[]).unwrap(), abi::DENIED),
        (
            wire::request(wire::REPLACE, 2, before.generation, b"intruder").unwrap(),
            abi::DENIED,
        ),
        (
            wire::request(
                wire::REPLACE,
                wire::OBJECT,
                before.generation.wrapping_add(1),
                b"stale",
            )
            .unwrap(),
            abi::WOULD_BLOCK,
        ),
        (
            wire::request(99, wire::OBJECT, 0, &[]).unwrap(),
            abi::INVALID_ARGUMENT,
        ),
    ] {
        send.send(&request)?;
        let mut reply = [0; 64];
        let len = receive.receive_until(&mut reply, time::after(100)?)?;
        assert_eq!(wire::Record::decode(&reply[..len]), Err(error));
    }
    for request in [&[0xf0, 1][..], &[0xf1, 4][..]] {
        send.send(request)?;
        let mut reply = [0; 64];
        let len = receive.receive_until(&mut reply, time::after(100)?)?;
        assert_eq!(
            wire::Record::decode(&reply[..len]),
            Err(abi::INVALID_ARGUMENT)
        );
    }
    assert_eq!(super::storage::load(connections)?, before);
    Ok(())
}
pub fn fail(generation: u64) -> ! {
    write(b"Cathedral: application fault armed\n").unwrap();
    match generation % 3 {
        0 => {
            // SAFETY: Deliberate fault in the test app, contained by its task root.
            unsafe {
                core::arch::asm!("ud2", options(noreturn));
            }
        }
        1 => loop {
            core::hint::spin_loop();
        },
        _ => {
            let output = cathedral_user_runtime::ipc::Handle::bootstrap(1).unwrap();
            loop {
                // Never drain responses: backpressure must replace only this app.
                let _ = output.send(cathedral_session_protocol::STATUS);
                cathedral_user_runtime::yield_now();
            }
        }
    }
}

/// Test keys are kept out of the ordinary event/state update path.
pub(super) fn handle(
    event: cathedral_contracts::input::Event,
    generation: u64,
    connections: &mut Connections,
) -> Result<bool, Error> {
    use cathedral_contracts::input;
    if event.state != input::PRESS {
        return Ok(false);
    }
    match event.key {
        input::F1 | input::F2 | input::F4 => {
            connections.request(&[
                0xf0,
                match event.key {
                    input::F1 => 1,
                    input::F4 => 2,
                    _ => 0,
                },
            ])?;
            connections.refresh()?;
            Ok(true)
        }
        input::F3 => fail(generation),
        input::F5..=input::F8 => {
            let phase = event.key - input::F5 + 1;
            let (reply, len) = connections.request(&[0xf1, phase])?;
            assert_eq!(&reply[..len], b"armed");
            let mut message = *b"Cathedral: storage armed=0\n";
            message[25] = b'0' + phase;
            write(&message)?;
            Ok(false)
        }
        _ => Ok(false),
    }
}
