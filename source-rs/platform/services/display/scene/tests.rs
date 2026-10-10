use super::*;
fn snapshot(nodes: &[wire::Node]) -> [u8; wire::SNAPSHOT_BYTES] {
    wire::encode(nodes, &[0x123456; wire::IMAGE_PIXELS]).unwrap()
}
#[test]
fn nested_clips_and_scope_bound_every_pixel() {
    let bytes = snapshot(&[
        wire::Node([wire::ROOT, wire::RECT, 0, 0, 256, 256, 0xff0000]),
        wire::Node([wire::ROOT, wire::GROUP, 10, 12, 20, 18, 0]),
        wire::Node([1, wire::GROUP, 5, 3, 30, 30, 0]),
        wire::Node([2, wire::RECT, 0, 0, 40, 40, 0x00ff00]),
        wire::Node([2, wire::IMAGE, 2, 2, 16, 16, 0]),
    ]);
    let mut client = Client {
        scope: Rect {
            x: 7,
            y: 9,
            w: 40,
            h: 35,
        },
        ..Client::default()
    };
    client.submit(10, 1, &bytes, &mut Scene::empty()).unwrap();
    for y in 0..60 {
        for x in 0..60 {
            let expected = if !(7..47).contains(&x) || !(9..44).contains(&y) {
                None
            } else if (24..37).contains(&x) && (26..39).contains(&y) {
                Some(0x123456)
            } else if (22..37).contains(&x) && (24..39).contains(&y) {
                Some(0x00ff00)
            } else {
                Some(0xff0000)
            };
            assert_eq!(client.pixel(x, y), expected, "({x},{y})");
        }
    }
}
#[test]
fn invalid_or_stale_snapshots_preserve_committed_content_and_disconnect_drops_it() {
    let good = snapshot(&[wire::Node([wire::ROOT, wire::RECT, 0, 0, 10, 10, 123])]);
    let mut client = Client {
        scope: Rect {
            x: 0,
            y: 0,
            w: 10,
            h: 10,
        },
        ..Client::default()
    };
    client.submit(7, 3, &good, &mut Scene::empty()).unwrap();
    assert_eq!(
        client.submit(7, 3, &good, &mut Scene::empty()),
        Err(abi::BAD_HANDLE)
    );
    for bad in [
        snapshot(&[wire::Node([0, wire::GROUP, 0, 0, 1, 1, 0])]),
        snapshot(&[wire::Node([wire::ROOT, wire::RECT, u64::MAX, 0, 2, 1, 0])]),
        snapshot(&[wire::Node([wire::ROOT, wire::TEXT, 0, 0, 1, 7, 65])]),
        snapshot(&[wire::Node([wire::ROOT, wire::IMAGE, 0, 0, 17, 16, 0])]),
    ] {
        assert_eq!(
            client.submit(7, 4, &bad, &mut Scene::empty()),
            Err(abi::INVALID_ARGUMENT)
        );
        assert_eq!(client.revision, 3);
        assert_eq!(client.pixel(0, 0), Some(123));
    }
    client.disconnect(6);
    assert!(client.visible);
    client.disconnect(7);
    assert!(!client.visible);
    client.submit(8, 1, &good, &mut Scene::empty()).unwrap();
    client.disconnect(7);
    assert_eq!(client.revision, 1);
}
#[test]
fn text_and_image_payloads_are_fully_validated_before_commit() {
    let mut bytes = snapshot(&[wire::Node([wire::ROOT, wire::TEXT, 0, 0, 6, 7, 65])]);
    assert!(Scene::empty().decode(&bytes).is_ok());
    bytes[wire::SNAPSHOT_BYTES - 1] = 255;
    assert!(Scene::empty().decode(&bytes).is_err());
    assert!(Scene::empty().decode(&bytes[..10]).is_err());
    bytes = snapshot(&[]);
    bytes[8] = 9;
    assert!(Scene::empty().decode(&bytes).is_err());
}
