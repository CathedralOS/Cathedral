// The Rust probe app knows only its local extent, events and parent connection.
// Enter toggles its color through its own private object. The owner may probe
// it over the control link; lab compositions add proof commands to that link,
// non-probe sessions answer every command with their liveness report.
use cathedral_desktop_matrix_contracts::matrix as wire;
use cathedral_kernel_contracts::user as abi;
use cathedral_platform_contracts::input;
use cathedral_user_runtime::task::Environment;
use cathedral_user_runtime::{
    Error,
    connection::Client,
    server::{OwnerControl, SharedSlot},
    services::Services,
    storage::Scope,
    task::Launch,
    time,
};
pub const LEFT: u64 = 0x315ca8;
pub const RIGHT: u64 = 0x2c8762;
/// What an owner command may act on: the scene, the state and the paint record.
pub struct Probe<'a> {
    pub style: u64,
    pub parent: &'a mut Services,
    pub control: &'a mut OwnerControl,
    pub view: wire::Packet,
    pub scene: &'a mut Option<surfaces::Scene>,
    pub state: &'a mut Option<state::State>,
    pub painted: &'a mut Option<(wire::Packet, u64, u64)>,
    pub save_status: &'a mut u64,
}
impl Probe<'_> {
    pub fn repaint(&mut self, record: bool) {
        let state = self.state.as_ref().unwrap();
        let color = state.color();
        self.scene
            .as_mut()
            .unwrap()
            .paint(self.parent, self.view, color, state.error)
            .unwrap();
        if record {
            *self.painted = Some((self.view, color, state.error));
        }
    }
}
pub fn run<const PROBES: bool>(
    style: u64,
    environment: Environment,
    command: fn(u64, &mut Probe<'_>) -> Option<u64>,
) -> u64 {
    run_services::<PROBES>(
        style,
        cathedral_user_runtime::services::Services::load(&environment).unwrap(),
        command,
    )
}
pub fn run_services<const PROBES: bool>(
    style: u64,
    mut services: cathedral_user_runtime::services::Services,
    command: fn(u64, &mut Probe<'_>) -> Option<u64>,
) -> u64 {
    static mut CONTROL: SharedSlot = SharedSlot::empty();
    let slot = &raw mut CONTROL;
    // SAFETY: The single app entry owns this control slot until exit.
    let mut control = OwnerControl::open(unsafe { &mut *slot }).unwrap();
    // Admission starts this independent connection immediately. Establish it
    // before storage/rendering work can consume a tightly capped CPU share.
    control.establish(time::after(300).unwrap()).unwrap();
    // Apps have logical input only, never access to the physical pointer channel.
    assert_eq!(
        cathedral_user_runtime::keyboard::virtual_pointer(10, 0),
        Err(Error(abi::DENIED as i64))
    );
    services.enable_shared().unwrap();
    let interaction = services.interaction(1, 1).unwrap();
    let mut storage =
        Client::connect(services.storage(1, 1).unwrap(), time::after(300).unwrap()).unwrap();
    let wake = interaction.wake;
    cathedral_user_runtime::write(
        b"Cathedral Colors probe: shared interaction and storage ready\n",
    )
    .unwrap();
    assert!(Launch::at(0).is_err());
    assert!(cathedral_user_runtime::link::Link::at(0).is_err());
    assert!(cathedral_user_runtime::display::mapping().is_err());
    let parent = &mut services;
    assert_eq!(
        call(parent, [wire::APPLY_LAYOUT, 0, 24, 192, 16, 0, 0, 0])[0],
        abi::DENIED
    );
    assert_eq!(
        call(parent, [wire::STORAGE_INFO, 0, 0, 0, 0, 0, 0, 0])[0],
        abi::DENIED
    );
    for operation in [wire::VIEW, wire::POINTER_SOURCE] {
        let (bytes, n) = storage
            .call(
                &wire::encode([operation, 0, 0, 0, 0, 0, 0, 0]),
                time::after(300).unwrap(),
            )
            .unwrap();
        assert_eq!(wire::decode(&bytes[..n]), Some(wire::error(abi::DENIED)));
    }
    let mut scope = Some(Scope::from_client(storage).unwrap());
    let mut state: Option<state::State> = None;
    let mut painted = None;
    let mut scene: Option<surfaces::Scene> = None;
    let mut paints = 0;
    let mut wakes = 0;
    let mut save_status = 0;
    let mut pointers = 0;
    let mut close = closing::Button::default();
    loop {
        let mut snapshot = call(parent, [wire::POLL, 0, 0, 0, 0, 0, 0, 0]);
        let mut view = wire::poll_view(snapshot);
        if view[0] == 0 {
            if !PROBES && scene.as_ref().is_some_and(|s| s.epoch != view[1]) {
                close.rebind();
            }
            if state.is_none() {
                state = Some(state::State::open(scope.take().unwrap(), style));
            }
            for _ in 0..8 {
                let event = wire::poll_event(snapshot);
                if event[0] != 0 || event[2] == wire::NONE {
                    break;
                }
                if !PROBES {
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
            if wire::poll_frames(snapshot) && painted != Some((view, color, error)) {
                let rendered = (|| -> Result<(), u64> {
                    if scene.as_ref().is_none_or(|s| s.epoch != view[1]) {
                        scene = Some(surfaces::Scene::new::<PROBES>(parent, view)?);
                    }
                    scene.as_mut().unwrap().paint(
                        parent,
                        view,
                        color,
                        state.as_ref().unwrap().error,
                    )?;
                    if !PROBES {
                        scene
                            .as_ref()
                            .unwrap()
                            .close_button(parent, view, close.confirm)?;
                    }
                    Ok(())
                })();
                if let Err(status) = rendered {
                    let now = call(parent, [wire::VIEW, 0, 0, 0, 0, 0, 0, 0]);
                    assert!(
                        now[0] != 0 || now[1] != view[1],
                        "color drawing refused: {status}"
                    );
                    scene = None;
                    close.rebind();
                    painted = None;
                    continue;
                }
                painted = Some((view, color, state.as_ref().unwrap().error));
                paints += 1;
                #[cfg(feature = "trace-input")]
                time::trace("app-painted");
            }
        } else {
            painted = None;
            scene = None;
        }
        loop {
            match control.next(wake.interest(), Some(time::after(100).unwrap())) {
                Ok(None) => {
                    if wake.take().unwrap() {
                        wakes += 1;
                        break;
                    }
                }
                Ok(Some((bytes, len))) => {
                    let operation = wire::decode(&bytes[..len]).map_or(0, |words| words[0]);
                    // Closing asks what would be lost: a toggle whose save failed.
                    if operation == wire::LOSS {
                        control
                            .reply(&wire::encode([
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
                    // A non-probe session answers owner commands with its liveness
                    // report; lab compositions act on proof commands first.
                    let status = if PROBES {
                        let Some(status) = command(
                            operation,
                            &mut Probe {
                                style,
                                parent,
                                control: &mut control,
                                view,
                                scene: &mut scene,
                                state: &mut state,
                                painted: &mut painted,
                                save_status: &mut save_status,
                            },
                        ) else {
                            continue;
                        };
                        status
                    } else {
                        0
                    };
                    control
                        .reply(&wire::encode([
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

fn call(parent: &mut Services, words: wire::Packet) -> wire::Packet {
    parent
        .request_interaction(words, None)
        .unwrap_or_else(|error| panic!("Colors request {words:?}: {error:?}"))
}
