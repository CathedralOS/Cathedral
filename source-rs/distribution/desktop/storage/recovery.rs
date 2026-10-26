//! Killing the root drops descendants; reopening the disk recovers committed
//! objects and their tree through newly authorized matrix connections.
use super::*;
pub(super) fn exercise(
    f: Fixture,
    provider: Child,
    mut control: Client,
) -> (Fixture, Child, Client) {
    f.root.cancel().unwrap();
    f.root.wait().unwrap();
    // Retire the old shared owner before a new fixture can consume its funding.
    drop(f);
    provider.cancel().unwrap();
    provider.wait().unwrap();
    control.retire();
    let used = cathedral_user_runtime::resources::budget().unwrap().used;
    assert_eq!(
        [used.private_pages, used.reader_pages, used.channels],
        [0; 3]
    );
    let notifications = cathedral_user_runtime::notification::budget().unwrap();
    assert_eq!([notifications[1], notifications[3]], [0; 2]);
    cathedral_user_runtime::write(
        b"Cathedral matrix fixture: retired control resources reclaimed\n",
    )
    .unwrap();
    let (provider, mut control) = start_provider();
    let mut reopened = Fixture::open();
    assert_eq!(text(&mut reopened, 2, 2), Err(abi::DENIED));
    grant(&mut reopened);
    reopened.owner(0, [wire::STORAGE_GRANT, 1, 1, 0, 0, 0, 0, 0]);
    expect(text(&mut reopened, 2, 2).unwrap(), b"nested root");
    expect(text(&mut reopened, 2, 3).unwrap(), b"nested copy");
    expect(text(&mut reopened, 2, 5).unwrap(), b"inside realm");
    expect(text(&mut reopened, 2, 7).unwrap(), b"inside copy");
    expect(text(&mut reopened, 2, 8).unwrap(), b"program state");
    expect(text(&mut reopened, 3, 1).unwrap(), b"outside root");
    assert_eq!(listing(&mut reopened, 2).unwrap()[1..7], [5, 1, 2, 3, 4, 6]);
    regrant_scoped(&mut reopened, 6);
    assert_eq!(listing(&mut reopened, 2).unwrap()[1..3], [1, 7]);
    regrant_scoped(&mut reopened, 1);
    assert_eq!(listing(&mut reopened, 2).unwrap()[1..4], [2, 8, 9]);
    regrant_scoped(&mut reopened, 9);
    assert_eq!(listing(&mut reopened, 2).unwrap()[1..3], [1, 10]);
    expect(text(&mut reopened, 2, 10).unwrap(), b"sibling state");
    let (bytes, length) = control
        .call(store::HEALTH, time::after(300).unwrap())
        .unwrap();
    assert_eq!(&bytes[..length], b"ready");
    assert_eq!(
        provider.wait_until(time::now().unwrap()),
        Err(Error(abi::TIMED_OUT as i64))
    );
    (reopened, provider, control)
}
