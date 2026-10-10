//! Exercise the real display data endpoint with invalid and retired page offers.
use super::*;
use cathedral_contracts::display as wire;
use cathedral_user_runtime::{
    ipc::Handle,
    link::Link,
    memory::{Private, raw},
};
pub(super) fn check(connections: &mut Connections) -> Result<(), Error> {
    let pair = connections.pair(cathedral_session_protocol::link::DISPLAY)?;
    let link = Link::at(cathedral_session_protocol::link::DISPLAY)?;
    let private = raw::allocate(1)?;
    assert_eq!(
        request(pair, [wire::BLIT, 0, 0, 1, 1, private])?,
        abi::DENIED
    );
    // SAFETY: Raw allocation has no Rust wrapper or outstanding references.
    unsafe { raw::release(private) }?;
    for (x, width, height, invalid_color) in [
        (u64::MAX, 1, 1, false),
        (0, 0, 1, false),
        (0, 64, 17, false),
        (0, 64, 16, true),
    ] {
        let mut buffer = Private::allocate(1)?;
        if invalid_color {
            buffer.bytes_mut()[4095] = 255;
        }
        let sealed = buffer.seal(link)?;
        let token = sealed.handle();
        assert_eq!(
            request(pair, [wire::BLIT, x, 0, width, height, token])?,
            abi::INVALID_ARGUMENT
        );
        // The provider must release even an invalid payload before error completion.
        sealed.release()?;
        assert_eq!(
            request(pair, [wire::BLIT, 0, 0, 1, 1, token])?,
            abi::BAD_HANDLE
        );
    }
    write(b"Cathedral: shared pixel bounds permissions and error completion passed\n")
}
fn request(pair: (Handle, Handle), words: [u64; 6]) -> Result<u64, Error> {
    pair.0.send(&wire::encode(words))?;
    let mut bytes = [0; 64];
    let len = pair.1.receive_until(&mut bytes, time::after(100)?)?;
    Ok(wire::decode(&bytes[..len]).unwrap()[0])
}
