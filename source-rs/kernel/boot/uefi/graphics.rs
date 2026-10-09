//! Snapshot GOP through the UEFI crate. No firmware protocol survives handoff.
use cathedral_contracts::display::{self, Framebuffer};
use uefi::proto::console::gop::{GraphicsOutput, PixelFormat};

pub fn capture() -> Option<Framebuffer> {
    let handle = uefi::boot::get_handle_for_protocol::<GraphicsOutput>().ok()?;
    let mut gop = uefi::boot::open_protocol_exclusive::<GraphicsOutput>(handle).ok()?;
    // A predictable lab mode also makes captured output independently checkable.
    let mode = gop.modes().find(|mode| {
        mode.info().resolution() == (1024, 768)
            && matches!(
                mode.info().pixel_format(),
                PixelFormat::Rgb | PixelFormat::Bgr
            )
    });
    if let Some(mode) = mode {
        gop.set_mode(&mode).ok()?;
    }
    let info = gop.current_mode_info();
    let format = match info.pixel_format() {
        PixelFormat::Rgb => display::RGB,
        PixelFormat::Bgr => display::BGR,
        _ => return None,
    };
    let mut buffer = gop.frame_buffer();
    let framebuffer = Framebuffer {
        physical: buffer.as_mut_ptr() as u64,
        bytes: buffer.size() as u64,
        width: info.resolution().0 as u64,
        height: info.resolution().1 as u64,
        stride: info.stride() as u64,
        format,
    };
    framebuffer.valid().then_some(framebuffer)
}
