// Load local connections, reconcile durable state, handle editing, then present.
// Storage owns the object exchanges; view owns immutable bitmap publication.
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
/// Owner control reply: revision, length, last error, dirty, note count, note index.
fn report(control: Pair, notes: &storage::Notes, draft: &Document, dirty: bool, error: u64) {
    let _ = control.1.send(&wire::encode([
        0,
        notes.revision,
        draft.len as u64,
        error,
        u64::from(dirty),
        notes.count as u64,
        if notes.count == 0 {
            0
        } else {
            notes.current as u64 + 1
        },
        0,
    ]));
}
pub fn run(style: u64, environment: Environment) {
    let connections =
        cathedral_user_runtime::matrix::OptionalConnections::load(&environment).unwrap();
    let parent = connections.interaction;
    if connections.storage.is_none() {
        // The interactive endpoint cannot substitute for the omitted service.
        assert_eq!(
            call(parent, [wire::STORAGE_INFO, 0, 0, 0, 0, 0, 0, 0]),
            wire::error(abi::DENIED)
        );
        cathedral_user_runtime::write(
            b"Cathedral Notes: temporary; no storage endpoint; interaction storage denied\n",
        )
        .unwrap();
    }

    assert_eq!(
        call(parent, [wire::STORAGE_STATUS, 0, 0, 0, 0, 0, 0, 0]),
        wire::error(abi::DENIED)
    );
    // The owner may retire this control link; the app keeps serving its world.
    let mut control = Some((Handle::bootstrap(0).unwrap(), Handle::bootstrap(1).unwrap()));
    let mut notes = storage::Notes::new(connections.storage, style);
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
                match if notes.temporary() {
                    Ok(Document::default())
                } else {
                    notes.load()
                } {
                    Ok(doc) => {
                        confirmed = doc;
                        draft = doc;
                        cursor = doc.len;
                    }
                    Err(code) => error = code,
                }
                loaded = true;
                time::storage_sample("note-loaded", &[style, notes.revision, draft.len as u64]);
                // Page allowances, use and frame funding, so a refused allocation
                // is diagnosable from the serial log.
                if let (Ok(budget), Ok(physical)) = (
                    cathedral_user_runtime::resources::budget(),
                    cathedral_user_runtime::resources::physical_budget(),
                ) {
                    time::storage_sample(
                        "note-budget",
                        &[
                            budget.limits.private_pages,
                            budget.limits.reader_pages,
                            budget.used.private_pages,
                            budget.used.reader_pages,
                            physical.reserved,
                            physical.charged,
                        ],
                    );
                }
            }
            let event = wire::poll_event(snapshot);
            if event[2] == wire::KEY && event[4] != input::RELEASE as u64 {
                let key = event[3] as u8;
                let function = matches!(key, input::F1 | input::F2 | input::F3 | input::F4);
                if function && event[4] == input::PRESS as u64 {
                    let dirty = draft != confirmed;
                    let outcome = if notes.temporary() {
                        Err(abi::DENIED) // No save/reload action may discard a temporary draft.
                    } else {
                        match key {
                            // F1 saves a draft; on an unresolved error it reads first,
                            // keeping the draft so a lost acknowledgement never
                            // repeats a write blindly.
                            input::F1 if error == 0 && (dirty || notes.revision == 0) => {
                                notes.save(&draft).map(|()| {
                                    confirmed = draft;
                                    time::storage_sample(
                                        "note-saved",
                                        &[style, notes.revision, draft.len as u64],
                                    );
                                })
                            }
                            input::F1 | input::F2 => notes.load().map(|doc| {
                                confirmed = doc;
                                if key == input::F2 {
                                    draft = doc;
                                    cursor = doc.len;
                                }
                            }),
                            // Duplicate and switch act on confirmed notes only.
                            input::F3 if error == 0 && !dirty && notes.revision != 0 => {
                                notes.duplicate().map(|doc| {
                                    confirmed = doc;
                                    draft = doc;
                                    cursor = doc.len;
                                })
                            }
                            input::F4 if error == 0 && !dirty && notes.count > 1 => {
                                notes.next().map(|doc| {
                                    confirmed = doc;
                                    draft = doc;
                                    cursor = doc.len;
                                })
                            }
                            _ => Ok(()),
                        }
                    };
                    error = outcome.err().unwrap_or(0);
                    repaint = true;
                } else if !function {
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
                    &notes,
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
        let mut handles = [connections.wake; 2];
        let mut count = 1;
        if let Some(control) = control {
            handles[1] = control.0;
            count = 2;
        }
        match ipc::wait_many(&handles[..count], time::after(100).unwrap()) {
            Ok(0) => {
                connections.wake.receive(&mut bytes).unwrap();
            }
            Ok(_) => {
                let pair = control.unwrap();
                match pair.0.receive(&mut bytes) {
                    // Closing asks what would be lost: an unconfirmed draft.
                    Ok(len)
                        if wire::decode(&bytes[..len])
                            .is_some_and(|words| words[0] == wire::LOSS) =>
                    {
                        let _ = pair.1.send(&wire::encode([
                            0,
                            u64::from(draft != confirmed),
                            0,
                            0,
                            0,
                            0,
                            0,
                            0,
                        ]));
                    }
                    Ok(_) => report(pair, &notes, &draft, draft != confirmed, error),
                    Err(_) => control = None,
                }
            }
            Err(Error(code)) if code == abi::TIMED_OUT as i64 => (),
            Err(e) => panic!("notes wait {e:?}"),
        }
    }
}
