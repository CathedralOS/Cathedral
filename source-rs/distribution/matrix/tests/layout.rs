//! Live-layout refusal order and stock placement, independent of grant custody.
use crate::policy::layout;
use cathedral_desktop_matrix_contracts::matrix::{self as wire, config};
use cathedral_kernel_contracts::user as abi;
fn request() -> [u64; 8] {
    [wire::APPLY_LAYOUT, 42, 8, 80, 8, 0, 0, 0]
}
#[test]
fn layout_checks_authority_then_busy_then_canonical_request() {
    for word in [0, 5, 6, 7] {
        let mut malformed = request();
        malformed[word] ^= 1;
        assert_eq!(
            layout::prepare(malformed, false, true, [1024, 768], 31),
            Err(abi::DENIED)
        );
        assert_eq!(
            layout::prepare(malformed, true, true, [1024, 768], 31),
            Err(abi::BUSY)
        );
        assert_eq!(
            layout::prepare(malformed, true, false, [1024, 768], 31),
            Err(abi::INVALID_ARGUMENT)
        );
    }
}
#[test]
fn layout_maps_stock_roles_and_only_remembers_existing_app_placements() {
    for standing in [0, 1, 2, 4, 8, 14, 16, 31, u64::MAX] {
        let plan = layout::prepare(request(), true, false, [1024, 768], standing).unwrap();
        assert_eq!(plan.rectangles[0], [8, 680, 1008, 80]);
        assert_eq!(&plan.rectangles[1..4], &[[8, 8, 1008, 664]; 3]);
        assert_eq!(plan.rectangles[4], [0, 0, 1024, 768]);
        assert_eq!(plan.remember, standing & 14);
    }
}
#[test]
fn live_geometry_matches_checked_contract_at_zero_and_overflow_boundaries() {
    for width in [0, 1, 16, 1024, u64::MAX] {
        for height in [0, 1, 96, 768, u64::MAX] {
            for margin in [0, 8, u64::MAX / 2, u64::MAX] {
                for bar in [0, 1, 80, u64::MAX] {
                    for gap in [0, 8, u64::MAX] {
                        let words = [wire::APPLY_LAYOUT, 1, margin, bar, gap, 0, 0, 0];
                        let actual = layout::prepare(words, true, false, [width, height], 14);
                        match config::rectangles(margin, bar, gap, width, height) {
                            Some([chrome, app]) => {
                                let plan = actual.unwrap();
                                assert_eq!(
                                    plan.rectangles,
                                    [chrome, app, app, app, [0, 0, width, height]]
                                );
                            }
                            None => assert_eq!(actual, Err(abi::INVALID_ARGUMENT)),
                        }
                    }
                }
            }
        }
    }
}
