// App knows only its granted local extent, events and parent surface connection.
use super::*;
use cathedral_user_runtime::task::Environment;
pub(super) const LEFT: u64 = 0x315ca8;
pub(super) const RIGHT: u64 = 0x2c8762;
pub fn run(style: u64, environment: Environment) -> u64 {
    let connections = cathedral_user_runtime::matrix::Connections::load(&environment).unwrap();
    assert!(Launch::at(0).is_err());
    assert!(cathedral_user_runtime::link::Link::at(0).is_err());
    assert!(cathedral_user_runtime::display::mapping().is_err());
    let parent = connections.interaction;
    let storage = connections.storage;
    assert_eq!(
        call(parent, [wire::STORAGE_INFO, 0, 0, 0, 0, 0, 0, 0])[0],
        abi::DENIED
    );
    assert_eq!(
        call(storage, [wire::VIEW, 0, 0, 0, 0, 0, 0, 0])[0],
        abi::DENIED
    );
    assert_eq!(
        call(storage, [wire::POINTER_SOURCE, 0, 0, 0, 0, 0, 0, 0])[0],
        abi::DENIED
    );
    let control = (Handle::bootstrap(0).unwrap(), Handle::bootstrap(1).unwrap());
    let mut state: Option<state::State> = None;
    let wake = connections.wake;
    let mut painted = None;
    let mut scene: Option<surfaces::Scene> = None;
    let mut paints = 0;
    let mut wakes = 0;
    let mut save_status = 0;
    let mut pointers = 0;
    loop {
        let mut snapshot = call(parent, [wire::POLL, 0, 0, 0, 0, 0, 0, 0]);
        let mut view = wire::poll_view(snapshot);
        if view[0] == 0 {
            if state.is_none() {
                state = Some(state::State::open(storage, style));
            }
            for _ in 0..8 {
                let event = wire::poll_event(snapshot);
                if event[0] != 0 || event[2] == wire::NONE {
                    break;
                }
                if event[2] == wire::POINTER {
                    pointers += 1;
                    assert!(event[3] < view[2] && event[4] < view[3]);
                }
                if event[2] == wire::KEY
                    && event[3] == input::ENTER as u64
                    && event[4] == input::PRESS as u64
                {
                    #[cfg(feature = "trace-input")]
                    time::trace("app-key");
                    if state.as_mut().unwrap().toggle(storage).is_ok() {
                        cathedral_user_runtime::write(if style == 0 {
                            b"Cathedral matrix app: nested key\n"
                        } else if style == 1 {
                            b"Cathedral matrix app: sibling key\n"
                        } else {
                            b"Cathedral matrix app: nested probe key\n"
                        })
                        .unwrap();
                    }
                }
                if snapshot[5] & wire::MORE == 0 {
                    break;
                }
                snapshot = call(parent, [wire::POLL, 0, 0, 0, 0, 0, 0, 0]);
                view = wire::poll_view(snapshot);
            }
            let color = state.as_ref().unwrap().color();
            if painted != Some((view, color)) {
                if scene.as_ref().is_none_or(|s| s.epoch != view[1]) {
                    scene = Some(surfaces::Scene::new(parent, view));
                }
                scene.as_mut().unwrap().paint(parent, view, color);
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
                    let mut status = 0;
                    if let Some(words) = wire::decode(&bytes[..len]) {
                        match words[0] {
                            1 => scene.as_mut().unwrap().remove_accent(parent),
                            2 => scene.as_mut().unwrap().restore_accent(parent),
                            3 => scene.as_mut().unwrap().staged_probe(parent),
                            4 => scene.as_ref().unwrap().commit(parent),
                            5 => scene.as_mut().unwrap().rejected_probe(parent),
                            6 => scene.as_mut().unwrap().paint(
                                parent,
                                view,
                                if style == 0 { LEFT } else { RIGHT },
                            ),
                            7 => {
                                state = Some(state::State::open(storage, style));
                                scene.as_mut().unwrap().paint(
                                    parent,
                                    view,
                                    state.as_ref().unwrap().color(),
                                );
                            }
                            8 => {
                                status = state.as_mut().unwrap().toggle(storage).err().unwrap_or(0);
                                if status == 0 {
                                    let color = state.as_ref().unwrap().color();
                                    scene.as_mut().unwrap().paint(parent, view, color);
                                    painted = Some((view, color));
                                }
                            }
                            9 => {
                                state.as_ref().unwrap().stage(storage);
                                scene.as_mut().unwrap().staged_probe(parent);
                            }
                            10 => cathedral_user_runtime::task::exit(19),
                            11 => scene.as_mut().unwrap().compact_probe(parent),
                            12 => {
                                // Test owner gets admission acknowledgement before
                                // the save blocks; completion is observed via HEALTH.
                                control.1.send(&wire::encode([0; 8])).unwrap();
                                save_status =
                                    state.as_mut().unwrap().toggle(storage).err().unwrap_or(0);
                                if save_status == 0 {
                                    let color = state.as_ref().unwrap().color();
                                    scene.as_mut().unwrap().paint(parent, view, color);
                                    painted = Some((view, color));
                                }
                                continue;
                            }
                            _ => (),
                        }
                    }
                    control
                        .1
                        .send(&wire::encode([
                            status,
                            paints,
                            wakes,
                            time::cpu_us().unwrap(),
                            u64::from(state.as_ref().is_some_and(|s| s.alternate)),
                            save_status,
                            pointers,
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
