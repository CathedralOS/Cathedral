use super::fixture::*;
use cathedral_platform_contracts::matrix as wire;
use std::vec;

#[test]
fn boots_from_scoped_storage_and_publishes_platform_compatible_scene() {
    with_app(|p| {
        p.pump();
        assert_eq!(p.acquired, [1, 2]);
        assert_eq!(p.record.as_deref(), Some(&b"MC01\0\0"[..]));
        assert_eq!(p.color(), BLUE);
        assert_eq!(p.loss(), 0);
        let checker = &p.images[0].1;
        assert_eq!(&checker[..4], &0xffcc44u32.to_le_bytes());
        assert_eq!(&checker[32..36], &0x7755ddu32.to_le_bytes());
        assert_eq!(p.images[1].0[5], 20);
        p.enter();
        assert_eq!(p.color(), GOLD);
        assert_eq!(p.record.as_deref(), Some(&b"MC01\0\x01"[..]));
        assert_eq!(p.revision, 2);
        p.click(42);
        assert!(p.exited);
    });
}
#[test]
fn cold_open_reads_existing_color_without_creation_or_replacement() {
    with_app(|p| {
        p.record = Some(b"MC01\0\x01".to_vec());
        p.revision = 37;
        p.pump();
        assert_eq!(p.color(), GOLD);
        assert!(!p.commands.iter().any(|q| q[0] == 3));
        p.enter();
        assert_eq!(p.color(), BLUE);
        assert_eq!(p.revision, 38);
    });
}
#[test]
fn uncertain_save_keeps_confirmed_pixels_and_reconciles_without_replaying() {
    with_app(|p| {
        p.pump();
        p.save_error = true;
        p.commit_before_error = true;
        p.enter();
        assert_eq!(p.color(), BLUE);
        assert_ne!(p.loss(), 0);
        assert_eq!(p.revision, 2);
        p.save_error = false;
        p.enter();
        assert_eq!(p.color(), GOLD);
        assert_eq!(p.loss(), 0);
        assert_eq!(p.revision, 2, "reconcile must not replay the toggle");
    });
}
#[test]
fn ambiguous_creation_is_settled_by_listing_not_duplicate_creation() {
    with_app(|p| {
        p.create_error = true;
        p.pump();
        assert_eq!(p.color(), BLUE);
        assert_eq!(p.loss(), 0);
        assert_eq!(p.revision, 1);
        assert_eq!(p.commands.iter().filter(|q| q[0] == 3).count(), 1);
    });
}
#[test]
fn malformed_records_are_not_silently_overwritten() {
    with_app(|p| {
        p.record = Some(b"BAD!\0\0".to_vec());
        p.revision = 1;
        p.pump();
        assert_ne!(p.loss(), 0);
        p.enter();
        assert_eq!(p.record.as_deref(), Some(&b"BAD!\0\0"[..]));
        assert_eq!(p.revision, 1);
    });
}
#[test]
fn uncertain_close_requires_a_second_edge_not_held_motion() {
    with_app(|p| {
        p.pump();
        p.save_error = true;
        p.enter();
        p.click(19);
        assert!(!p.exited);
        assert_eq!(p.images.last().unwrap().0[5], 156);
        p.click(19);
        assert!(!p.exited);
        p.event(wire::CANCEL_KEYS, 0, 0, 0, 19);
        p.click(19);
        assert!(!p.exited);
        p.click(73);
        assert!(p.exited, "independent pointer has its own edge");
    });
}
#[test]
fn pointer_cancellation_and_rebinding_end_old_holds() {
    with_app(|p| {
        p.pump();
        p.save_error = true;
        p.enter();
        p.click(19);
        p.event(wire::CANCEL, 0, 0, 0, 19);
        p.click(19);
        assert!(p.exited);
    });
    with_app(|p| {
        p.pump();
        p.save_error = true;
        p.enter();
        p.click(19);
        p.epoch += 1;
        p.wake();
        p.click(19);
        assert!(p.exited);
    });
}
#[test]
fn no_frame_demand_does_not_stop_input_or_storage() {
    with_app(|p| {
        p.pump();
        let images = p.images.len();
        let paints = p.paints.len();
        p.frames = false;
        p.enter();
        assert_eq!(p.images.len(), images);
        assert_eq!(p.paints.len(), paints);
        assert_eq!(p.record.as_deref(), Some(&b"MC01\0\x01"[..]));
        p.frames = true;
        p.wake();
        assert_eq!(p.color(), GOLD);
    });
}
#[test]
fn close_image_matches_the_original_font_and_bounds() {
    with_app(|p| {
        p.pump();
        p.save_error = true;
        p.enter();
        p.click(19);
        for (image, text) in [
            (&p.images[1], &b"X"[..]),
            (p.images.last().unwrap(), &b"CLOSE ANYWAY"[..]),
        ] {
            let width = image.0[5] as usize;
            let mut expected = vec![0x993f46u32; width * 16];
            for (i, &ch) in text.iter().enumerate() {
                for (y, bits) in cathedral_bitmap_font::glyph(ch).into_iter().enumerate() {
                    for x in 0..5 {
                        if bits & (1 << (4 - x)) != 0 {
                            for dy in 0..2 {
                                for dx in 0..2 {
                                    expected[(1 + y * 2 + dy) * width + 5 + i * 12 + x * 2 + dx] =
                                        0xffffff;
                                }
                            }
                        }
                    }
                }
            }
            assert!(
                image
                    .1
                    .chunks_exact(4)
                    .zip(expected)
                    .all(|(a, b)| a == b.to_le_bytes())
            );
        }
    });
}
#[test]
fn refused_required_service_stops_before_app_work() {
    with_app(|p| {
        assert_eq!(p.app.advance(), 1);
        p.app.packet[9] = (-13i64) as u64;
        assert_eq!(p.app.advance(), 0);
    });
}

#[test]
fn continuous_input_yields_for_owner_questions_between_bounded_batches() {
    with_app(|p| {
        p.pump();
        for _ in 0..24 {
            p.events
                .push_back([0, p.epoch, wire::POINTER, 10, 40, 0, 19, 1]);
        }
        p.wake();
        assert_eq!(p.events.len(), 16);
        assert_eq!(p.loss(), 0);
        p.wake();
        assert_eq!(p.events.len(), 8);
        p.wake();
        assert!(p.events.is_empty());
    });
}

#[test]
fn rejected_surface_rechecks_the_grant_before_rebuilding_or_stopping() {
    for changed in [false, true] {
        with_app(|p| {
            p.reject_surface = true;
            p.rebind_on_rejection = changed;
            p.pump();
            assert_eq!(p.exited, !changed);
            assert_eq!(p.commands.iter().filter(|q| q[0] == wire::VIEW).count(), 1);
            if changed {
                assert_eq!(p.epoch, 8);
                assert_eq!(p.color(), BLUE);
                assert_eq!(p.loss(), 0);
            } else {
                assert!(p.images.is_empty());
                assert!(p.paints.is_empty());
            }
        });
    }
}

#[test]
fn record_codec_rejects_wrong_lengths_reserved_bytes_and_unknown_colors() {
    for record in [
        &b""[..],
        &b"MC01\0"[..],
        &b"MC01\0\0x"[..],
        &b"MC01\x01\0"[..],
        &b"MC01\0\x02"[..],
    ] {
        with_app(|p| {
            p.record = Some(record.to_vec());
            p.revision = 12;
            p.pump();
            assert_ne!(p.loss(), 0);
            p.enter();
            assert_eq!(p.record.as_deref(), Some(record));
            assert_eq!(p.revision, 12);
            assert!(!p.commands.iter().any(|q| q[0] == 3));
        });
    }
}

#[test]
fn retired_channel_frees_one_pointer_slot_without_resetting_other_channels() {
    with_app(|p| {
        p.pump();
        for channel in [19, 20, 21, 22] {
            p.event(wire::POINTER, 10, 40, 0, channel);
        }
        p.click(23);
        assert!(!p.exited, "a fifth channel cannot take a live pointer slot");
        p.event(wire::RETIRED, 0, 0, 0, 19);
        p.click(23);
        assert!(p.exited, "retirement returns just that channel's slot");
    });
}
