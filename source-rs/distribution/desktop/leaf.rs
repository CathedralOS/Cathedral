//! Test peer relays probes over supplied shared interaction/storage. No world/global lookup.
//! A relayed publication carries its bytes inline in the owner packet; the leaf
//! turns them into the sealed attachment the real protocol requires. A detached
//! probe is answered before it is relayed, so the leaf can ask its host for
//! something the host answers only after talking to the owner's other children;
//! the result waits for the next LAST_PROBE.
use super::*;
use cathedral_platform_contracts::storage;
use cathedral_user_runtime::{
    connection::Client,
    link::Link,
    memory::Private,
    server::{Server, SharedSlot},
    services::{Services, Wake},
    task::Launch,
};
pub const EXIT_PROBE: u64 = 0xfffe;
pub const DETACHED_PROBE: u64 = 0xfffd;
pub const LAST_PROBE: u64 = 0xfffc;
pub const LOSS_PROBE: u64 = 0xfffb;
pub const LOSS_HOLD_PROBE: u64 = 0xfffa;
pub const DROP_CONTROL_PROBE: u64 = 0xfff9;
pub const DROP_LOSS_REPLY_PROBE: u64 = 0xfff8;
pub const EXIT_ON_LOSS_PROBE: u64 = 0xfff7;
pub fn run(environment: cathedral_user_runtime::task::Environment) -> u64 {
    assert_eq!(environment.len(), 2);
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
    static mut CONTROL: SharedSlot = SharedSlot::empty();
    let mut control = Server::open_clients(0).unwrap();
    let slot = &raw mut CONTROL;
    // SAFETY: This single-entry process lends its control storage once.
    control
        .enable_shared_control(unsafe { &mut *slot })
        .unwrap();
    control
        .establish_control(time::after(300).unwrap())
        .unwrap();
    let mut parent = Services::load(&environment).unwrap();
    parent.enable_shared().unwrap();
    assert!(matches!(
        parent.interaction(1, 1).unwrap().wake,
        Wake::Notification(_)
    ));
    let mut store =
        Client::connect(parent.storage(1, 1).unwrap(), time::after(300).unwrap()).unwrap();
    cathedral_user_runtime::write(
        b"Cathedral matrix leaf: shared control, interaction and storage ready\n",
    )
    .unwrap();
    let mut storage_epoch = 0;
    let mut last = wire::error(abi::WOULD_BLOCK);
    let (mut loss, mut loss_delay, mut next_loss) = (0, 0, 0);
    let mut loss_reserved = 0;
    let mut hold_loss = false;
    let mut drop_loss_reply = false;
    let mut exit_on_loss = false;
    loop {
        let Ok(request) = control.next_request() else {
            return 0;
        };
        let Some(mut words) = wire::decode(&request.bytes[..request.len]) else {
            return 1;
        };
        if words == [EXIT_PROBE, 0, 0, 0, 0, 0, 0, 0] {
            return 0;
        }
        if words == [DROP_CONTROL_PROBE, 0, 0, 0, 0, 0, 0, 0] {
            // Remain alive with the interaction connection while refusing further
            // control requests. A sibling cannot interpret this as clean consent.
            control.reply(&request, &wire::encode([0; 8])).unwrap();
            drop(control);
            loop {
                cathedral_user_runtime::yield_now();
            }
        }
        if words == [EXIT_ON_LOSS_PROBE, 0, 0, 0, 0, 0, 0, 0] {
            exit_on_loss = true;
            control.reply(&request, &wire::encode([0; 8])).unwrap();
            continue;
        }
        if words == [DROP_LOSS_REPLY_PROBE, 0, 0, 0, 0, 0, 0, 0] {
            drop_loss_reply = true;
            control.reply(&request, &wire::encode([0; 8])).unwrap();
            continue;
        }
        if words == [LAST_PROBE, 0, 0, 0, 0, 0, 0, 0] {
            control.reply(&request, &wire::encode(last)).unwrap();
            continue;
        }
        if let [LOSS_PROBE, value, delay, next, reserved, 0, 0, 0] = words {
            (loss, loss_delay, next_loss) = (value, delay, next);
            loss_reserved = reserved;
            control.reply(&request, &wire::encode([0; 8])).unwrap();
            continue;
        }
        if words == [LOSS_HOLD_PROBE, 0, 0, 0, 0, 0, 0, 0] {
            hold_loss = true;
            control.reply(&request, &wire::encode([0; 8])).unwrap();
            continue;
        }
        // Normally nothing to lose. A fixture can delay one old clean answer
        // while subsequent answers become dirty, exercising retained correlation.
        // Everything else, a liveness probe included, is relayed to the parent
        // so a fixture sees exactly what the matrix answers a child.
        if words == [wire::LOSS, 0, 0, 0, 0, 0, 0, 0] {
            if exit_on_loss {
                return 0;
            }
            if drop_loss_reply {
                drop(control);
                loop {
                    cathedral_user_runtime::yield_now();
                }
            }
            if core::mem::take(&mut hold_loss) {
                // A shared control client retains one timed-out request. Repeated
                // owner surveys must not queue new work or steal its late reply.
                let until = time::after(1000).unwrap();
                match control.next_event_for(&[], Some(until)) {
                    Ok(None) => (),
                    Err(Error(code)) if code == abi::TIMED_OUT as i64 => (),
                    _ => panic!("control replayed while LOSS pending"),
                }
                control.reply(&request, &wire::encode([0; 8])).unwrap();
                (loss, loss_delay, next_loss) = (1, 0, 1);
                cathedral_user_runtime::write(
                    b"Cathedral matrix leaf: held one LOSS without queued replay\n",
                )
                .unwrap();
                continue;
            }
            let answer = [0, loss, 0, 0, 0, 0, 0, loss_reserved];
            let until = time::after(loss_delay).unwrap();
            while !time::reached(time::now().unwrap(), until) {
                cathedral_user_runtime::yield_now();
            }
            loss_delay = 0;
            loss = next_loss;
            control.reply(&request, &wire::encode(answer)).unwrap();
            continue;
        }
        if words[0] == DETACHED_PROBE {
            control.reply(&request, &wire::encode([0; 8])).unwrap();
            let mut detached = [0; 8];
            detached[..7].copy_from_slice(&words[1..]);
            last = exchange(&mut parent, detached);
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
        let reply = if stored || words[0] == wire::STORAGE_INFO {
            let (bytes, n, object) = store
                .call_memory(
                    &wire::encode(words),
                    attachment.as_ref(),
                    time::after(300).unwrap(),
                )
                .unwrap();
            drop(object);
            wire::decode(&bytes[..n]).unwrap()
        } else {
            exchange(&mut parent, words)
        };
        if words[0] == wire::STORAGE_INFO {
            storage_epoch = if reply[0] == 0 { reply[1] } else { 0 };
        }
        control.reply(&request, &wire::encode(reply)).unwrap();
    }
}
fn exchange(parent: &mut Services, words: wire::Packet) -> wire::Packet {
    parent.request_interaction(words, None).unwrap()
}
