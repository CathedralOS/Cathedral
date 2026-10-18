// List the realm, present users/apps, then launch a choice or copy the template.
// Labels and hidden entries are distribution presentation; every request stays
// within the scoped object identities returned by this child's realm.
use cathedral_kernel_contracts::user as abi;
use cathedral_platform_contracts::{input, matrix as wire};
use cathedral_user_runtime::{
    Error,
    ipc::{self, Handle},
    task::Environment,
    time,
};
pub type Pair = (Handle, Handle);
/// Bounded realm listing; presentation hides the template/chrome at index zero.
pub const ENTRIES: usize = 4;
pub const NEW: usize = ENTRIES;

pub fn run(environment: Environment, user_picker: bool) -> u64 {
    let connections = cathedral_user_runtime::matrix::Connections::load(&environment).unwrap();
    let parent = connections.interaction;
    let control = (Handle::bootstrap(0).unwrap(), Handle::bootstrap(1).unwrap());
    let mut realm = realm::Realm::new(connections.storage, user_picker);
    let mut status = match realm.reload() {
        Ok(()) => "",
        Err(_) => "CANNOT OPEN",
    };
    let (mut selected, mut pressed) = (1usize, false);
    let mut scene: Option<view::Scene> = None;
    let mut repaint = true;
    loop {
        let snapshot = call(parent, [wire::POLL, 0, 0, 0, 0, 0, 0, 0]);
        let extent = wire::poll_view(snapshot);
        if extent[0] == 0 {
            let event = wire::poll_event(snapshot);
            let mut choose = false;
            match event[2] {
                wire::POINTER => {
                    let down = event[5] & 1 != 0;
                    if down
                        && !pressed
                        && let Some(index) =
                            layout::hit(extent, realm.count, user_picker, event[3], event[4])
                    {
                        selected = index;
                        choose = true;
                    }
                    pressed = down;
                }
                wire::CANCEL => pressed = false,
                wire::KEY if event[4] == input::PRESS as u64 => match event[3] as u8 {
                    input::UP | input::LEFT => {
                        selected = layout::step(selected, realm.count, user_picker, false);
                        repaint = true;
                    }
                    input::DOWN | input::RIGHT => {
                        selected = layout::step(selected, realm.count, user_picker, true);
                        repaint = true;
                    }
                    input::ENTER => choose = true,
                    _ => (),
                },
                _ => (),
            }
            if choose {
                status = act(&mut realm, parent, selected, user_picker);
                if status.is_empty() && selected == NEW {
                    selected = realm.count.saturating_sub(1);
                }
                repaint = true;
            }
            if scene.as_ref().is_none_or(|s| s.epoch != extent[1]) {
                scene = Some(view::Scene::new(parent, extent[1]));
                repaint = true;
            }
            if repaint {
                scene.as_ref().unwrap().paint(
                    parent,
                    extent,
                    &realm,
                    selected,
                    status,
                    user_picker,
                );
                repaint = false;
            }
        } else {
            scene = None;
        }
        if snapshot[5] & wire::MORE != 0 {
            continue;
        }
        let mut bytes = [0; 64];
        match ipc::wait_many(&[connections.wake, control.0], time::after(100).unwrap()) {
            Ok(0) => {
                connections.wake.receive(&mut bytes).unwrap();
            }
            Ok(_) => {
                // The owner's liveness probe: alive, with nothing to lose.
                let len = control.0.receive(&mut bytes).unwrap();
                let reply = match wire::decode(&bytes[..len]) {
                    Some([wire::HEALTH, 0, 0, 0, 0, 0, 0, 0]) => [0; 8],
                    Some([wire::LOSS, 0, 0, 0, 0, 0, 0, 0]) => [0; 8],
                    _ => wire::error(abi::DENIED),
                };
                control.1.send(&wire::encode(reply)).unwrap();
            }
            Err(Error(code)) if code == abi::TIMED_OUT as i64 => (),
            Err(error) => panic!("chooser wait {error:?}"),
        }
    }
}
/// Launch a visible app/user, or copy the hidden user template.
fn act(realm: &mut realm::Realm, parent: Pair, selected: usize, user_picker: bool) -> &'static str {
    if user_picker && selected == NEW && realm.count < ENTRIES {
        return match realm.copy_first() {
            Ok(()) => "",
            Err(abi::NO_MEMORY) => "NO ROOM",
            Err(_) => "TRY AGAIN",
        };
    }
    if selected == 0 || selected >= realm.count {
        return "";
    }
    match call(
        parent,
        [wire::LAUNCH, realm.ids[selected], 0, 0, 0, 0, 0, 0],
    )[0]
    {
        0 => "",
        abi::BUSY => "CLOSE APP FIRST",
        _ => "CANNOT OPEN",
    }
}
fn call(pair: Pair, words: wire::Packet) -> wire::Packet {
    let mut bytes = [0; 64];
    pair.0.send(&wire::encode(words)).unwrap();
    let len = pair
        .1
        .receive_until(&mut bytes, time::after(600).unwrap())
        .unwrap_or_else(|error| panic!("chooser request {words:?}: {error:?}"));
    wire::decode(&bytes[..len]).unwrap()
}
