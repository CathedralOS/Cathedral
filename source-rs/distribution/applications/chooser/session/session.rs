// List the realm, run the chosen entry as a sibling, copy the first entry as
// a new one, or ask the host to close the sibling. The chooser holds no
// authority of its own beyond its realm: launching is the host's act, bounded
// to what this program can see. Entries are object numbers in the order the
// realm lists them; naming is an open owner question, not something this
// program invents.
use cathedral_kernel_contracts::user as abi;
use cathedral_platform_contracts::{input, matrix as wire};
use cathedral_user_runtime::{
    Error,
    ipc::{self, Handle},
    task::Environment,
    time,
};
pub type Pair = (Handle, Handle);
/// Rows: the first entries of the realm, then NEW, then CLOSE, then the status.
pub const ENTRIES: usize = 4;
pub const NEW: usize = ENTRIES;
pub const CLOSE: usize = ENTRIES + 1;
pub const ROWS: usize = ENTRIES + 3;

pub fn run(environment: Environment, user_picker: bool) -> u64 {
    let connections = cathedral_user_runtime::matrix::Connections::load(&environment).unwrap();
    let parent = connections.interaction;
    let control = (Handle::bootstrap(0).unwrap(), Handle::bootstrap(1).unwrap());
    let mut realm = realm::Realm::new(connections.storage);
    let mut status = match realm.reload() {
        Ok(()) => "CHOOSE",
        Err(_) => "NO REALM",
    };
    let (mut selected, mut pressed) = (0usize, false);
    let mut scene: Option<view::Scene> = None;
    let mut repaint = true;
    let mut without_storage = false;
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
                        && (view::LEFT..view::LEFT + view::WIDTH as u64).contains(&event[3])
                        && (view::ENTRY_TOP..view::ENTRY_TOP + view::ROW * (CLOSE as u64 + 1))
                            .contains(&event[4])
                    {
                        selected = ((event[4] - view::ENTRY_TOP) / view::ROW) as usize;
                        choose = true;
                    }
                    pressed = down;
                }
                wire::CANCEL => pressed = false,
                wire::KEY if event[4] == input::PRESS as u64 => match event[3] as u8 {
                    input::UP => {
                        selected = if selected == 0 { CLOSE } else { selected - 1 };
                        repaint = true;
                    }
                    input::DOWN => {
                        selected = if selected == CLOSE { 0 } else { selected + 1 };
                        repaint = true;
                    }
                    input::ENTER => choose = true,
                    _ => (),
                },
                _ => (),
            }
            if choose {
                status = act(
                    &mut realm,
                    parent,
                    selected,
                    user_picker,
                    &mut without_storage,
                );
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
                    without_storage,
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
/// Carry out the selected row and say what happened.
fn act(
    realm: &mut realm::Realm,
    parent: Pair,
    selected: usize,
    user_picker: bool,
    without_storage: &mut bool,
) -> &'static str {
    match selected {
        row if row < realm.count => {
            match call(
                parent,
                [
                    wire::LAUNCH,
                    realm.ids[row],
                    if *without_storage {
                        cathedral_platform_contracts::imports::STORAGE
                    } else {
                        0
                    },
                    0,
                    0,
                    0,
                    0,
                    0,
                ],
            )[0]
            {
                0 => "RUNNING",
                abi::BUSY => "BUSY",
                abi::DENIED => "NOT OPEN",
                cathedral_platform_contracts::imports::MISSING => "NEEDS SERVICE",
                cathedral_platform_contracts::imports::INCOMPATIBLE => "BAD VERSION",
                _ => "REFUSED",
            }
        }
        NEW if !user_picker => {
            *without_storage = !*without_storage;
            if *without_storage {
                "TEMPORARY"
            } else {
                "PERSISTENT"
            }
        }
        NEW => match realm.copy_first() {
            Ok(()) => "COPIED",
            Err(abi::NO_MEMORY) => "FULL",
            Err(_) => "REFUSED",
        },
        CLOSE => match call(parent, [wire::CLOSE, 0, 0, 0, 0, 0, 0, 0])[0] {
            0 => "CLOSED",
            abi::BUSY => "KEPT",
            abi::PEER_CLOSED => "NOTHING OPEN",
            _ => "REFUSED",
        },
        _ => "CHOOSE",
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
