//! Three cold boots: preserve a pin while replacing, release and reuse space,
//! then verify the collected state. Bulk custody is separate from revision pins.
use super::client::{Client, packet};
use cathedral_kernel_contracts::user as abi;
use cathedral_platform_contracts::storage as wire;
use cathedral_user_runtime::{memory::Private, write};
fn byte(root: u64, revision: u64, index: usize) -> u8 {
    let base = ((index * 17
        + index / 4096 * 71
        + root as usize * 43
        + if revision >= 10 {
            revision as usize * 13
        } else {
            0
        })
        % 251) as u8;
    if revision > 1 && revision < 10 && index == 41 {
        base ^ revision as u8
    } else {
        base
    }
}
fn publish(client: &mut Client, root: u64, revision: u64) {
    let mut pages = Private::allocate(2).unwrap();
    for (i, b) in pages.bytes_mut().iter_mut().enumerate() {
        *b = byte(root, revision, i);
    }
    let object = pages.freeze().unwrap();
    let reply = client.call(
        packet(
            wire::PUBLISH,
            u64::from(revision > 1),
            revision - 1,
            0,
            8192,
        ),
        Some(&object),
    );
    let head = wire::Reply::decode(&reply).unwrap();
    assert_eq!((head.object, head.revision), (1, revision));
}
fn collect(client: &mut Client) {
    for _ in 0..32 {
        let reply = client.call(packet(wire::COLLECT, 0, 0, 0, 0), None);
        assert_eq!(wire::word(&reply, 0), 0);
        assert!(wire::word(&reply, 1) <= 8 && wire::word(&reply, 2) <= 24576);
        if wire::word(&reply, 3) == 0 {
            return;
        }
    }
    panic!("collection did not converge");
}
fn verify(client: &mut Client, root: u64, revision: u64, pinned: bool) {
    let (reply, object) = client.exchange(
        if pinned {
            packet(wire::BULK_PIN, 1, revision, 0, 8192)
        } else {
            packet(wire::BULK, 1, revision, 0, 8192)
        },
        None,
    );
    let head = wire::Reply::decode(&reply).unwrap();
    assert_eq!((head.revision, head.extra), (revision, 8192));
    let mut object = object.unwrap();
    assert_eq!(object.len(), 8192);
    for (i, b) in object.map().unwrap().iter().enumerate() {
        assert_eq!(*b, byte(root, revision, i));
    }
}
fn pressure(client: &mut Client, root: u64, revision: u64) {
    let (a, first) = client.exchange(packet(wire::BULK, 1, revision, 0, 8192), None);
    let (b, second) = client.exchange(packet(wire::BULK, 1, revision, 0, 8192), None);
    assert_eq!(wire::word(&a, 0), 0);
    assert_eq!(wire::word(&b, 0), 0);
    let (reply, refused) = client.exchange(packet(wire::BULK, 1, revision, 0, 8192), None);
    assert!(refused.is_none());
    // Two replies pin all four data-reader pages beside the shared mailbox.
    assert_eq!(
        cathedral_user_runtime::resources::budget()
            .unwrap()
            .used
            .reader_pages,
        5
    );
    // The current kernel reports reader
    // capacity as NO_MEMORY; dropping them must make the next reply possible.
    assert_eq!(wire::word(&reply, 0), abi::NO_MEMORY);
    drop(first);
    drop(second);
    assert_eq!(
        cathedral_user_runtime::resources::budget()
            .unwrap()
            .used
            .reader_pages,
        1
    );
    verify(client, root, revision, false);
    let (reply, object) = client.exchange(packet(wire::BULK, 1, revision, 4091, 17), None);
    assert_eq!(wire::word(&reply, 0), 0);
    let mut object = object.unwrap();
    let bytes = object.map().unwrap();
    for (i, b) in bytes[..17].iter().enumerate() {
        assert_eq!(*b, byte(root, revision, 4091 + i));
    }
    assert!(bytes[17..].iter().all(|b| *b == 0));
}
pub fn run(root: u64) -> u64 {
    let mut client = Client::open();
    let current = match wire::Reply::decode(&client.call(packet(wire::HEAD, 1, 0, 0, 0), None)) {
        Ok(head) => head.revision,
        Err(wire::MISSING) => 0,
        Err(error) => panic!("retention head {error}"),
    };
    let outcome = match current {
        0 => {
            publish(&mut client, root, 1);
            assert_eq!(
                wire::word(&client.call(packet(wire::PIN, 1, 1, 0, 0), None), 0),
                0
            );
            for revision in 2..=9 {
                publish(&mut client, root, revision);
                collect(&mut client);
                verify(&mut client, root, 1, true);
            }
            verify(&mut client, root, 9, false);
            pressure(&mut client, root, 9);
            9
        }
        9 => {
            verify(&mut client, root, 9, false);
            verify(&mut client, root, 1, true);
            assert_eq!(
                wire::word(&client.call(packet(wire::RELEASE, 1, 2, 0, 0), None), 0),
                abi::WOULD_BLOCK
            );
            assert_eq!(
                wire::word(&client.call(packet(wire::RELEASE, 1, 1, 0, 0), None), 0),
                0
            );
            collect(&mut client);
            for revision in 10..=25 {
                publish(&mut client, root, revision);
                collect(&mut client);
            }
            verify(&mut client, root, 25, false);
            25
        }
        25 => {
            verify(&mut client, root, 25, false);
            assert_eq!(
                wire::word(&client.call(packet(wire::PIN, 1, 1, 0, 0), None), 0),
                abi::WOULD_BLOCK
            );
            let (reply, object) = client.exchange(packet(wire::BULK_PIN, 1, 1, 0, 8192), None);
            assert_eq!(wire::word(&reply, 0), abi::WOULD_BLOCK);
            assert!(object.is_none());
            pressure(&mut client, root, 25);
            26
        }
        _ => panic!("unexpected retained fixture revision"),
    };
    write(b"Cathedral objects: retention and bulk checks passed\n").unwrap();
    outcome
}
