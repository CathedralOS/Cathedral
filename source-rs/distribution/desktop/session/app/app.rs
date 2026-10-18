// App knows only its granted local extent, events and parent surface connection.
// Enter toggles its color through its own private object. The owner may probe
// it over the control link; lab compositions add proof commands to that link,
// the shipped app answers every command with its liveness report.
use super::*;
use cathedral_user_runtime::task::Environment;
pub(crate) const LEFT: u64 = 0x315ca8;
pub(crate) const RIGHT: u64 = 0x2c8762;
/// What an owner command may act on: the scene, the state and the paint record.
#[cfg(not(feature = "shell"))]
pub(crate) struct Probe<'a> {
    pub style: u64,
    pub parent: Pair,
    pub control: Pair,
    pub view: wire::Packet,
    pub scene: &'a mut Option<surfaces::Scene>,
    pub state: &'a mut Option<state::State>,
    pub painted: &'a mut Option<(wire::Packet, u64, u64)>,
    pub save_status: &'a mut u64,
}
#[cfg(not(feature = "shell"))]
impl Probe<'_> {
    pub fn repaint(&mut self, record: bool) {
        let state = self.state.as_ref().unwrap();
        let color = state.color();
        self.scene
            .as_mut()
            .unwrap()
            .paint(self.parent, self.view, color, state.error);
        if record {
            *self.painted = Some((self.view, color, state.error));
        }
    }
}
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
    #[cfg(not(feature = "shell"))]
    let mut save_status = 0;
    #[cfg(feature = "shell")]
    let save_status = 0;
    let mut pointers = 0;
    #[cfg(feature = "shell")]
    let mut close = closing::Button::default();
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
                #[cfg(feature = "shell")]
                {
                    let previous = close.confirm;
                    if close.close(event, view[2], state.as_ref().unwrap().error != 0) {
                        return 0;
                    }
                    if previous != close.confirm {
                        painted = None;
                    }
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
                    let state = state.as_mut().unwrap();
                    let result = if state.error != 0 {
                        state.reload()
                    } else {
                        state.toggle()
                    };
                    if result.is_ok() {
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
            let error = state.as_ref().unwrap().error;
            if painted != Some((view, color, error)) {
                if scene.as_ref().is_none_or(|s| s.epoch != view[1]) {
                    scene = Some(surfaces::Scene::new(parent, view));
                }
                scene
                    .as_mut()
                    .unwrap()
                    .paint(parent, view, color, state.as_ref().unwrap().error);
                #[cfg(feature = "shell")]
                scene
                    .as_ref()
                    .unwrap()
                    .close_button(parent, view, close.confirm);
                painted = Some((view, color, state.as_ref().unwrap().error));
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
                    let command = wire::decode(&bytes[..len]).map_or(0, |words| words[0]);
                    // Closing asks what would be lost: a toggle whose save failed.
                    if command == wire::LOSS {
                        control
                            .1
                            .send(&wire::encode([
                                0,
                                u64::from(
                                    save_status != 0
                                        || state.as_ref().is_some_and(|s| s.error != 0),
                                ),
                                0,
                                0,
                                0,
                                0,
                                0,
                                0,
                            ]))
                            .unwrap();
                        continue;
                    }
                    // The shipped app answers any owner command with its liveness
                    // report; lab compositions act on proof commands first.
                    #[cfg(feature = "shell")]
                    let status = 0;
                    #[cfg(not(feature = "shell"))]
                    let Some(status) = crate::lab::command(
                        command,
                        &mut Probe {
                            style,
                            parent,
                            control,
                            view,
                            scene: &mut scene,
                            state: &mut state,
                            painted: &mut painted,
                            save_status: &mut save_status,
                        },
                    ) else {
                        continue;
                    };
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
                            state.as_ref().map_or(0, |s| s.revision),
                        ]))
                        .unwrap();
                }
                Err(Error(code)) if code == abi::TIMED_OUT as i64 => (),
                Err(error) => panic!("matrix app wait {error:?}"),
            }
        }
    }
}
