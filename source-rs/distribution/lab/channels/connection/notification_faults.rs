//! Malformed notification grants cannot satisfy another call or survive retirement.
use super::*;
use cathedral_user_runtime::{
    mailbox::Mailbox,
    memory::shared::{Reader, Writer},
};
pub(super) fn run(launch: Launch) {
    for mode in 94..=98 {
        let baseline = resources::budget().unwrap();
        let physical = resources::physical_budget().unwrap();
        let notifications = cathedral_user_runtime::notification::budget().unwrap();
        let (send, input) = ipc::channel().unwrap();
        let (output, recv) = ipc::channel().unwrap();
        let child = launch.spawn_with(mode, &[input, output]).unwrap();
        let _control = Port::bootstrap().unwrap().connect().unwrap();
        let mut client = Client::connect((send, recv), time::after(300).unwrap()).unwrap();
        let result = client.call_retained_notification(b"request");
        if mode == 98 {
            let (_, _, grant) = result.unwrap();
            let wait = grant.unwrap().into_handle();
            assert_eq!(
                wait.wait(Some(time::after(300).unwrap())),
                Ok(Wake::Signaled)
            );
            wait.release().unwrap();
            client.retire();
        } else {
            assert_eq!(
                result.err(),
                Some(Error(if mode == 95 || mode == 96 {
                    abi::TIMED_OUT
                } else {
                    abi::IO_ERROR
                } as i64))
            );
        }
        assert_eq!(
            child.wait_until(time::after(300).unwrap()).unwrap(),
            Outcome::Returned(0)
        );
        drop(client);
        assert_eq!(resources::budget().unwrap(), baseline);
        assert_eq!(resources::physical_budget().unwrap(), physical);
        assert_eq!(
            cathedral_user_runtime::notification::budget().unwrap(),
            notifications
        );
    }
    failed_signal(launch);
}
pub(super) fn worker(mode: u64, env: Environment) -> u64 {
    let (input, output) = (env.endpoint(0).unwrap(), env.endpoint(1).unwrap());
    let mut bytes = [0; 64];
    assert_eq!(input.receive(&mut bytes).unwrap(), wire::OPEN.len());
    let writer = Writer::allocate(1).unwrap();
    let bell = N::create().unwrap();
    let [wait, signal, control] = N::create().unwrap();
    output.send(wire::ACK).unwrap();
    let reader = Reader::accept(input).unwrap();
    output.send(wire::ACK).unwrap();
    let (_, receiver) = N::accept(input, &mut bytes).unwrap();
    let receiver = receiver.unwrap();
    writer.delegate(output).unwrap();
    input.receive(&mut bytes).unwrap();
    bell[1].delegate(output, b"").unwrap();
    input.receive(&mut bytes).unwrap();
    output.send(wire::ACK).unwrap();
    let mut mailbox = Mailbox::new(writer, reader).unwrap();
    let mut frame = [0; wire::FRAME_BYTES];
    let n = loop {
        if let Some(n) = mailbox.try_receive(&mut frame).unwrap() {
            break n;
        }
        assert_eq!(bell[0].wait(None), Ok(Wake::Signaled));
    };
    let (id, _, _) = wire::decode(&frame[..n]).unwrap();
    let (frame, n) = wire::encode(
        id,
        b"wait",
        if mode == 97 {
            wire::Attachment::Memory
        } else {
            wire::Attachment::Notification
        },
    )
    .unwrap();
    if mode == 98 {
        mailbox.try_send(&frame[..n]).unwrap();
        receiver.signal(false).unwrap();
        assert_eq!(
            event::wait(&[], time::after(2).unwrap()),
            Err(Error(abi::TIMED_OUT as i64))
        );
    }
    if mode != 96 {
        wait.delegate(
            output,
            &(if mode == 94 { id + 1 } else { id }).to_le_bytes(),
        )
        .unwrap();
    }
    if mode != 95 && mode != 98 {
        mailbox.try_send(&frame[..n]).unwrap();
    }
    signal.signal(false).unwrap();
    receiver.signal(false).unwrap();
    assert_eq!(
        input.receive(&mut bytes),
        Err(Error(abi::PEER_CLOSED as i64))
    );
    control.close_object().unwrap();
    for right in bell.into_iter().chain([wait, signal, control, receiver]) {
        right.release().unwrap();
    }
    0
}

// After escrow/frame publication, signal failure must not replay delegation or
// consume the provider's source right. The accepted wait grant remains usable.
fn failed_signal(launch: Launch) {
    let baseline = resources::budget().unwrap();
    let physical = resources::physical_budget().unwrap();
    let notifications = cathedral_user_runtime::notification::budget().unwrap();
    let (send, input) = ipc::channel().unwrap();
    let (output, recv) = ipc::channel().unwrap();
    let child = launch.spawn_with(100, &[input, output]).unwrap();
    let (control, _) = Port::bootstrap().unwrap().connect().unwrap();
    let writer = Writer::allocate(1).unwrap();
    let bell = N::create().unwrap();
    let ack = || {
        let mut bytes = [0; 64];
        assert_eq!(recv.receive(&mut bytes).unwrap(), wire::ACK.len());
    };
    send.send(wire::OPEN).unwrap();
    ack();
    writer.delegate(send).unwrap();
    ack();
    bell[1].delegate(send, b"").unwrap();
    let reader = Reader::accept(recv).unwrap();
    send.send(wire::ACK).unwrap();
    let (_, signal) = N::accept(recv, &mut [0; 64]).unwrap();
    let signal = signal.unwrap();
    send.send(wire::COMMIT).unwrap();
    ack();
    let mut mailbox = Mailbox::new(writer, reader).unwrap();
    bell[2].close_object().unwrap();
    let (frame, n) = wire::encode(1, b"wait", wire::Attachment::None).unwrap();
    mailbox.try_send(&frame[..n]).unwrap();
    signal.signal(false).unwrap();
    let (n, right) = N::accept(recv, &mut [0; 64]).unwrap();
    assert_eq!(n, 8);
    let right = right.unwrap();
    // Escrow can be accepted before the provider publishes its frame. Failed
    // final signalling then retires the endpoint; observe that boundary first.
    ipc::wait_many(&[recv], time::after(300).unwrap()).unwrap();
    let mut frame = [0; wire::FRAME_BYTES];
    let n = mailbox.try_receive(&mut frame).unwrap().unwrap();
    assert_eq!(
        wire::decode(&frame[..n]),
        Some((1, b"wait".as_slice(), wire::Attachment::Notification))
    );
    assert_eq!(right.wait(Some(time::now().unwrap())), Ok(Wake::TimedOut));
    assert_eq!(right.signal(false), Err(Error(abi::DENIED as i64)));
    right.release().unwrap();
    control.send(b"done").unwrap();
    assert_eq!(
        child.wait_until(time::after(300).unwrap()).unwrap(),
        Outcome::Returned(0)
    );
    drop(mailbox);
    for right in bell.into_iter().chain([signal]) {
        right.release().unwrap();
    }
    send.close().unwrap();
    recv.close().unwrap();
    assert_eq!(resources::budget().unwrap(), baseline);
    assert_eq!(resources::physical_budget().unwrap(), physical);
    assert_eq!(
        cathedral_user_runtime::notification::budget().unwrap(),
        notifications
    );
}
