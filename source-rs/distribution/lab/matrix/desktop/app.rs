//! App knows only its granted local extent, events and parent surface connection.
use super::*;
use cathedral_user_runtime::task::Environment;
pub(super) const LEFT: u64 = 0x315ca8;
pub(super) const RIGHT: u64 = 0x2c8762;
pub fn run(style: u64, environment: Environment) -> u64 {
    assert_eq!(environment.len(), 2);
    assert!(Launch::at(0).is_err());
    assert!(cathedral_user_runtime::link::Link::at(0).is_err());
    assert!(cathedral_user_runtime::display::mapping().is_err());
    let parent = (
        environment.endpoint(0).unwrap(),
        environment.endpoint(1).unwrap(),
    );
    let control = (Handle::bootstrap(0).unwrap(), Handle::bootstrap(1).unwrap());
    let mut alternate = false;
    loop {
        let view = call(parent, [wire::VIEW, 0, 0, 0, 0, 0, 0, 0]);
        if view[0] == 0 {
            for _ in 0..8 {
                let event = call(parent, [wire::NEXT, view[1], 0, 0, 0, 0, 0, 0]);
                if event[0] != 0 || event[2] == wire::NONE {
                    break;
                }
                if event[2] == wire::POINTER {
                    assert!(event[3] < view[2] && event[4] < view[3]);
                }
                if event[2] == wire::KEY
                    && event[3] == input::ENTER as u64
                    && event[4] == input::PRESS as u64
                {
                    alternate = !alternate;
                    cathedral_user_runtime::write(if style == 0 {
                        b"Cathedral matrix app: nested key\n"
                    } else {
                        b"Cathedral matrix app: sibling key\n"
                    })
                    .unwrap();
                }
            }
            let color = if alternate {
                0xd18b35
            } else if style == 0 {
                LEFT
            } else {
                RIGHT
            };
            let _ = call(
                parent,
                [
                    wire::PAINT,
                    view[1],
                    0,
                    0,
                    0,
                    view[2] + 99,
                    view[3] + 99,
                    color,
                ],
            );
        }
        let mut bytes = [0; 64];
        if control
            .0
            .receive_until(&mut bytes, time::after(2).unwrap())
            .is_ok()
        {
            control.1.send(&wire::encode([0; 8])).unwrap();
        }
    }
}
