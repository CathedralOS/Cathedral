//! A disposable producer and an independent capacity-filling sibling.
use super::*;
pub fn run(mode: u64) -> u64 {
    let input = Handle::bootstrap(0).unwrap();
    let output = Handle::bootstrap(1).unwrap();
    let budget = ipc::budget().unwrap();
    assert_eq!(budget.charged_handles, 0);
    if mode == 8 {
        assert_eq!(budget.charged_channels, 0);
        let mut channels = [None; 8];
        for pair in channels.iter_mut().take(budget.channel_limit as usize) {
            *pair = Some(ipc::channel().unwrap());
        }
        assert!(matches!(ipc::channel(), Err(Error(code)) if code == abi::NO_MEMORY as i64));
        assert_eq!(
            ipc::budget().unwrap().charged_channels,
            budget.channel_limit
        );
        output.send(b"reserved").unwrap();
        input.receive(&mut [0; 64]).unwrap();
        return 254; // Owner cancels while this task retains its full reservation.
    }
    let mut bytes = [0; 32];
    for (word, target) in [
        budget.channel_limit,
        budget.charged_channels,
        budget.handle_limit,
        budget.charged_handles,
    ]
    .into_iter()
    .zip(bytes.chunks_exact_mut(8))
    {
        target.copy_from_slice(&word.to_le_bytes());
    }
    output.send(&bytes).unwrap();
    input.receive(&mut [0; 64]).unwrap();
    match ipc::channel() {
        Ok((send, receive)) => {
            output.send_with(b"retained", receive).unwrap();
            send.send(b"survives").unwrap();
            71 // Only the transferred receiver survives; its channel remains charged here.
        }
        Err(Error(code)) if code == abi::NO_MEMORY as i64 => {
            assert_eq!(budget.charged_channels, budget.channel_limit);
            output.send(b"exhausted").unwrap();
            input.receive(&mut [0; 64]).unwrap();
            assert_eq!(
                ipc::budget().unwrap().charged_channels,
                budget.channel_limit - 1
            );
            let (send, receive) = ipc::channel().unwrap();
            send.close().unwrap();
            receive.close().unwrap();
            output.send(b"recovered").unwrap();
            72
        }
        Err(error) => panic!("unexpected channel failure: {error:?}"),
    }
}
