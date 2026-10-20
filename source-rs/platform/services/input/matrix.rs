//! Physical source adapter for the same matrix input boundary used recursively.
//! Decoding/controller code is shared with the existing compositor source.
use cathedral_input_service::decoder::{Decoder, mouse::Mouse};
use cathedral_kernel_contracts::user as abi;
use cathedral_platform_contracts::input;
use cathedral_platform_contracts::matrix as wire;
use cathedral_user_runtime::{Error, ipc::Handle, keyboard, task::Environment, time};
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
    super::controller::initialize_mouse()?;
    #[cfg(feature = "absolute-pointer")]
    let mut absolute = super::absolute::Pointer::open();
    let mut mouse = Mouse::default();
    let mut keys = Decoder::source();
    let (mut x, mut y) = (40, 40);
    publish(pair, [wire::POINTER_SOURCE, x, y, 0, 0, 0, 0, 0])?;
    Handle::bootstrap(1)?.send(b"ready")?;
    loop {
        #[cfg(feature = "absolute-pointer")]
        for _ in 0..64 {
            match absolute.next(width, height) {
                Ok(Some((px, py, buttons))) => {
                    x = px;
                    y = py;
                    publish(pair, [wire::POINTER_SOURCE, x, y, buttons, 0, 0, 0, 0])?;
                }
                Ok(None) => break,
                Err(_) => {
                    absolute.disable();
                    mouse.reset();
                    keys.reset();
                    publish(
                        pair,
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
                pair,
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
                        pair,
                        [wire::KEY_SOURCE, input::RESET as u64, 0, 0, 0, 0, 0, 0],
                    )?;
                }
                publish(
                    pair,
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
                pair,
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
fn publish(pair: (Handle, Handle), words: wire::Packet) -> Result<(), Error> {
    if exchange(pair, words)?[0] == 0 {
        Ok(())
    } else {
        Err(Error(abi::IO_ERROR as i64))
    }
}
fn exchange(pair: (Handle, Handle), words: wire::Packet) -> Result<wire::Packet, Error> {
    pair.0.send(&wire::encode(words))?;
    let mut bytes = [0; 64];
    let len = pair.1.receive_until(&mut bytes, time::after(100)?)?;
    wire::decode(&bytes[..len]).ok_or(Error(abi::IO_ERROR as i64))
}
