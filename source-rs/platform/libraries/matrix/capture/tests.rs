use super::*;
use std::{cell::Cell, rc::Rc, vec, vec::Vec};
fn rect(x: u64, y: u64, width: u64, height: u64) -> Rect {
    Rect {
        x,
        y,
        width,
        height,
    }
}
fn plan() -> Plan {
    Plan::new(40, 30, rect(2, 3, 8, 6), &[rect(4, 4, 2, 3)]).unwrap()
}
fn source() -> Source {
    Source {
        participant: Participant {
            slot: 2,
            incarnation: 11,
        },
        epoch: 3,
    }
}
fn receiver() -> Participant {
    Participant {
        slot: 4,
        incarnation: 29,
    }
}
fn lease<T>() -> Lease<T> {
    let mut l = Lease::empty();
    l.grant(source(), receiver(), plan()).unwrap();
    l
}
#[test]
fn capture_requires_exact_granted_endpoint_and_source_view() {
    let mut l = lease();
    let impostor = Participant {
        slot: receiver().slot,
        incarnation: 30,
    };
    assert_eq!(l.begin(impostor, 0, 20), Err(Error::Denied));
    let ticket = l.begin(receiver(), 0, 20).unwrap();
    assert_eq!(l.demand(), Some((source(), ticket)));
    assert_eq!(l.status(impostor, ticket, 50), Err(Error::Denied));
    // Unauthorized observations do not expire another recipient's request.
    assert_eq!(l.demand(), Some((source(), ticket)));
    assert_eq!(
        l.publish(
            Source {
                epoch: 4,
                ..source()
            },
            ticket,
            1,
            vec![1]
        ),
        Err(Error::Denied)
    );
    assert_eq!(
        l.publish(
            Source {
                participant: Participant {
                    slot: 3,
                    ..source().participant
                },
                ..source()
            },
            ticket,
            1,
            vec![1]
        ),
        Err(Error::Denied)
    );
    assert_eq!(l.status(receiver(), ticket, 1), Ok(Status::Waiting));
    l.publish(source(), ticket, 2, vec![2]).unwrap();
    assert_eq!(l.demand(), None);
    assert_eq!(l.ready(impostor, ticket), Err(Error::Denied));
    assert_eq!(l.ready(receiver(), ticket).unwrap(), &[2]);
}
#[test]
fn one_funded_result_survives_failed_transfer_without_redrawing() {
    let mut l = lease();
    let ticket = l.begin(receiver(), 10, 20).unwrap();
    assert_eq!(l.begin(receiver(), 11, 20), Err(Error::Busy));
    assert_eq!(l.ready(receiver(), ticket), Err(Error::Pending));
    l.publish(source(), ticket, 12, vec![42]).unwrap();
    // A failed transport borrows the same backing again, with no renewed demand.
    let ptr = l.ready(receiver(), ticket).unwrap().as_ptr();
    assert_eq!(l.ready(receiver(), ticket).unwrap().as_ptr(), ptr);
    assert_eq!(l.status(receiver(), ticket, 500), Ok(Status::Ready));
    assert_eq!(l.begin(receiver(), 500, 20), Err(Error::Busy));
    assert_eq!(l.release(receiver(), ticket + 1), Err(Error::Denied));
    l.release(receiver(), ticket).unwrap();
    let next = l.begin(receiver(), 500, 20).unwrap();
    assert!(next > ticket);
    assert_eq!(
        l.publish(source(), ticket, 501, vec![99]),
        Err(Error::Denied)
    );
    assert_eq!(l.release(receiver(), ticket), Err(Error::Denied));
    assert_eq!(l.demand(), Some((source(), next)));
}
#[test]
fn expiration_wrap_regrant_and_ticket_exhaustion_fail_closed() {
    let mut l = lease::<u32>();
    let ticket = l.begin(receiver(), u64::MAX - 3, 8).unwrap();
    assert_eq!(l.status(receiver(), ticket, 2), Ok(Status::Waiting));
    assert_eq!(l.status(receiver(), ticket, 4), Ok(Status::Expired));
    assert_eq!(l.demand(), None);
    assert_eq!(l.publish(source(), ticket, 4, 1), Err(Error::Expired));
    l.revoke();
    l.grant(source(), receiver(), plan()).unwrap();
    let next = l.begin(receiver(), 5, 8).unwrap();
    assert!(next > ticket);
    assert_eq!(l.publish(source(), ticket, 6, 1), Err(Error::Denied));
    l.release(receiver(), next).unwrap();
    l.last = u64::MAX;
    assert_eq!(l.begin(receiver(), 10, 8), Err(Error::Exhausted));
    assert_eq!(l.demand(), None);
    assert_eq!(l.begin(receiver(), 10, 0), Err(Error::Invalid));
}
struct Tracked(Rc<Cell<usize>>);
impl Drop for Tracked {
    fn drop(&mut self) {
        self.0.set(self.0.get() + 1);
    }
}
#[test]
fn replacement_and_death_release_retained_results_without_reviving_grants() {
    let dropped = Rc::new(Cell::new(0));
    let mut l = lease();
    let ticket = l.begin(receiver(), 0, 10).unwrap();
    l.publish(source(), ticket, 1, Tracked(dropped.clone()))
        .unwrap();
    l.reconcile(
        Some(Source {
            epoch: 4,
            ..source()
        }),
        Some(receiver()),
    );
    assert_eq!(dropped.get(), 1);
    assert_eq!(l.begin(receiver(), 2, 10), Err(Error::Denied));
    l.grant(source(), receiver(), plan()).unwrap();
    let ticket = l.begin(receiver(), 2, 10).unwrap();
    l.publish(source(), ticket, 3, Tracked(dropped.clone()))
        .unwrap();
    l.reconcile(Some(source()), None);
    assert_eq!(dropped.get(), 2);
    assert_eq!(l.demand(), None);
    l.grant(source(), receiver(), plan()).unwrap();
    let ticket = l.begin(receiver(), 4, 10).unwrap();
    l.reconcile(None, Some(receiver()));
    assert!(matches!(
        l.publish(source(), ticket, 5, Tracked(dropped.clone())),
        Err(Error::Denied)
    ));
    assert_eq!(dropped.get(), 3);
}
#[test]
fn cropped_stride_mask_and_padding_never_leak_source_or_stale_bytes() {
    let p = Plan::new(8, 8, rect(2, 2, 4, 3), &[rect(3, 3, 2, 1)]).unwrap();
    let mut source = vec![0u8; 8 * 8 * 4];
    for y in 0..8 {
        for x in 0..8 {
            let at = (y * 8 + x) * 4;
            source[at..at + 4].copy_from_slice(&(0xff000000u32 | (y * 8 + x) as u32).to_le_bytes());
        }
    }
    let layers = [
        Layer {
            rect: rect(0, 0, 8, 8),
            paint: Paint::Image {
                bytes: &source,
                stride: 8,
            },
        },
        Layer {
            rect: rect(3, 3, 1, 1),
            paint: Paint::Solid(0xffffff),
        },
    ];
    let mut out = vec![0xcc; 4096];
    p.render(&layers, &mut out).unwrap();
    for y in 0..3 {
        for x in 0..4 {
            let expected = if y == 1 && (1..=2).contains(&x) {
                0
            } else {
                ((y + 2) * 8 + x + 2) as u32
            };
            let at = (y * 4 + x) * 4;
            assert_eq!(
                u32::from_le_bytes(out[at..at + 4].try_into().unwrap()),
                expected
            );
        }
    }
    assert!(out[p.bytes()..].iter().all(|b| *b == 0));
}
#[test]
fn masks_cannot_be_overpainted_and_cursors_preserve_transparency() {
    let p = Plan::new(12, 16, rect(0, 0, 12, 16), &[rect(3, 3, 1, 1)]).unwrap();
    let layers = [
        Layer {
            rect: rect(0, 0, 12, 16),
            paint: Paint::Solid(0x112233),
        },
        Layer {
            rect: rect(2, 2, 7, 12),
            paint: Paint::Cursor(0xffd166),
        },
    ];
    let mut out = vec![0; 12 * 16 * 4];
    p.render(&layers, &mut out).unwrap();
    let pixel = |x: usize, y: usize| {
        u32::from_le_bytes(
            out[(y * 12 + x) * 4..(y * 12 + x) * 4 + 4]
                .try_into()
                .unwrap(),
        )
    };
    assert_eq!(pixel(2, 2), 0xffd166);
    assert_eq!(pixel(8, 2), 0x112233);
    assert_eq!(pixel(3, 3), 0);
}
#[test]
fn invalid_geometry_backing_and_capacity_do_not_publish_partial_output() {
    for crop in [
        rect(u64::MAX, 0, 2, 2),
        rect(0, 0, 0, 1),
        rect(0, 0, 4096, 4096),
        rect(30, 0, 20, 2),
    ] {
        assert_eq!(Plan::new(40, 30, crop, &[]), Err(Error::Invalid));
    }
    assert_eq!(
        Plan::new(40, 30, rect(0, 0, 2, 2), &[rect(0, 0, 1, 1); 9]),
        Err(Error::Invalid)
    );
    assert_eq!(
        Plan::new(40, 30, rect(0, 0, 2, 2), &[rect(u64::MAX, 0, 2, 1)]),
        Err(Error::Invalid)
    );
    let p = plan();
    let mut out = vec![0xcc; 4096];
    let short = [0; 4];
    let bad = Layer {
        rect: rect(0, 0, 10, 10),
        paint: Paint::Image {
            bytes: &short,
            stride: 10,
        },
    };
    assert_eq!(p.render(&[bad], &mut out), Err(Error::Invalid));
    assert!(out.iter().all(|b| *b == 0xcc));
    let solid = Layer {
        rect: rect(0, 0, 10, 10),
        paint: Paint::Solid(1),
    };
    assert_eq!(p.render(&[solid; 65], &mut out), Err(Error::Invalid));
    assert_eq!(p.render(&[], &mut [0; 4]), Err(Error::Invalid));
    assert_eq!(p.render(&[], &mut vec![0; 131073]), Err(Error::Invalid));
    let mut exact: Vec<u8> = vec![0xcc; p.bytes()];
    p.render(&[], &mut exact).unwrap();
    assert!(exact.iter().all(|b| *b == 0));
}

#[test]
fn hidden_frame_handshake_masked_result_and_release_compose() {
    use crate::world::{Grant, Rect as Placement, World};
    use cathedral_platform_contracts::matrix as wire;
    let mut world = World::<1>::empty();
    world.inherit(Grant {
        epoch: 1,
        width: 40,
        height: 30,
        keys: 0,
    });
    world
        .place(
            0,
            Placement {
                x: 0,
                y: 0,
                width: 40,
                height: 30,
            },
            0,
        )
        .unwrap();
    let epoch = world.request(0, 11, [wire::VIEW, 0, 0, 0, 0, 0, 0, 0])[1];
    let src = Source {
        participant: Participant {
            slot: 0,
            incarnation: 11,
        },
        epoch,
    };
    let mut l = Lease::empty();
    l.grant(src, receiver(), plan()).unwrap();
    world.visible(0, false).unwrap();
    let request = l.begin(receiver(), 0, 20).unwrap();
    let frame = world.request_frame(0, 11, epoch).unwrap();
    assert!(wire::poll_frames(world.request(
        0,
        11,
        [wire::POLL, 0, 0, 0, 0, 0, 0, 0]
    )));
    assert_eq!(
        world.request(0, 11, [wire::FRAME_DEMAND, epoch, 0, 0, 0, 0, 0, 0])[2],
        frame
    );
    // The producer finishes its accepted scene, then acknowledges that ticket.
    // Real providers must pin sealed backing before dispatching another update.
    assert_eq!(
        world.request(0, 11, [wire::FRAME_READY, epoch, frame, 0, 0, 0, 0, 0]),
        [0; 8]
    );
    assert!(world.frame_ready(0, 11, epoch, frame));
    let mut pixels = vec![0xcc; 4096];
    l.plan()
        .unwrap()
        .render(
            &[Layer {
                rect: rect(0, 0, 40, 30),
                paint: Paint::Solid(0xffffff),
            }],
            &mut pixels,
        )
        .unwrap();
    l.publish(src, request, 1, pixels).unwrap();
    world.release_frame(0, 11, epoch, frame).unwrap();
    assert!(!wire::poll_frames(world.request(
        0,
        11,
        [wire::POLL, 0, 0, 0, 0, 0, 0, 0]
    )));
    let result = l.ready(receiver(), request).unwrap();
    // Mask (4,4) is result pixel (2,1), independent of hidden source placement.
    assert_eq!(&result[(8 + 2) * 4..(8 + 3) * 4], &[0; 4]);
    assert_eq!(&result[..4], &[255, 255, 255, 0]);
    assert!(result[plan().bytes()..].iter().all(|b| *b == 0));
    world.disconnected(0);
    l.reconcile(None, Some(receiver()));
    assert_eq!(l.ready(receiver(), request), Err(Error::Denied));
}
