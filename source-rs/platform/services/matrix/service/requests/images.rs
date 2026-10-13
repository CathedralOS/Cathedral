//! Authenticate the surface and buffer shape before forwarding authority. Commit
//! retained geometry only after upstream success, so rejection preserves the old image.
use super::*;
use cathedral_user_runtime::memory::Object;
pub(in super::super) fn image(
    request: Request,
    object: Object,
    parent: &mut Parent,
    world: &mut World,
    server: &mut Server,
) -> Result<(), Error> {
    let response = (|| {
        let words = wire::decode(&request.bytes[..request.len]).ok_or(abi::INVALID_ARGUMENT)?;
        if request.control
            || !matches!(request.client, Some(0 | 1))
            || words[0] != wire::SURFACE_IMAGE
        {
            return Err(abi::DENIED);
        }
        if !cathedral_contracts::composition::valid_image(
            words[5],
            words[6],
            words[7],
            object.len(),
        ) {
            return Err(abi::INVALID_ARGUMENT);
        }
        let slot = request.client.unwrap();
        let prepared = world.preview_surface(slot, request.incarnation, words)?;
        let reply = parent.image(prepared, object).map_err(|e| e.0 as u64)?;
        if reply[0] == 0 {
            world
                .surface(slot, request.incarnation, words)
                .expect("validated image update");
        }
        Ok(reply)
    })()
    .unwrap_or_else(wire::error);
    server.reply(&request, &wire::encode(response))
}
