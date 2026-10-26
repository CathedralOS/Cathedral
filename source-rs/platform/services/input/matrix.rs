//! Physical source adapter for the same matrix input boundary used recursively.
//! Decoding/controller code is shared with the existing compositor source.
use cathedral_input_service::decoder::{Decoder, mouse::Mouse};
use cathedral_kernel_contracts::user as abi;
use cathedral_platform_contracts::input;
use cathedral_platform_contracts::matrix as wire;
use cathedral_user_runtime::{Error, connection::Client, keyboard, task::Environment, time};
pub fn run(extent: u64, environment: Environment) -> u64 {
    serve(extent, environment).unwrap();
    0
}
fn serve(extent: u64, environment: Environment) -> Result<(), Error> {
    assert_eq!(environment.len(), 2);
    let (width, height) = (extent & 0xffff_ffff, extent >> 32);
    assert!((1..=wire::MAX_EXTENT).contains(&width) && (1..=wire::MAX_EXTENT).contains(&height));
    let pair = (
        environment.endpoint(0).unwrap(),
        environment.endpoint(1).unwrap(),
    );
    let mut connection = Client::connect(pair, time::after(300)?)?;
    // Validate attenuation on the connection the driver actually owns. The boot
    // owner passes untouched endpoints, never an already-upgraded raw alias.
    for op in [wire::SPAWN, wire::READ_PIXEL, wire::BIND] {
        let (bytes, n) =
            connection.call(&wire::encode([op, 0, 0, 0, 0, 0, 0, 0]), time::after(300)?)?;
        if wire::decode(&bytes[..n]) != Some(wire::error(abi::DENIED)) {
            return Err(Error(abi::IO_ERROR as i64));
        }
    }
    cathedral_user_runtime::write(b"Cathedral input: shared source authority checked\n")?;
    super::controller::initialize_mouse()?;
    #[cfg(feature = "absolute-pointer")]
    let mut absolute = super::absolute::Pointer::open();
    let mut mouse = Mouse::default();
    let mut keys = Decoder::source();
    let (mut x, mut y) = (40, 40);
    publish(&mut connection, [wire::POINTER_SOURCE, x, y, 0, 0, 0, 0, 0])?;
    cathedral_user_runtime::write(b"Cathedral input: shared source connection ready\n")?;
    super::readiness::serve()?;
    loop {
        #[cfg(feature = "absolute-pointer")]
        for _ in 0..64 {
            match absolute.next(width, height) {
                Ok(Some((px, py, buttons))) => {
                    x = px;
                    y = py;
                    publish(
                        &mut connection,
                        [wire::POINTER_SOURCE, x, y, buttons, 0, 0, 0, 0],
                    )?;
                }
                Ok(None) => break,
                Err(_) => {
                    absolute.disable();
                    mouse.reset();
                    keys.reset();
                    publish(
                        &mut connection,
                        [wire::KEY_SOURCE, input::RESET as u64, 0, 0, 0, 0, 0, 0],
                    )?;
                    break;
                }
            }
        }
        let byte = match keyboard::read_until(time::after(10)?) {
            Ok(byte) => byte,
            Err(Error(code)) if code == abi::TIMED_OUT as i64 => continue,
            Err(error) => return Err(error),
        };
        if byte == keyboard::LOST {
            mouse.reset();
            keys.reset();
            publish(
                &mut connection,
                [wire::KEY_SOURCE, input::RESET as u64, 0, 0, 0, 0, 0, 0],
            )?;
        } else if byte & keyboard::AUX != 0 {
            #[cfg(feature = "absolute-pointer")]
            if absolute.enabled() {
                continue;
            }
            if let Some(motion) = mouse.feed(byte as u8) {
                x = (x as i64 + motion.dx as i64).clamp(0, width as i64 - 1) as u64;
                y = (y as i64 + motion.dy as i64).clamp(0, height as i64 - 1) as u64;
                if motion.reset {
                    publish(
                        &mut connection,
                        [wire::KEY_SOURCE, input::RESET as u64, 0, 0, 0, 0, 0, 0],
                    )?;
                }
                publish(
                    &mut connection,
                    [
                        wire::POINTER_SOURCE,
                        x,
                        y,
                        motion.buttons as u64,
                        0,
                        0,
                        0,
                        0,
                    ],
                )?;
            }
        } else if let Some(event) = keys.feed(byte as u8) {
            #[cfg(feature = "trace-input")]
            time::trace(if event.state == input::RELEASE {
                "source-release"
            } else {
                "source-key"
            });
            publish(
                &mut connection,
                [
                    wire::KEY_SOURCE,
                    event.key as u64,
                    event.state as u64,
                    0,
                    0,
                    0,
                    0,
                    0,
                ],
            )?;
        }
    }
}
fn publish(connection: &mut Client, words: wire::Packet) -> Result<(), Error> {
    #[cfg(feature = "trace-input")]
    if words[0] == wire::KEY_SOURCE {
        time::input_sample("source", &[words[1], words[2], time::now()?]);
    }
    let (bytes, n) = connection.call_retained(&wire::encode(words))?;
    if wire::decode(&bytes[..n]).is_some_and(|reply| reply[0] == 0) {
        Ok(())
    } else {
        Err(Error(abi::IO_ERROR as i64))
    }
}
