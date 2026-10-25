//! Acquire a bounded masked image, then publish it in our own granted rectangle.
use cathedral_kernel_contracts::user as abi;
use cathedral_platform_contracts::{matrix as wire, services as service};
use cathedral_user_runtime::{
    Error,
    capture::Capture,
    server::{Event, Server},
    services::Services,
    time,
};
pub fn run(mut services: Services) -> Result<(), Error> {
    services.enable_shared()?;
    services.interaction(1, 1)?;
    let mut capture = Capture::connect(services.capture(1, 1)?)?;
    cathedral_user_runtime::write(b"Cathedral capture: shared broker and capture ready\n")?;
    cathedral_user_runtime::write(b"Cathedral capture: recipient services acquired\n")?;
    assert_eq!(
        services.storage(1, 1).err(),
        Some(Error(service::UNAVAILABLE as i64))
    );
    // The capture endpoint cannot acquire services, and interaction cannot capture.
    assert_eq!(
        services.request_interaction(
            [
                cathedral_platform_contracts::capture::BEGIN,
                0,
                0,
                0,
                0,
                0,
                0,
                0
            ],
            None
        )?,
        wire::error(abi::DENIED)
    );
    #[cfg(feature = "capture-probe")]
    crate::probe::refusals(&mut capture)?;
    let ticket = capture.begin()?;
    cathedral_user_runtime::write(b"Cathedral capture: recipient frame requested\n")?;
    if cfg!(feature = "capture-recipient-exit") {
        cathedral_user_runtime::write(
            b"Cathedral capture: recipient exits with pending request\n",
        )?;
        return Ok(());
    }
    let mut server = Server::open()?;
    let mut image = None;
    let mut painted = false;
    loop {
        if image.is_none() {
            image = match capture.take(ticket) {
                Err(Error(code))
                    if cfg!(feature = "capture-timeout")
                        && code == cathedral_platform_contracts::capture::EXPIRED as i64 =>
                {
                    cathedral_user_runtime::write(
                        b"Cathedral capture: recipient observed expiry\n",
                    )?;
                    return Ok(());
                }
                result => result?,
            };
        }
        if let Some(image) = &mut image
            && !painted
        {
            #[cfg(feature = "capture-probe")]
            crate::probe::image(image, &mut capture, ticket)?;
            let view = services.request_interaction([wire::POLL, 0, 0, 0, 0, 0, 0, 0], None)?;
            if view[0] != 0 {
                return Err(Error(view[0] as i64));
            }
            let created = services
                .request_interaction([wire::SURFACE_CREATE, view[1], 0, 0, 0, 0, 0, 0], None)?;
            if created[0] != 0 {
                return Err(Error(created[0] as i64));
            }
            let reply = services.request_interaction(
                [
                    wire::SURFACE_IMAGE,
                    view[1],
                    created[1],
                    0,
                    0,
                    image.width,
                    image.height,
                    image.stride,
                ],
                Some(&image.object),
            )?;
            if reply[0] != 0 {
                return Err(Error(reply[0] as i64));
            }
            cathedral_user_runtime::write(
                b"Cathedral capture: masked result received and presented\n",
            )?;
            painted = true;
        }
        if let Some(Event::Request(request)) =
            server.next_event_or(&[], if painted { None } else { Some(time::after(5)?) })?
        {
            let reply = match wire::decode(&request.bytes[..request.len]) {
                Some([wire::LOSS | wire::HEALTH, 0, 0, 0, 0, 0, 0, 0]) => [0; 8],
                _ => wire::error(abi::DENIED),
            };
            server.reply(&request, &wire::encode(reply))?;
        }
    }
}
