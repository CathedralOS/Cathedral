//! Exercise the compiled Omega state transitions, including adversarial replies.
use crate::policy::lifecycle::{self, Close, Survey};
use cathedral_kernel_contracts::user as abi;
#[test]
fn only_canonical_clean_answers_authorize_live_close() {
    assert_eq!(lifecycle::close(false, Ok([0; 8])), Close::Stop);
    for word in 0..8 {
        for value in [1, u64::MAX, abi::BUSY] {
            let mut reply = [0; 8];
            reply[word] = value;
            assert_eq!(lifecycle::close(false, Ok(reply)), Close::Refuse);
            assert_eq!(lifecycle::close(true, Ok(reply)), Close::AlreadyStopped);
        }
    }
    for error in [
        0,
        abi::TIMED_OUT,
        abi::PEER_CLOSED,
        abi::BUSY,
        abi::IO_ERROR,
    ] {
        assert_eq!(lifecycle::close(false, Err(error)), Close::Refuse);
        assert_eq!(lifecycle::close(true, Err(error)), Close::AlreadyStopped);
    }
    assert_eq!(lifecycle::close(true, Ok([0; 8])), Close::AlreadyStopped);
}
#[test]
fn busy_admission_and_recovery_cannot_report_clean() {
    for flags in 0..16 {
        assert_eq!(
            Survey::begin(core::array::from_fn(|i| flags & (1 << i) != 0)).is_some(),
            flags == 0
        );
    }
    let empty = Survey::begin([false; 4]).unwrap();
    assert_eq!(empty.finish(false), Some(false));
    assert_eq!(empty.finish(true), Some(true));
    for close in [false, true] {
        for busy in [false, true] {
            assert_eq!(lifecycle::wait(close, busy), close && busy);
        }
    }
}
#[test]
fn every_child_subset_waits_for_all_answers_and_preserves_uncertainty() {
    for mask in 0..32 {
        for dirty in 0..5 {
            let mut survey = Survey::begin([false; 4]).unwrap();
            for slot in 0..5 {
                if mask & (1 << slot) != 0 {
                    survey.sent(slot, true);
                }
            }
            let mut left = mask;
            for slot in (0..5).rev() {
                if mask & (1 << slot) == 0 {
                    continue;
                }
                assert_eq!(survey.finish(false), None);
                let mut answer = [0; 8];
                if slot == dirty {
                    answer[1] = 1;
                }
                survey.answer(slot, Ok(answer));
                left &= !(1 << slot);
                if left != 0 {
                    assert_eq!(survey.finish(false), None);
                }
            }
            assert_eq!(survey.finish(false), Some(mask & (1 << dirty) != 0));
        }
    }
}
#[test]
fn failed_send_timeout_and_malformed_reply_remain_uncertain_after_clean_siblings() {
    for failure in 0..10 {
        let mut survey = Survey::begin([false; 4]).unwrap();
        survey.sent(0, failure != 0);
        survey.sent(1, true);
        if failure == 1 {
            survey.answer(0, Err(abi::TIMED_OUT));
        }
        if failure >= 2 {
            let mut words = [0; 8];
            words[failure - 2] = 1;
            survey.answer(0, Ok(words));
        }
        assert_eq!(survey.finish(false), None);
        survey.answer(1, Ok([0; 8]));
        assert_eq!(survey.finish(false), Some(true));
    }
}
