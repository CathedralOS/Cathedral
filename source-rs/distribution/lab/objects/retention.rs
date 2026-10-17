//! Three cold boots: preserve a pin while replacing, release and reuse space,
//! then verify the collected state. Bulk custody is separate from revision pins.
use super::client::Client;
use cathedral_platform_contracts::database as wire;
use cathedral_user_runtime::{
    link::Link,
    memory::{Object, Private},
    write,
};
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
fn publish(client: &Client, root: u64, revision: u64) {
    let mut pages = Private::allocate(2).unwrap();
    for (i, b) in pages.bytes_mut().iter_mut().enumerate() {
        *b = byte(root, revision, i);
    }
    let object = pages.freeze().unwrap();
    let reply = client.call(wire::PUBLISH, revision - 1, 0, 8192, Some(&object));
    assert_eq!(wire::word(&reply, 0), wire::OK);
    assert_eq!(wire::word(&reply, 8), revision);
}
fn collect(client: &Client) {
    for _ in 0..32 {
        let reply = client.call(wire::COLLECT, 0, 0, 0, None);
        assert_eq!(wire::word(&reply, 0), wire::OK);
        assert!(wire::word(&reply, 8) <= 8 && wire::word(&reply, 16) <= 24576);
        if wire::word(&reply, 24) == 0 {
            return;
        }
    }
    panic!("collection did not converge");
}
fn bulk(
    client: &Client,
    op: u64,
    revision: u64,
    offset: u64,
    len: u64,
) -> ([u8; 64], Option<Object>) {
    client
        .send
        .send(
            &wire::Request {
                op,
                expected: revision,
                offset,
                len,
            }
            .encode(),
        )
        .unwrap();
    let mut reply = [0; 64];
    let (n, object) = client.receive.receive_memory(&mut reply).unwrap();
    assert_eq!(n, 64);
    (reply, object)
}
fn verify(client: &Client, root: u64, revision: u64, pinned: bool) {
    let (reply, object) = bulk(
        client,
        if pinned { wire::BULK_PIN } else { wire::BULK },
        revision,
        0,
        8192,
    );
    assert_eq!(wire::word(&reply, 0), wire::OK);
    assert_eq!(wire::word(&reply, 8), revision);
    let mut object = object.unwrap();
    assert_eq!(object.len(), 8192);
    for (i, b) in object.map().unwrap().iter().enumerate() {
        assert_eq!(*b, byte(root, revision, i));
    }
}
fn pressure(client: &Client, root: u64, revision: u64) {
    let (a, first) = bulk(client, wire::BULK, revision, 0, 8192);
    let (b, second) = bulk(client, wire::BULK, revision, 0, 8192);
    assert_eq!(wire::word(&a, 0), wire::OK);
    assert_eq!(wire::word(&b, 0), wire::OK);
    let (reply, refused) = bulk(client, wire::BULK, revision, 0, 8192);
    assert!(refused.is_none());
    assert_eq!(wire::word(&reply, 0), wire::BUSY);
    drop(first);
    drop(second);
    verify(client, root, revision, false);
    let (reply, object) = bulk(client, wire::BULK, revision, 4091, 17);
    assert_eq!(wire::word(&reply, 0), wire::OK);
    let mut object = object.unwrap();
    let bytes = object.map().unwrap();
    for (i, b) in bytes[..17].iter().enumerate() {
        assert_eq!(*b, byte(root, revision, 4091 + i));
    }
    assert!(bytes[17..].iter().all(|b| *b == 0));
}
pub fn run(root: u64) -> u64 {
    let (send, receive) = Link::at(0).unwrap().connect().unwrap();
    let client = Client { send, receive };
    let current = wire::word(&client.call(wire::HEAD, 0, 0, 0, None), 8);
    let outcome = match current {
        0 => {
            publish(&client, root, 1);
            assert_eq!(
                wire::word(&client.call(wire::PIN, 1, 0, 0, None), 0),
                wire::OK
            );
            for revision in 2..=9 {
                publish(&client, root, revision);
                collect(&client);
                verify(&client, root, 1, true);
            }
            verify(&client, root, 9, false);
            pressure(&client, root, 9);
            9
        }
        9 => {
            verify(&client, root, 9, false);
            verify(&client, root, 1, true);
            assert_eq!(
                wire::word(&client.call(wire::RELEASE, 2, 0, 0, None), 0),
                wire::CONFLICT
            );
            assert_eq!(
                wire::word(&client.call(wire::RELEASE, 1, 0, 0, None), 0),
                wire::OK
            );
            collect(&client);
            for revision in 10..=25 {
                publish(&client, root, revision);
                collect(&client);
            }
            verify(&client, root, 25, false);
            25
        }
        25 => {
            verify(&client, root, 25, false);
            assert_eq!(
                wire::word(&client.call(wire::PIN, 1, 0, 0, None), 0),
                wire::CONFLICT
            );
            let (reply, object) = bulk(&client, wire::BULK_PIN, 1, 0, 8192);
            assert_eq!(wire::word(&reply, 0), wire::CONFLICT);
            assert!(object.is_none());
            pressure(&client, root, 25);
            26
        }
        _ => panic!("unexpected retained fixture revision"),
    };
    write(b"Cathedral objects: retention and bulk checks passed\n").unwrap();
    outcome
}
