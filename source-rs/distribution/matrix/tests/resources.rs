//! Compiled Omega offers must fit owner ceilings without changing profile semantics.
use crate::policy::resources::{self as p, Allocation, Facts};
use cathedral_desktop_matrix_contracts::matrix as wire;
use cathedral_kernel_contracts::user as abi;
use cathedral_platform_contracts::imports;
fn offered() -> Facts {
    Facts {
        desktop: true,
        app_slot: true,
        pages: [256, 384],
        profile: None,
        resources: None,
        channels: 40,
        descendants: 5,
    }
}
#[test]
fn stock_roles_keep_distinct_offers_and_background_has_no_storage_authority() {
    let launcher = p::launcher(72);
    assert_eq!(launcher.pages, [72, wire::CHOOSER_READERS]);
    assert_eq!(
        (launcher.channels, launcher.descendants, launcher.cpu),
        (3, 0, 0)
    );
    assert_eq!(launcher.imports, imports::ALL);
    assert_eq!(launcher.surfaces, wire::config::LAUNCHER_SURFACES as u64);
    let background = p::background();
    assert_eq!(background.pages, [5, 2]);
    assert_eq!(
        (background.channels, background.descendants, background.cpu),
        (2, 0, 0)
    );
    assert_eq!(background.imports, imports::INTERACTION);
    assert_eq!(background.surfaces, 1);
    let app = p::workspace([88, 93], 17);
    assert_eq!(app.pages, [88, 93]);
    assert_eq!((app.channels, app.descendants, app.cpu), (40, 5, 0));
    assert_eq!(app.surfaces, 17);
}
#[test]
fn picker_requires_host_capacity_and_reserves_its_own_surfaces_from_user_offer() {
    for backed in 0..wire::config::HOST_SURFACES {
        assert_eq!(p::picker(backed), Err(abi::NO_MEMORY));
        assert_eq!(p::user(backed), Err(abi::NO_MEMORY));
    }
    for backed in [wire::config::HOST_SURFACES, 40, 256, usize::MAX] {
        let picker = p::picker(backed).unwrap();
        let user = p::user(backed).unwrap();
        assert_eq!(picker.pages, [48, wire::CHOOSER_READERS]);
        assert_eq!((picker.channels, picker.descendants), (3, 0));
        assert_eq!(user.pages, [256, 384]);
        assert_eq!((user.channels, user.descendants), (24, 3));
        assert_eq!(picker.surfaces + user.surfaces, backed as u64);
        assert_eq!(picker.imports, imports::ALL);
        assert_eq!(user.imports, imports::ALL);
    }
}
#[test]
fn explicit_profile_refuses_every_excess_instead_of_silently_clamping() {
    for (profile, resources) in [
        ([257, 384, 10], Some((40, 5))),
        ([256, 385, 10], Some((40, 5))),
        ([256, 384, 10], Some((41, 5))),
        ([256, 384, 10], Some((40, 6))),
        ([u64::MAX, 0, 0], None),
    ] {
        assert_eq!(
            p::allocate(Facts {
                profile: Some(profile),
                resources,
                ..offered()
            }),
            Err(abi::NO_MEMORY)
        );
    }
    let accepted = p::allocate(Facts {
        profile: Some([256, 384, 23]),
        resources: Some((40, 5)),
        ..offered()
    })
    .unwrap();
    assert_eq!(
        accepted,
        Allocation {
            pages: [256, 384],
            channels: 40,
            descendants: 5,
            surfaces: Some(23)
        }
    );
    // Surface authority is checked later, after checked transport funding arithmetic.
    assert_eq!(
        p::allocate(Facts {
            profile: Some([0, 0, u64::MAX]),
            ..offered()
        })
        .unwrap()
        .surfaces,
        Some(u64::MAX)
    );
}
#[test]
fn profile_applies_to_all_roles_and_absent_resources_retain_the_offer() {
    for desktop in [false, true] {
        for app_slot in [false, true] {
            let result = p::allocate(Facts {
                desktop,
                app_slot,
                profile: Some([7, 8, 0]),
                ..offered()
            })
            .unwrap();
            assert_eq!(
                result,
                Allocation {
                    pages: [7, 8],
                    channels: 40,
                    descendants: 5,
                    surfaces: Some(0)
                }
            );
        }
    }
}
#[test]
fn unprofiled_apps_are_bounded_but_host_roles_keep_their_offers() {
    assert_eq!(
        p::allocate(offered()).unwrap(),
        Allocation {
            pages: [128, 128],
            channels: 40,
            descendants: 5,
            surfaces: Some(10)
        }
    );
    let nested = p::allocate(Facts {
        resources: Some((u64::MAX, u64::MAX)),
        ..offered()
    })
    .unwrap();
    assert_eq!(
        nested,
        Allocation {
            pages: [168, 144],
            channels: 40,
            descendants: 5,
            surfaces: Some(10)
        }
    );
    // Declaring descendants selects the host default before the offer clamps them.
    let refused_descendants = p::allocate(Facts {
        resources: Some((2, 1)),
        descendants: 0,
        ..offered()
    })
    .unwrap();
    assert_eq!(refused_descendants.pages, [168, 144]);
    assert_eq!(refused_descendants.descendants, 0);
    for (desktop, app_slot) in [(false, false), (false, true), (true, false)] {
        let result = p::allocate(Facts {
            desktop,
            app_slot,
            ..offered()
        })
        .unwrap();
        assert_eq!(result.pages, [256, 384]);
        assert_eq!(result.surfaces, None);
    }
    for pages in [[0, 0], [1, 2], [127, 129], [u64::MAX, u64::MAX]] {
        for resources in [None, Some((1, 0)), Some((1, 1))] {
            let result = p::allocate(Facts {
                pages,
                resources,
                ..offered()
            })
            .unwrap();
            assert!(result.pages[0] <= pages[0] && result.pages[1] <= pages[1]);
            assert!(result.pages[0] <= 168 && result.pages[1] <= 144);
        }
    }
}
#[test]
fn surface_preference_preserves_owner_ceiling_and_room_for_chrome_and_background() {
    assert_eq!(
        p::preferred_surfaces(11, 10, 0, false, 256),
        Err(abi::NO_MEMORY)
    );
    assert_eq!(p::preferred_surfaces(10, 60, 0, true, 256), Ok(10));
    assert_eq!(p::preferred_surfaces(10, 60, 1, false, 256), Ok(16));
    assert_eq!(p::preferred_surfaces(10, 60, 1, true, 256), Ok(60));
    assert_eq!(p::preferred_surfaces(10, 12, 1, true, 256), Ok(12));
    for backed in [0, 8, 9, 10, 23, 24, 40, usize::MAX] {
        let preferred = p::preferred_surfaces(10, u64::MAX, 1, true, backed).unwrap();
        assert_eq!(
            preferred,
            backed.saturating_sub(wire::config::LAUNCHER_SURFACES + 1) as u64
        );
    }
    // A preferred range smaller than the minimum must produce no allocation;
    // no new pages are conjured and the host refuses before spawning.
    let preferred = p::preferred_surfaces(20, 60, 1, false, 256).unwrap();
    assert_eq!((20..=preferred).rev().next(), None);
}
