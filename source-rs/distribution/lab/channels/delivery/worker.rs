//! No registry, service role, launch authority or knowledge of the parent's peers.
use super::*;
pub fn run(mode: u64) -> u64 {
    let input = Handle::bootstrap(0).unwrap();
    let output = Handle::bootstrap(1).unwrap();
    assert!(cathedral_user_runtime::task::Launch::bootstrap().is_err());
    if mode == 1 {
        let (_send, recv) = ipc::channel().unwrap();
        output.send(b"parked").unwrap();
        let _ = recv.receive(&mut [0; 64]);
        return 254;
    }
    if mode == 2 {
        output.send(b"waiting for closure").unwrap();
        assert!(
            matches!(input.receive_with(&mut [0;64]), Err(Error(code)) if code == abi::PEER_CLOSED as i64)
        );
        return 43;
    }
    let (raw, cap) = receive(input);
    assert!(cap.is_none());
    let foreign = Handle::from_raw(u64::from_le_bytes(raw[..8].try_into().unwrap()));
    assert_eq!(foreign.close(), Err(Error(abi::BAD_HANDLE as i64)));
    output.send(b"no authority from bytes").unwrap();
    ipc::wait_many(&[input], time::after(300).unwrap()).unwrap();
    probes::receive(input);
    let (_, requests) = receive(input);
    let requests = requests.unwrap();
    output.send(b"requests installed").unwrap();
    let (_, answers) = input.receive_with(&mut [0; 64]).unwrap();
    let answers = answers.unwrap();
    output.send(b"answers installed").unwrap();
    let mut bytes = [0; 64];
    let len = requests
        .receive_until(&mut bytes, time::after(300).unwrap())
        .unwrap();
    answers.send(&bytes[..len]).unwrap();
    42 // Kernel closes owned endpoints; already queued reply remains deliverable.
}
