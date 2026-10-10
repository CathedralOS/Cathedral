use super::{
    Error,
    pixels::{self, Source},
    rows::Row,
    scene::{self, Node, Rect},
};
use std::vec;
#[test]
fn row_children_cannot_touch_neighbors_or_recover_parent() {
    let mut bytes = [0xaa; 20];
    {
        let parent = Row::new(&mut bytes).unwrap().narrow(1, 3).unwrap();
        let (mut left, mut right) = parent.split(1).unwrap();
        left.put(0, 1).unwrap();
        right.put(1, 2).unwrap();
        assert_eq!(left.put(1, 3), Err(Error::Bounds));
        assert_eq!(right.put(usize::MAX, 3), Err(Error::Bounds));
    }
    assert_eq!(&bytes[..4], &[0xaa; 4]);
    assert_eq!(&bytes[8..12], &[0xaa; 4]);
    assert_eq!(&bytes[16..], &[0xaa; 4]);
    assert!(Row::new(&mut bytes).unwrap().split(usize::MAX).is_err());
    assert!(Row::new(&mut bytes).unwrap().narrow(1, usize::MAX).is_err());
    assert!(Row::new(&mut bytes[..3]).is_err());
}
#[test]
fn all_depths_and_odd_sizes_produce_identical_pixels() {
    for (w, h) in [(2, 2), (65, 17), (64, 16)] {
        for depth in 1..=8 {
            let (nodes, count) = scene::fixture(w, h, depth).unwrap();
            let plan = scene::resolve(&nodes[..count], w, h).unwrap();
            let leaves: [std::vec::Vec<u8>; 4] = core::array::from_fn(|i| {
                let rect = nodes[depth + i].rect;
                let mut bytes = vec![0; rect.bytes().unwrap()];
                pixels::leaf(&mut bytes, rect, rect.width * 4).unwrap();
                bytes
            });
            let sources = core::array::from_fn(|i| Source {
                bytes: &leaves[i],
                stride: nodes[depth + i].rect.width * 4,
            });
            let mut flat = vec![0; w * h * 4];
            let mut direct = flat.clone();
            assert_eq!(pixels::compose(&plan, &sources, &mut flat), Ok(flat.len()));
            pixels::direct(&mut direct, w, h, depth, None).unwrap();
            assert_eq!(flat, direct);
            assert!(pixels::verify(&flat, w, h));
        }
    }
}
#[test]
fn nested_translation_and_clipping_resolve_without_ancestor_images() {
    let nodes = [
        Node {
            rect: Rect {
                x: 0,
                y: 0,
                width: 8,
                height: 8,
            },
            ..Node::default()
        },
        Node {
            parent: Some(0),
            rect: Rect {
                x: 2,
                y: 1,
                width: 3,
                height: 4,
            },
            source: None,
        },
        Node {
            parent: Some(1),
            rect: Rect {
                x: 1,
                y: 2,
                width: 4,
                height: 4,
            },
            source: Some(0),
        },
    ];
    let plan = scene::resolve(&nodes, 8, 8).unwrap();
    let draw = plan.draws[0].unwrap();
    assert_eq!(
        draw.target,
        Rect {
            x: 3,
            y: 3,
            width: 2,
            height: 2
        }
    );
    let mut leaf = [0; 64];
    pixels::leaf(
        &mut leaf,
        Rect {
            x: 3,
            y: 3,
            width: 4,
            height: 4,
        },
        16,
    )
    .unwrap();
    let mut sources = [Source {
        bytes: &[],
        stride: 0,
    }; 4];
    sources[0] = Source {
        bytes: &leaf,
        stride: 16,
    };
    let mut target = [0xaa; 256];
    assert_eq!(pixels::compose(&plan, &sources, &mut target), Ok(16));
    for y in 0..8 {
        for x in 0..8 {
            let expected = if (3..5).contains(&x) && (3..5).contains(&y) {
                pixels::color(x, y).to_le_bytes()
            } else {
                [0xaa; 4]
            };
            assert_eq!(target[(y * 8 + x) * 4..(y * 8 + x + 1) * 4], expected);
        }
    }
}
#[test]
fn malformed_sources_are_rejected_before_any_destination_write() {
    let (nodes, count) = scene::fixture(8, 8, 4).unwrap();
    let mut plan = scene::resolve(&nodes[..count], 8, 8).unwrap();
    let source = [0; 64];
    let mut sources = [Source {
        bytes: &source,
        stride: 16,
    }; 4];
    sources[3].bytes = &source[..63];
    let mut target = [0xaa; 256];
    assert_eq!(
        pixels::compose(&plan, &sources, &mut target),
        Err(Error::Bounds)
    );
    assert_eq!(target, [0xaa; 256]);
    sources[3].bytes = &source;
    plan.draws[3].as_mut().unwrap().target.x = usize::MAX;
    assert_eq!(
        pixels::compose(&plan, &sources, &mut target),
        Err(Error::Bounds)
    );
    assert_eq!(target, [0xaa; 256]);
    assert!(
        pixels::leaf(
            &mut target,
            Rect {
                x: usize::MAX,
                y: 0,
                width: 2,
                height: 2
            },
            8
        )
        .is_err()
    );
    assert_eq!(target, [0xaa; 256]);
}
#[test]
fn overlapping_exclusive_leaves_and_invalid_parents_are_rejected() {
    let (mut nodes, count) = scene::fixture(8, 8, 2).unwrap();
    nodes[3].rect = nodes[2].rect;
    assert!(matches!(
        scene::resolve(&nodes[..count], 8, 8),
        Err(Error::Overlap)
    ));
    nodes[2].parent = Some(2);
    assert!(matches!(
        scene::resolve(&nodes[..count], 8, 8),
        Err(Error::Geometry)
    ));
}
#[test]
fn aborted_frame_is_never_published() {
    let mut front = [0xaa; 256];
    let mut back = [0; 256];
    let result = pixels::direct(&mut back, 8, 8, 4, Some(4));
    if result.is_ok() {
        core::mem::swap(&mut front, &mut back);
    }
    assert_eq!(result, Err(Error::Incomplete));
    assert_eq!(front, [0xaa; 256]);
    assert_ne!(back, [0; 256]);
}
