//! Distribution recovery policy: lost input interrupts holds, preserving route
//! selection; lost display retires
//! the root. Rebuild adapters/tree explicitly, restoring only committed app data.
use super::*;
fn app(matrix: &mut RootControl, nested: bool, op: u64) -> wire::Packet {
    let w = wire::route([op, 0, 0, 0, 0, 0, 0, 0], if nested { 0 } else { 1 }).unwrap();
    call(
        matrix,
        if nested {
            wire::route(w, 0).unwrap()
        } else {
            w
        },
    )
}
pub(super) fn exercise(mut session: admission::Session) -> admission::Session {
    // A rebuilt capped subtree may still be loading its apps. Geometry admission
    // is not app readiness: observe both initial scenes before starting saves.
    pixel(&mut session.matrix, 140, 190, app::LEFT);
    pixel(&mut session.matrix, 510, 190, app::RIGHT);
    for nested in [true, false] {
        assert_eq!(app(&mut session.matrix, nested, 8)[0], 0);
    }
    for display_loss in [false, true] {
        pixel(&mut session.matrix, 140, 190, 0xd18b35);
        pixel(&mut session.matrix, 510, 190, 0xd18b35);
        if display_loss {
            session.display_task.cancel().unwrap();
            session.display_task.wait().unwrap();
            assert_eq!(
                session.matrix.exchange(
                    [wire::READ_PIXEL, 140, 190, 0, 0, 0, 0, 0],
                    time::after(300).unwrap()
                ),
                Err(Error(abi::PEER_CLOSED as i64))
            );
            assert_eq!(
                session.root.wait().unwrap(),
                cathedral_user_runtime::task::Outcome::Returned(1)
            );
        } else {
            let pointers = app(&mut session.matrix, true, wire::HEALTH)[6];
            owner(
                &mut session.matrix,
                [wire::POINTER_SOURCE, 170, 220, 1, 0, 0, 0, 0],
            );
            owner(
                &mut session.matrix,
                [wire::POINTER_SOURCE, 170, 220, 0, 0, 0, 0, 0],
            );
            // Selection must reach the nested app before killing its source.
            // Cancellation may legitimately discard an undelivered click;
            // that is not evidence that an established route was forgotten.
            let deadline = time::after(1000).unwrap();
            while app(&mut session.matrix, true, wire::HEALTH)[6] < pointers + 2 {
                assert!(
                    !time::reached(time::now().unwrap(), deadline),
                    "focus click not consumed"
                );
            }
            cathedral_user_runtime::write(
                b"Cathedral matrix desktop: focus click consumed before source loss\n",
            )
            .unwrap();
            session.start_input().expect("lab input admission");
            let input = session.input.unwrap();
            input.cancel().unwrap();
            input.wait().unwrap();
            // Allow source disconnect processing before injecting a fresh press
            // through the explicitly trusted owner adapter. Cancellation retains
            // route selection; it does not make this owner endpoint unauthorized.
            let _ = session.matrix.idle(time::after(10).unwrap());
            for state in [input::PRESS, input::RELEASE] {
                owner(
                    &mut session.matrix,
                    [
                        wire::KEY_SOURCE,
                        input::ENTER as u64,
                        state as u64,
                        0,
                        0,
                        0,
                        0,
                        0,
                    ],
                );
            }
            pixel(&mut session.matrix, 140, 190, app::LEFT);
            assert_eq!(app(&mut session.matrix, true, wire::HEALTH)[4], 0);
            assert_eq!(app(&mut session.matrix, false, wire::HEALTH)[4], 1);
            // Restore the persisted value for the subsequent whole-tree recovery.
            assert_eq!(app(&mut session.matrix, true, 8)[0], 0);
            pixel(&mut session.matrix, 140, 190, 0xd18b35);
        }
        session.stop();
        session = setup(false);
        pixel(&mut session.matrix, 140, 190, 0xd18b35);
        pixel(&mut session.matrix, 510, 190, 0xd18b35);
    }
    for nested in [true, false] {
        assert_eq!(app(&mut session.matrix, nested, 8)[0], 0);
    }
    cathedral_user_runtime::write(b"Cathedral matrix desktop: input loss preserved route selection; display loss retired root; explicit tree recovery restored persisted apps\n").unwrap();
    session
}
