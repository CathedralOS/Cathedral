//! Killing the root drops descendants; reopening the disk recovers committed
//! objects through newly authorized matrix connections.
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
    assert_eq!(text(&reopened, 2, 1), Err(abi::DENIED));
    grant(&reopened);
    reopened.owner(0, [wire::STORAGE_GRANT, 1, 1, 0, 0, 0, 0, 0]);
    expect(text(&reopened, 2, 1).unwrap(), b"nested root");
    expect(text(&reopened, 2, 2).unwrap(), b"nested copy");
    expect(text(&reopened, 3, 1).unwrap(), b"outside root");
    assert_eq!(listing(&reopened, 2).unwrap()[1], 2);
    assert_eq!(
        provider.wait_until(time::now().unwrap()),
        Err(Error(abi::TIMED_OUT as i64))
    );
}
