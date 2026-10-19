//! Paint on grant changes, then sleep. The owner can replace this program without
//! changing the compositor; this process owns no input keys, storage or children.
use cathedral_platform_contracts::matrix as wire;
use cathedral_user_runtime::{
    Error,
    ipc::{self, Handle},
    matrix::upstream,
    services::Services,
    time,
};
pub fn run(mut services: Services) -> Result<(), Error> {
    let interaction = services.interaction(1, 1)?;
    assert!(
        matches!(services.storage(1, 1), Err(Error(code)) if code == cathedral_platform_contracts::services::UNAVAILABLE as i64)
    );
    let control = (Handle::bootstrap(0)?, Handle::bootstrap(1)?);
    let mut epoch = 0;
    let mut surface = 0;
    loop {
        let view = upstream::call(interaction.pair, [wire::POLL, 0, 0, 0, 0, 0, 0, 0])?;
        if view[0] == 0 && epoch != view[1] {
            epoch = view[1];
            if surface == 0 {
                let reply = upstream::call(
                    interaction.pair,
                    [wire::SURFACE_CREATE, epoch, 0, 0, 0, 0, 0, 0],
                )?;
                if reply[0] != 0 {
                    return Err(Error(reply[0] as i64));
                }
                surface = reply[1];
            }
            let reply = upstream::call(
                interaction.pair,
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
            )?;
            if reply[0] != 0 {
                return Err(Error(reply[0] as i64));
            }
        }
        match ipc::wait_many(&[control.0, interaction.wake], time::after(100)?) {
            Ok(1) => {
                interaction.wake.receive_until(&mut [0; 64], time::now()?)?;
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
