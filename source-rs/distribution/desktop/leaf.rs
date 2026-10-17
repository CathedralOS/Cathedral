//! Test peer relays probes over its sole parent endpoint. No world/global lookup.
//! A relayed publication carries its bytes inline in the owner packet; the leaf
//! turns them into the sealed attachment the real protocol requires. A detached
//! probe is answered before it is relayed, so the leaf can ask its host for
//! something the host answers only after talking to the owner's other children;
//! the result waits for the next LAST_PROBE.
use super::*;
use cathedral_platform_contracts::storage;
use cathedral_user_runtime::{ipc, link::Link, memory::Private, task::Launch};
pub const EXIT_PROBE: u64 = 0xfffe;
pub const DETACHED_PROBE: u64 = 0xfffd;
pub const LAST_PROBE: u64 = 0xfffc;
pub fn run(environment: cathedral_user_runtime::task::Environment) -> u64 {
    assert_eq!(environment.len(), 3);
    assert!(matches!(Launch::at(0), Err(Error(code)) if code == abi::DENIED as i64));
    assert!(Link::at(0).is_err());
    assert_eq!(
        cathedral_user_runtime::display::mapping(),
        Err(Error(abi::DENIED as i64))
    );
    assert_eq!(
        cathedral_user_runtime::keyboard::read(false),
        Err(Error(abi::DENIED as i64))
    );
    assert_eq!(
        cathedral_user_runtime::disk::read(7),
        Err(Error(abi::DENIED as i64))
    );
    let control = (Handle::bootstrap(0).unwrap(), Handle::bootstrap(1).unwrap());
    let parent = (
        environment.endpoint(0).unwrap(),
        environment.endpoint(1).unwrap(),
    );
    let mut storage_epoch = 0;
    let mut last = wire::error(abi::WOULD_BLOCK);
    loop {
        let mut bytes = [0; 64];
        let Ok(len) = control.0.receive(&mut bytes) else {
            return 0;
        };
        let Some(mut words) = wire::decode(&bytes[..len]) else {
            return 1;
        };
        if words == [EXIT_PROBE, 0, 0, 0, 0, 0, 0, 0] {
            return 0;
        }
        if words == [LAST_PROBE, 0, 0, 0, 0, 0, 0, 0] {
            control.1.send(&wire::encode(last)).unwrap();
            continue;
        }
        // The host's close question is the leaf's own answer: nothing to lose.
        // Everything else, a liveness probe included, is relayed to the parent
        // so a fixture sees exactly what the matrix answers a child.
        if words == [wire::LOSS, 0, 0, 0, 0, 0, 0, 0] {
            control.1.send(&wire::encode([0; 8])).unwrap();
            continue;
        }
        if words[0] == DETACHED_PROBE {
            control.1.send(&wire::encode([0; 8])).unwrap();
            let mut detached = [0; 8];
            detached[..7].copy_from_slice(&words[1..]);
            last = exchange(parent, detached, None);
            continue;
        }
        let stored = (wire::STORAGE_BASE..wire::STORAGE_BASE + 256).contains(&words[0]);
        let mut attachment = None;
        if stored {
            if words[0] == wire::STORAGE_BASE + storage::PUBLISH {
                let mut pages = Private::allocate(1).unwrap();
                let payload = pages.bytes_mut();
                for (i, word) in words[5..].iter().enumerate() {
                    payload[i * 8..i * 8 + 8].copy_from_slice(&word.to_le_bytes());
                }
                attachment = Some(pages.freeze().unwrap());
                words[5..].fill(0);
            }
            words[0] |= storage_epoch << wire::STORAGE_SCOPE_BITS;
        }
        let reply = exchange(parent, words, attachment.as_ref());
        if words[0] == wire::STORAGE_INFO {
            storage_epoch = if reply[0] == 0 { reply[1] } else { 0 };
        }
        control.1.send(&wire::encode(reply)).unwrap();
    }
}
fn exchange(
    parent: Pair,
    words: wire::Packet,
    attachment: Option<&cathedral_user_runtime::memory::Object>,
) -> wire::Packet {
    match attachment {
        Some(object) => parent.0.send_memory(&wire::encode(words), object).unwrap(),
        None => parent.0.send(&wire::encode(words)).unwrap(),
    }
    ipc::wait_many(&[parent.1], time::after(300).unwrap()).unwrap();
    let mut bytes = [0; 64];
    let (len, object) = parent.1.try_receive_memory(&mut bytes).unwrap();
    drop(object);
    wire::decode(&bytes[..len]).unwrap()
}
