//! Killing the root drops descendants; reopening the disk recovers committed
//! objects and their tree through newly authorized matrix connections.
use super::*;
pub(super) fn exercise(f: &Fixture, provider: cathedral_user_runtime::task::Child) {
    f.root.cancel().unwrap();
    f.root.wait().unwrap();
    provider.cancel().unwrap();
    provider.wait().unwrap();
    let provider = Launch::at(4).unwrap().spawn(0).unwrap();
    let control = Port::at(4).unwrap().connect().unwrap();
    control.0.send(store::HEALTH).unwrap();
    assert_eq!(control.1.receive(&mut [0; 64]).unwrap(), 5);
    let reopened = Fixture::open();
    assert_eq!(text(&reopened, 2, 2), Err(abi::DENIED));
    grant(&reopened);
    reopened.owner(0, [wire::STORAGE_GRANT, 1, 1, 0, 0, 0, 0, 0]);
    expect(text(&reopened, 2, 2).unwrap(), b"nested root");
    expect(text(&reopened, 2, 3).unwrap(), b"nested copy");
    expect(text(&reopened, 2, 5).unwrap(), b"inside realm");
    expect(text(&reopened, 2, 7).unwrap(), b"inside copy");
    expect(text(&reopened, 2, 8).unwrap(), b"program state");
    expect(text(&reopened, 3, 1).unwrap(), b"outside root");
    assert_eq!(listing(&reopened, 2).unwrap()[1..7], [5, 1, 2, 3, 4, 6]);
    regrant_scoped(&reopened, 6);
    assert_eq!(listing(&reopened, 2).unwrap()[1..3], [1, 7]);
    regrant_scoped(&reopened, 1);
    assert_eq!(listing(&reopened, 2).unwrap()[1..4], [2, 8, 9]);
    regrant_scoped(&reopened, 9);
    assert_eq!(listing(&reopened, 2).unwrap()[1..3], [1, 10]);
    expect(text(&reopened, 2, 10).unwrap(), b"sibling state");
    assert_eq!(
        provider.wait_until(time::now().unwrap()),
        Err(Error(abi::TIMED_OUT as i64))
    );
}
