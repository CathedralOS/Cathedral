//! Correlated control of the storage fault fixture, never unsolicited replies.
//! The provider implementation is platform/services/storage/gate.rs.
use super::*;
use cathedral_platform_contracts::storage;
use persistence::Provider;

pub(super) fn arm(provider: &mut Provider, phase: u8) {
    expect(
        provider,
        &[0xf2, phase, storage::PUBLISH as u8, 0],
        b"armed",
    );
}
pub(super) fn blocked(provider: &mut Provider) {
    let deadline = time::after(500).unwrap();
    loop {
        let (bytes, n) = provider.command(&[0xf6]);
        if &bytes[..n] == b"blocked" {
            return;
        }
        assert_eq!(&bytes[..n], b"armed");
        assert!(
            !time::reached(time::now().unwrap(), deadline),
            "storage gate not reached"
        );
    }
}
pub(super) fn release(provider: &mut Provider) {
    expect(provider, &[0xf3], b"released");
}
pub(super) fn expect(provider: &mut Provider, request: &[u8], reply: &[u8]) {
    let (bytes, n) = provider.command(request);
    assert_eq!(&bytes[..n], reply);
}
