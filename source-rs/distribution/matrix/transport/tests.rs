use super::*;
use cathedral_kernel_contracts::user as abi;

#[test]
fn interleaved_success_refusal_and_stale_poll_are_distinct() {
    for status in [0, abi::DENIED, abi::TIMED_OUT] {
        let reply = wire::error(status);
        assert_eq!(classify(reply, Some(7), true), Reply::Presentation(status));
        assert_eq!(
            classify(wire::tag_poll(reply, 7), Some(7), true),
            Reply::Poll(reply)
        );
        assert_eq!(
            classify(wire::tag_poll(reply, 8), Some(7), true),
            Reply::Unmatched(8)
        );
        assert_eq!(classify(reply, Some(7), false), Reply::Invalid);
    }
}

#[test]
fn last_concurrent_id_cannot_alias_a_negative_status() {
    let denied = wire::error(abi::DENIED);
    assert_eq!(
        classify(wire::tag_poll(denied, LAST_POLL), Some(LAST_POLL), true),
        Reply::Poll(denied)
    );
    assert_eq!(
        classify(denied, Some(LAST_POLL), true),
        Reply::Presentation(abi::DENIED)
    );
    assert_eq!(
        classify([0, 1, 0, 0, 0, 0, 0, 0], None, true),
        Reply::Invalid
    );
}
