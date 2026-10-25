//! Ordinary worker with explicit notification grants; it owns no ambient wake authority.
use super::*;
pub(crate) fn worker(mode: u64) -> u64 {
    let input = Handle::bootstrap(0).unwrap();
    let output = Handle::bootstrap(1).unwrap();
    if mode == 15 {
        let [_, signal, _] = N::create().unwrap();
        signal.delegate(output, b"retained").unwrap();
        receive(input);
        return 0; // Parent's accepted grant keeps this account charged.
    }
    if mode == 16 {
        assert_eq!(notification::budget().unwrap()[1], 1);
        output.send(b"charged across replacement").unwrap();
        receive(input);
        assert_eq!(notification::budget().unwrap()[1], 0);
        return 0;
    }
    let (bytes, _) = receive(input);
    let foreign = N::from_raw(u64::from_le_bytes(bytes[..8].try_into().unwrap()));
    assert_eq!(foreign.signal(false), Err(Error(abi::BAD_HANDLE as i64)));
    output.send(b"no authority from bytes").unwrap();
    ipc::wait_many(&[input], time::after(300).unwrap()).unwrap();
    assert_eq!(
        syscall::call(abi::NOTIFY_RECEIVE, input.raw(), 1, 0),
        abi::BAD_ADDRESS
    );
    assert_eq!(
        input.receive(&mut [0; 64]),
        Err(Error(abi::TOO_SMALL as i64))
    );
    let (_, wait) = N::accept(input, &mut [0; 64]).unwrap();
    let wait = wait.unwrap();
    output.send(b"wait accepted").unwrap();
    let (_, signal) = N::accept(input, &mut [0; 64]).unwrap();
    let signal = signal.unwrap();
    output.send(b"signal accepted").unwrap();
    if mode == 10 {
        assert_eq!(wait.wait(None), Ok(Wake::Signaled));
        for _ in 1..ROUNDS {
            assert_eq!(signal.signal_and_wait(wait, None), Ok(Wake::Signaled));
        }
        signal.signal(false).unwrap();
        return 0;
    }
    let result = signal.signal_and_wait(wait, None);
    match mode {
        17 => {
            assert_eq!(result, Ok(Wake::Signaled));
            output.send(b"one completion").unwrap();
        }
        11 => {
            assert_eq!(result, Ok(Wake::Cancelled));
            assert_eq!(wait.wait(Some(time::now().unwrap())), Ok(Wake::TimedOut));
        }
        12 => {
            assert_eq!(result, Ok(Wake::Revoked));
            assert_eq!(wait.wait(None), Err(Error(abi::REVOKED as i64)));
        }
        13 => {
            assert_eq!(result, Ok(Wake::Closed));
            assert_eq!(wait.wait(None), Err(Error(abi::PEER_CLOSED as i64)));
        }
        _ => panic!("cancelled task resumed"),
    }
    0
}
