//! Killing the root drops descendants and their transaction; reopening the disk
//! recovers committed catalogs through newly authorized matrix connections.
use super::*;
pub(super) fn exercise(f: &Fixture, provider: cathedral_user_runtime::task::Child) {
    request(f, 2, store::BEGIN, 0, listing(f, 2).generation, &[]).unwrap();
    request(f, 2, store::STAGE_DELETE, 2, 0, &[]).unwrap();
    f.root.cancel().unwrap();
    f.root.wait().unwrap();
    provider.cancel().unwrap();
    provider.wait().unwrap();
    let provider = Launch::at(4).unwrap().spawn(0).unwrap();
    let control = Port::at(4).unwrap().connect().unwrap();
    control.0.send(store::HEALTH).unwrap();
    assert_eq!(control.1.receive(&mut [0; 64]).unwrap(), 5);
    let reopened = Fixture::open();
    assert_eq!(
        request(&reopened, 2, store::READ, 2, 0, &[]),
        Err(abi::DENIED)
    );
    grant(&reopened);
    reopened.owner(0, [wire::STORAGE_GRANT, 1, 1, 0, 0, 0, 0, 0]);
    assert_eq!(
        request(&reopened, 2, store::COMMIT, 0, 0, &[]),
        Err(abi::BAD_HANDLE)
    );
    assert_eq!(
        request(&reopened, 2, store::READ, 2, 0, &[])
            .unwrap()
            .payload(),
        b"committed nested"
    );
    assert_eq!(
        request(&reopened, 3, store::READ, 2, 0, &[])
            .unwrap()
            .payload(),
        b"outside catalog"
    );
    assert_eq!(
        provider.wait_until(time::now().unwrap()),
        Err(Error(abi::TIMED_OUT as i64))
    );
}
