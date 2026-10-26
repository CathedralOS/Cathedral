//! Paint on grant changes, then sleep. The owner can replace this program without
//! changing the compositor; this process owns no input keys, storage or children.
use cathedral_platform_contracts::matrix as wire;
use cathedral_user_runtime::{
    Error,
    server::{OwnerControl, SharedSlot},
    services::Services,
    time,
};
pub fn run(mut services: Services) -> Result<(), Error> {
    services.enable_shared()?;
    let interaction = services.interaction(1, 1)?;
    static mut CONTROL: SharedSlot = SharedSlot::empty();
    let slot = &raw mut CONTROL;
    // SAFETY: One entry lends this process-owned slot until exit.
    let mut control = OwnerControl::open(unsafe { &mut *slot })?;
    let mut epoch = 0;
    let mut surface = 0;
    let mut checked_scope = false;
    loop {
        let view = services.request_interaction([wire::POLL, 0, 0, 0, 0, 0, 0, 0], None)?;
        if !checked_scope {
            // Exercise a refused lazy acquisition after interaction has begun.
            // It must retain this broker's sequence and leave drawing usable.
            assert!(
                matches!(services.storage(1, 1), Err(Error(code)) if code == cathedral_platform_contracts::services::UNAVAILABLE as i64)
            );
            checked_scope = true;
            cathedral_user_runtime::write(
                b"Cathedral background: shared interaction ready; storage denied\n",
            )?;
        }
        if view[0] == 0 && epoch != view[1] {
            epoch = view[1];
            if surface == 0 {
                let reply = services
                    .request_interaction([wire::SURFACE_CREATE, epoch, 0, 0, 0, 0, 0, 0], None)?;
                if reply[0] != 0 {
                    return Err(Error(reply[0] as i64));
                }
                surface = reply[1];
            }
            let reply = services.request_interaction(
                [
                    wire::PAINT,
                    epoch,
                    surface,
                    0,
                    0,
                    view[2],
                    view[3],
                    0x101827,
                ],
                None,
            )?;
            if reply[0] != 0 {
                return Err(Error(reply[0] as i64));
            }
        }
        match control.next(interaction.wake.interest(), Some(time::after(100)?))? {
            None => {
                interaction.wake.take()?;
            }
            Some((bytes, len)) => {
                let reply = match wire::decode(&bytes[..len]) {
                    Some([wire::HEALTH | wire::LOSS, 0, 0, 0, 0, 0, 0, 0]) => [0; 8],
                    _ => wire::error(cathedral_kernel_contracts::user::DENIED),
                };
                control.reply(&wire::encode(reply))?;
            }
        }
    }
}
