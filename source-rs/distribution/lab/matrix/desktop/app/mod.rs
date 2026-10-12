//! App knows only its granted local extent, events and parent surface connection.
mod surfaces;
use super::*;
use cathedral_user_runtime::task::Environment;
pub(super) const LEFT: u64 = 0x315ca8;
pub(super) const RIGHT: u64 = 0x2c8762;
pub fn run(style: u64, environment: Environment) -> u64 {
    assert_eq!(environment.len(), 3);
    assert!(Launch::at(0).is_err());
    assert!(cathedral_user_runtime::link::Link::at(0).is_err());
    assert!(cathedral_user_runtime::display::mapping().is_err());
    let parent = (
        environment.endpoint(0).unwrap(),
        environment.endpoint(1).unwrap(),
    );
    let control = (Handle::bootstrap(0).unwrap(), Handle::bootstrap(1).unwrap());
    let mut alternate = false;
    let wake = environment.endpoint(2).unwrap();
    let mut painted = None;
    let mut scene: Option<surfaces::Scene> = None;
    let mut paints = 0;
    let mut wakes = 0;
    loop {
        let mut snapshot = call(parent, [wire::POLL, 0, 0, 0, 0, 0, 0, 0]);
        let mut view = wire::poll_view(snapshot);
        if view[0] == 0 {
            for _ in 0..8 {
                let event = wire::poll_event(snapshot);
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
                    #[cfg(feature = "trace-input")]
                    time::trace("app-key");
                    alternate = !alternate;
                    cathedral_user_runtime::write(if style == 0 {
                        b"Cathedral matrix app: nested key\n"
                    } else {
                        b"Cathedral matrix app: sibling key\n"
                    })
                    .unwrap();
                }
                if snapshot[5] & wire::MORE == 0 {
                    break;
                }
                snapshot = call(parent, [wire::POLL, 0, 0, 0, 0, 0, 0, 0]);
                view = wire::poll_view(snapshot);
            }
            let color = if alternate {
                0xd18b35
            } else if style == 0 {
                LEFT
            } else {
                RIGHT
            };
            if painted != Some((view, color)) {
                if scene.as_ref().is_none_or(|s| s.epoch != view[1]) {
                    scene = Some(surfaces::Scene::new(parent, view));
                }
                scene.as_ref().unwrap().paint(parent, view, color);
                painted = Some((view, color));
                paints += 1;
                #[cfg(feature = "trace-input")]
                time::trace("app-painted");
            }
        } else {
            painted = None;
            scene = None;
        }
        let mut bytes = [0; 64];
        loop {
            match ipc::wait_many(&[wake, control.0], time::after(100).unwrap()) {
                Ok(0) => {
                    wake.receive(&mut bytes).unwrap();
                    wakes += 1;
                    break;
                }
                Ok(_) => {
                    let len = control.0.receive(&mut bytes).unwrap();
                    if let Some(words) = wire::decode(&bytes[..len]) {
                        match words[0] {
                            1 => scene.as_mut().unwrap().remove_accent(parent),
                            2 => scene.as_mut().unwrap().restore_accent(parent),
                            _ => (),
                        }
                    }
                    control
                        .1
                        .send(&wire::encode([
                            0,
                            paints,
                            wakes,
                            time::cpu_us().unwrap(),
                            0,
                            0,
                            0,
                            0,
                        ]))
                        .unwrap();
                }
                Err(Error(code)) if code == abi::TIMED_OUT as i64 => (),
                Err(error) => panic!("matrix app wait {error:?}"),
            }
        }
    }
}
