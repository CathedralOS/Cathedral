//! The explicitly installed source endpoint supplies two logical channels in one
//! focus group. These are this adapter's local names, not global input roles.
use super::*;
const POINTER: u32 = 1;
const KEYBOARD: u32 = 2;
const GROUP: u32 = 1;

fn bind(world: &mut World) -> Result<(), u64> {
    world.bind_input_stream(POINTER, GROUP)?;
    world.bind_input_stream(KEYBOARD, GROUP)
}
pub(in crate::service) fn cancel(world: &mut World) {
    if bind(world).is_ok() {
        let _ = world.cancel_stream(POINTER);
        let _ = world.cancel_stream(KEYBOARD);
    }
}
pub(super) fn dispatch(
    parent: &mut Parent,
    world: &mut World,
    words: wire::Packet,
) -> Result<wire::Packet, Error> {
    if let Err(code) = bind(world) {
        return Ok(wire::error(code));
    }
    let result = match words {
        [wire::POINTER_SOURCE, x, y, buttons @ 0..=7, 0, 0, 0, 0]
            if x < parent.grant.width && y < parent.grant.height =>
        {
            parent.pointer(x, y)?;
            world.pointer_stream(POINTER, x, y, buttons)
        }
        [wire::KEY_SOURCE, key, state, 0, 0, 0, 0, 0] => {
            if key == cathedral_platform_contracts::input::RESET as u64 {
                cancel(world);
                Ok(())
            } else {
                world.key_stream(KEYBOARD, key, state)
            }
        }
        _ => Err(abi::DENIED),
    };
    Ok(wire::error(result.err().unwrap_or(0)))
}
