// Load local connections, reconcile durable state, handle editing, then present.
// Storage owns the IPC transaction; view owns immutable bitmap publication.
use cathedral_kernel_contracts::user as abi;
use cathedral_notes::document::Document;
use cathedral_platform_contracts::{input, matrix as wire};
use cathedral_user_runtime::{
    Error,
    ipc::{self, Handle},
    task::Environment,
    time,
};
type Pair = (Handle, Handle);
fn call(pair: Pair, words: wire::Packet) -> wire::Packet {
    pair.0.send(&wire::encode(words)).unwrap();
    let mut bytes = [0; 64];
    let n = pair
        .1
        .receive_until(&mut bytes, time::after(300).unwrap())
        .unwrap();
    wire::decode(&bytes[..n]).unwrap()
}
pub fn run(style: u64, environment: Environment) {
    let connections = cathedral_user_runtime::matrix::Connections::load(&environment).unwrap();
    let parent = connections.interaction;
    assert_eq!(
        call(parent, [wire::STORAGE_STATUS, 0, 0, 0, 0, 0, 0, 0]),
        wire::error(abi::DENIED)
    );
    let control = (Handle::bootstrap(0).unwrap(), Handle::bootstrap(1).unwrap());
    let mut catalog = storage::Catalog::new(connections.storage, style);
    let mut draft = Document::default();
    let mut confirmed = draft;
    let mut error = 0;
    let mut loaded = false;
    let mut cursor = 0;
    let mut scene: Option<view::Scene> = None;
    let mut repaint = true;
    loop {
        let snapshot = call(parent, [wire::POLL, 0, 0, 0, 0, 0, 0, 0]);
        let extent = wire::poll_view(snapshot);
        if extent[0] == 0 {
            if !loaded {
                match catalog.load() {
                    Ok(doc) => {
                        confirmed = doc;
                        draft = doc;
                        cursor = doc.len;
                    }
                    Err(code) => error = code,
                }
                loaded = true;
                time::storage_sample(
                    "note-loaded",
                    &[style, catalog.generation, draft.len as u64],
                );
            }
            let event = wire::poll_event(snapshot);
            if event[2] == wire::KEY && event[4] != input::RELEASE as u64 {
                let key = event[3] as u8;
                if matches!(key, input::F1 | input::F2) && event[4] == input::PRESS as u64 {
                    if key == input::F2 || error != 0 {
                        match catalog.load() {
                            Ok(doc) => {
                                confirmed = doc;
                                error = 0;
                                if key == input::F2 {
                                    draft = doc;
                                    cursor = doc.len;
                                }
                            }
                            Err(code) => error = code,
                        }
                    } else if draft != confirmed || catalog.generation == 0 {
                        match catalog.save(&draft) {
                            Ok(()) => {
                                confirmed = draft;
                                time::storage_sample(
                                    "note-saved",
                                    &[style, catalog.generation, draft.len as u64],
                                );
                            }
                            Err(code) => error = code,
                        }
                    }
                    repaint = true;
                } else {
                    repaint |= draft.edit(&mut cursor, key);
                }
            }
            if scene.as_ref().is_none_or(|s| s.epoch != extent[1]) {
                scene = Some(view::Scene::new(parent, extent[1]));
                repaint = true;
            }
            if repaint {
                scene.as_ref().unwrap().paint(
                    parent,
                    extent,
                    &draft,
                    cursor,
                    draft != confirmed,
                    error,
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
                control.0.receive(&mut bytes).unwrap();
                control
                    .1
                    .send(&wire::encode([
                        0,
                        catalog.generation,
                        draft.len as u64,
                        error,
                        u64::from(draft != confirmed),
                        0,
                        0,
                        0,
                    ]))
                    .unwrap();
            }
            Err(Error(code)) if code == abi::TIMED_OUT as i64 => (),
            Err(e) => panic!("notes wait {e:?}"),
        }
    }
}
