//! Attachment correlation, refusal before publication, and custody after peer death.
use super::*;
use cathedral_user_runtime::memory::Private;
pub(super) fn run(launch: Launch) {
    for mode in 56..=64 {
        let (send, input) = ipc::channel().unwrap();
        let (output, recv) = ipc::channel().unwrap();
        let child = launch.spawn_with(mode, &[input, output]).unwrap();
        let (_, events) = Port::bootstrap().unwrap().connect().unwrap();
        let mut client = Client::connect((send, recv), time::after(300).unwrap()).unwrap();
        match mode {
            56 => {
                for value in 0..32 {
                    let mut page = Private::allocate(1).unwrap();
                    page.bytes_mut().fill(value);
                    let object = page.freeze().unwrap();
                    let (bytes, n, returned) = client
                        .call_memory(b"echo", Some(&object), time::after(300).unwrap())
                        .unwrap();
                    assert_eq!(&bytes[..n], b"echo");
                    let mut returned = returned.unwrap();
                    assert!(returned.bytes().is_none());
                    assert!(returned.map().unwrap().iter().all(|b| *b == value));
                }
                client.retire();
                assert_eq!(&receive(events).0[..4], b"gone");
            }
            57 => {
                // Reserve every spare reader page elsewhere. A failed typed
                // offer must still permit a plain refusal on this same request.
                let budget = resources::budget().unwrap();
                let spare = budget.limits.reader_pages - budget.used.reader_pages;
                let idle = Launch::at(4).unwrap();
                resources::assign(
                    idle,
                    Limits {
                        private_pages: 0,
                        reader_pages: spare,
                        channels: 4,
                    },
                )
                .unwrap();
                let (bytes, n, object) = client
                    .call_memory(b"capacity", None, time::after(300).unwrap())
                    .unwrap();
                assert_eq!(&bytes[..n], b"capacity");
                assert!(object.is_none());
                resources::assign(
                    idle,
                    Limits {
                        private_pages: 0,
                        reader_pages: 0,
                        channels: 4,
                    },
                )
                .unwrap();
                let (_, _, object) = client
                    .call_memory(b"capacity", None, time::after(300).unwrap())
                    .unwrap();
                assert!(object.is_some());
                drop(object);
                client.retire();
                assert_eq!(&receive(events).0[..4], b"gone");
            }
            58 => {
                let object = Private::allocate(1).unwrap().freeze().unwrap();
                assert!(
                    matches!(client.call_memory(b"held",Some(&object),time::after(3).unwrap()),Err(Error(c)) if c==abi::TIMED_OUT as i64)
                );
                assert_eq!(&receive(events).0[..4], b"gone");
            }
            59 => {
                let (_, _, object) = client
                    .call_memory(b"final", None, time::after(300).unwrap())
                    .unwrap();
                let mut object = object.unwrap();
                assert_eq!(
                    child.wait_until(time::after(300).unwrap()).unwrap(),
                    Outcome::Returned(0)
                );
                assert!(object.map().unwrap().iter().all(|b| *b == 59));
                assert!(resources::budget().unwrap().used.reader_pages >= 1);
                drop(object);
                client.retire();
                continue;
            }
            _ => {
                let start = time::now().unwrap();
                let result = client.call_memory(b"bad", None, time::after(600).unwrap());
                let expected = if mode <= 61 {
                    abi::TIMED_OUT
                } else {
                    abi::IO_ERROR
                };
                assert!(matches!(result,Err(Error(c)) if c==expected as i64));
                if mode <= 61 {
                    assert!(time::now().unwrap().wrapping_sub(start) < 500);
                }
            }
        }
        assert_eq!(
            child.wait_until(time::after(300).unwrap()).unwrap(),
            Outcome::Returned(0)
        );
    }
    adversarial::run(launch);
    cathedral_user_runtime::write(b"Cathedral shared attachments: correlated grants, capacity refusal, stale reply, orphan expiry and retained backing passed\n").unwrap();
}
