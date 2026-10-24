use super::fixture::*;
use cathedral_platform_contracts::matrix as wire;

#[test]
fn startup_waits_for_placement_and_keeps_the_first_input() {
    with_app(|p| {
        p.epoch = 0;
        p.pump();
        assert_eq!(p.storage_requests, 0);
        assert!(p.images.is_empty());
        p.epoch = 7;
        p.text("a");
        assert_eq!(p.status()[2..5], [1, 0, 1]);
        p.key(6);
        assert_eq!(p.records[0].1, b"NTS1a");
    });
}

#[test]
fn edits_persists_duplicates_and_reloads_the_real_schema() {
    with_app(|p| {
        p.pump();
        assert!(p.records.is_empty());
        assert_eq!(p.loss(), 0);
        p.text("abc");
        p.key(3);
        p.key(52);
        p.text("z");
        assert_eq!(p.loss(), 1);
        p.key(6);
        assert_eq!(p.records[0].1, b"NTS1azc");
        assert_eq!(p.loss(), 0);
        p.key(9);
        assert_eq!(p.records.len(), 2);
        assert_eq!(p.status()[6], 2);
        p.text("x");
        p.key(7);
        assert_eq!(p.loss(), 0);
        assert_eq!(p.status()[2], 3);
        p.key(10);
        assert_eq!(p.status()[6], 1);
    });
}
#[test]
fn cold_open_receives_a_revision_pinned_bulk_attachment() {
    with_app(|p| {
        p.records.push((37, b"NTS1hello".to_vec()));
        p.pump();
        assert_eq!(p.status()[1..5], [37, 5, 0, 0]);
        assert_eq!(p.storage_requests, 3); // LIST, HEAD, BULK; no publication.
        assert_eq!(p.text_image().len(), 46080);
    });
}
#[test]
fn uncertain_creation_reconciles_without_duplicate_or_draft_loss() {
    with_app(|p| {
        p.pump();
        p.text("first");
        p.save_error = true;
        p.commit_before_error = true;
        p.key(6);
        assert_eq!(p.status()[3], ERROR);
        assert_eq!(p.loss(), 1);
        assert_eq!(p.records.len(), 1);
        p.save_error = false;
        p.key(6);
        assert_eq!(p.loss(), 0);
        assert_eq!(p.records.len(), 1);
        p.text("draft");
        p.save_error = true;
        p.commit_before_error = false;
        p.key(6);
        p.save_error = false;
        p.key(6);
        assert_eq!(p.status()[2], 10);
        assert_eq!(p.loss(), 1);
        assert_eq!(p.records[0].1, b"NTS1first");
        p.key(6);
        assert_eq!(p.records[0].1, b"NTS1firstdraft");
    });
}
#[test]
fn malformed_or_truncated_records_are_not_accepted() {
    for bytes in [&b"MC01"[..], &b"NTS1bad\0"[..]] {
        with_app(|p| {
            p.records.push((1, bytes.to_vec()));
            p.pump();
            assert_eq!(p.status()[3], ERROR);
        });
    }
    with_app(|p| {
        p.records.push((1, b"NTS1hello".to_vec()));
        p.truncated_bulk = true;
        p.pump();
        assert_eq!(p.status()[3], ERROR);
    });
}
#[test]
fn optional_storage_retains_edits_and_close_requires_explicit_discard() {
    with_app(|p| {
        p.temporary = true;
        p.pump();
        p.text("draft");
        p.key(6);
        assert_ne!(p.status()[3], 0);
        assert_eq!(p.storage_requests, 0);
        assert_eq!(p.loss(), 1);
        p.click(310, 8, 41);
        assert!(!p.exited);
        p.click(20, 42, 42);
        assert!(!p.exited);
        assert_eq!(p.storage_requests, 0);
        p.click(20, 60, 43);
        assert!(p.exited);
    });
}
#[test]
fn close_save_commits_before_exit_and_cancel_keeps_draft() {
    with_app(|p| {
        p.pump();
        p.text("keep");
        p.click(310, 8, 41);
        assert!(!p.exited);
        p.click(20, 74, 42);
        p.text("x");
        assert_eq!(p.status()[2], 5);
        p.click(310, 8, 43);
        p.click(20, 42, 44);
        assert!(p.exited);
        assert_eq!(p.records[0].1, b"NTS1keepx");
    });
}
#[test]
fn bounded_text_retains_decorations_and_renders_font_pixels() {
    with_app(|p| {
        p.pump();
        p.text("a");
        // 'a' uses the existing 5x7 font: first row 01110, each dot doubled.
        assert_eq!(&p.text_image()[8..12], &0xffffffu32.to_le_bytes());
        let images = p.images.len();
        p.text("b");
        assert_eq!(p.images.len(), images + 1);
        for _ in 0..250 {
            p.text("c");
        }
        assert_eq!(p.status()[2], 240);
        p.key(6);
        assert_eq!(p.records[0].1.len(), 244);
    });
}
#[test]
fn hidden_edits_frame_requests_and_rebinding_preserve_state() {
    with_app(|p| {
        p.pump();
        let images = p.images.len();
        p.frames = false;
        p.text("a");
        assert_eq!(p.images.len(), images);
        p.frames = true;
        p.demand = true;
        p.wake();
        assert!(p.images.len() > images);
        assert!(p.commands.iter().any(|q| q[0] == wire::FRAME_READY));
        p.epoch += 1;
        p.wake();
        assert_eq!(p.status()[2], 1);
        assert_eq!(p.loss(), 1);
    });
}
#[test]
fn cursor_preferences_are_group_scoped_and_cancelled_streams_are_forgotten() {
    with_app(|p| {
        p.pump();
        p.event(wire::POINTER, 20, 20, 0, 41, 1);
        p.event(wire::POINTER, 20, 20, 0, 42, 2);
        p.key(12);
        let requests: std::vec::Vec<_> = p
            .commands
            .iter()
            .filter(|q| q[0] == wire::CURSOR_PREFERENCE)
            .collect();
        assert_eq!(requests.len(), 1);
        assert_eq!(requests[0][2..4], [41, 1]);
        p.event(wire::CANCEL, 0, 0, 0, 41, 1);
        p.key(12);
        assert_eq!(
            p.commands
                .iter()
                .filter(|q| q[0] == wire::CURSOR_PREFERENCE)
                .count(),
            1
        );
    });
}
