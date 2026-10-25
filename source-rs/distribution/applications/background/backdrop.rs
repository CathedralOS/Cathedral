//! Paint on grant changes, then sleep. The owner can replace this program without
//! changing the compositor; this process owns no input keys, storage or children.
use cathedral_platform_contracts::matrix as wire;
use cathedral_user_runtime::{
    Error,
    event::{self, Interest},
    ipc::Handle,
    services::Services,
    time,
};
pub fn run(mut services: Services) -> Result<(), Error> {
    services.enable_shared()?;
    let interaction = services.interaction(1, 1)?;
    let control = (Handle::bootstrap(0)?, Handle::bootstrap(1)?);
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
        match event::wait(
            &[Interest::Ipc(control.0), interaction.wake.interest()],
            time::after(100)?,
        ) {
            Ok(1) => {
                interaction.wake.take()?;
            }
            Ok(_) => {
                let mut bytes = [0; 64];
                let len = control.0.receive(&mut bytes)?;
                let reply = match wire::decode(&bytes[..len]) {
                    Some([wire::HEALTH | wire::LOSS, 0, 0, 0, 0, 0, 0, 0]) => [0; 8],
                    _ => wire::error(cathedral_kernel_contracts::user::DENIED),
                };
                control.1.send(&wire::encode(reply))?;
            }
            Err(Error(code)) if code == cathedral_kernel_contracts::user::TIMED_OUT as i64 => (),
            Err(error) => return Err(error),
        }
    }
}
