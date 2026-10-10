//! App-owned pixels cross into the display provider as a sealed read-only lease.
use cathedral_user_runtime::{Error, link::Link, memory::Private};
pub(super) fn draw(client: &cathedral_boot_scene::Client) -> Result<(), Error> {
    let mut buffer = Private::allocate(1)?;
    for (index, pixel) in buffer.bytes_mut().chunks_exact_mut(4).enumerate() {
        let color: u32 = if ((index % 64) / 8 + (index / 64) / 8) % 2 == 0 {
            0x101827
        } else {
            0x59d9cc
        };
        pixel.copy_from_slice(&color.to_le_bytes());
    }
    let sealed = buffer.seal(Link::at(cathedral_session_protocol::link::DISPLAY)?)?;
    client.buffer(80, 248, 64, 16, &sealed)?;
    sealed.release()?;
    client.text(80, 224, 1, 0x101827, b"SHARED PIXELS")
}
