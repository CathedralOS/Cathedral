use super::*;
#[test]
fn chrome_launches_visible_app_and_presents_bounded_images() {
    with_app(false, |p| {
        p.pump();
        assert!(!p.exited);
        assert!(!p.images.is_empty());
        p.click(32, 40);
        assert!(p.commands.iter().any(|q| q[0] == 14 && q[1] == 3));
        assert!(
            !p.commands
                .iter()
                .any(|q| q[0] == 14 && (q[1] == 2 || q[1] == 5))
        );
    });
}
#[test]
fn picker_copies_template_then_launches_new_user() {
    with_app(true, |p| {
        p.entries = vec![entry(2, "Template")];
        p.pump();
        p.click(512, 384);
        assert_eq!(p.entries.len(), 2);
        assert!(
            p.storage_commands
                .iter()
                .any(|q| q[0] == storage::COPY_TREE)
        );
        p.click(436, 384);
        assert!(p.commands.iter().any(|q| q[0] == 14 && q[1] == 3));
    });
}
#[test]
fn catalogue_streams_full_pages_and_rejects_truncated_payload() {
    with_app(false, |p| {
        p.entries = (2..65).map(|id| entry(id, "App")).collect();
        p.pump();
        let pages: Vec<_> = p
            .storage_commands
            .iter()
            .filter(|q| q[0] == storage::ENTRIES)
            .collect();
        assert_eq!(pages.len(), 2);
        assert_eq!(pages[1][3], 52);
        assert!(!p.exited);
    });
    with_app(false, |p| {
        p.malformed_entries = true;
        p.pump();
        assert!(!p.exited);
        assert!(!p.commands.iter().any(|q| q[0] == 14));
    });
}
#[test]
fn settings_save_and_loss_follow_the_actual_draft() {
    with_app(false, |p| {
        p.pump();
        p.click(930, 64);
        assert_eq!(p.loss(), 0);
        p.click(376, 60);
        assert_eq!(p.loss(), 1);
        p.click(648, 60);
        assert_eq!(p.saves(), 1);
        assert_eq!(p.loss(), 0);
        assert_eq!(u64::from_le_bytes(p.config[72..80].try_into().unwrap()), 96);
        assert!(p.commands.iter().any(|q| q[0] == 16 && q[3] == 96));
    });
}
#[test]
fn lost_save_acknowledgement_is_read_back_without_duplicate_publish() {
    with_app(false, |p| {
        p.pump();
        p.click(930, 64);
        p.click(376, 60);
        p.fail_save = true;
        p.commit_before_error = true;
        p.click(648, 60);
        assert_eq!(p.loss(), 1);
        assert_eq!(p.saves(), 1);
        p.fail_save = false;
        p.click(648, 60);
        assert_eq!(p.saves(), 1);
        assert_eq!(p.loss(), 0);
    });
}
#[test]
fn intervening_configuration_is_never_overwritten() {
    with_app(false, |p| {
        p.pump();
        p.click(930, 64);
        p.click(376, 60);
        p.config[64..72].copy_from_slice(&24u64.to_le_bytes());
        p.revision += 1;
        p.click(648, 60);
        assert_eq!(p.saves(), 0);
        assert_eq!(p.loss(), 1);
    });
}
#[test]
fn new_epoch_rebuilds_surfaces_after_an_image_race() {
    with_app(false, |p| {
        p.race = true;
        p.pump();
        assert!(!p.exited);
        assert!(p.next_surface >= 516);
        assert!(!p.images.is_empty());
    });
}

#[test]
fn confirmation_cannot_stop_a_replacement_task() {
    with_app(false, |p| {
        p.managed = 77;
        p.pump();
        p.click(930, 20); // Open force-stop confirmation for ticket 77.
        p.managed = 78;
        p.wake();
        p.click(500, 30); // The old confirmation no longer applies.
        assert!(!p.commands.iter().any(|q| q[0] == 19));
        p.click(930, 20);
        p.click(500, 30);
        assert!(p.commands.iter().any(|q| q[0] == 19 && q[1] == 78));
    });
}

#[test]
fn task_close_selects_the_observed_ticket_first() {
    with_app(false, |p| {
        p.tasks[0] = [0, 91, 3, 0, 0, 0, 0, 0];
        p.pump();
        p.click(260, 40); // Task strip starts at 128; close region starts at 252.
        let select = p
            .commands
            .iter()
            .position(|q| q[0] == 24 && q[1] == 91)
            .unwrap();
        let close = p.commands.iter().position(|q| q[0] == 15).unwrap();
        assert!(select < close);
    });
}

#[test]
fn settings_codec_preserves_authority_fields_and_rejects_bad_records() {
    use cathedral_desktop_matrix_contracts::matrix::{VALID_KEY_MASK, config::Config};
    with_app(false, |p| {
        p.config[8..40].fill(0);
        let name = "./桌面".as_bytes();
        p.config[8..8 + name.len()].copy_from_slice(name);
        p.config[96..104].copy_from_slice(&VALID_KEY_MASK.to_le_bytes());
        let original = Config::decode(&p.config).unwrap();
        p.pump();
        p.click(930, 64);
        p.click(376, 60);
        p.click(648, 60);
        let saved = Config::decode(&p.config).unwrap();
        assert_eq!(
            saved,
            Config {
                launcher_height: 96,
                ..original
            }
        );
    });
    for bad in [0, 1, 2, 3] {
        with_app(false, |p| {
            match bad {
                0 => p.config[8] = 0xff,
                1 => p.config[8] = 0, // Trailing nonzero bytes are not canonical.
                2 => p.config[40..48].copy_from_slice(&129u64.to_le_bytes()),
                _ => p.config[96..104].copy_from_slice(&u64::MAX.to_le_bytes()),
            }
            assert!(Config::decode(&p.config).is_none());
            p.pump();
            p.click(930, 64);
            p.click(376, 60);
            p.click(648, 60);
            assert_eq!(p.loss(), 0);
            assert_eq!(p.saves(), 0);
        });
    }
}

#[test]
fn idle_refresh_removes_a_collected_task_without_an_input_event() {
    with_app(false, |p| {
        p.tasks[0] = [0, 91, 3, 1, 0, 0, 0, 0];
        p.pump();
        assert_eq!(p.refresh, 100);
        p.tasks[0] = [0; 8];
        p.wake(); // The host's timeout resumes with an empty wake.
        let (_, image) = p.images.iter().rev().find(|(q, _)| q[5] == 432).unwrap();
        assert!(
            image
                .chunks_exact(4)
                .all(|pixel| pixel == 0x101827u32.to_le_bytes())
        );
        assert!(!p.commands.iter().any(|q| matches!(q[0], 14 | 15 | 19 | 24)));
    });
}
