// Real shared payloads: setup grants once, publish words, then ring doorbells.
use cathedral_user_runtime::{
    mailbox::Mailbox,
    memory::{
        raw,
        shared::{Reader, Writer},
    },
    notification::{Notification as N, Outcome as Wake},
    resources::{self, Limits},
    task::{Launch, Outcome, Port},
};
const ROUNDS: usize = 128;
pub(super) fn payload(round: usize) -> ([u8; 256], usize) {
    let length = [0, 1, 7, 8, 63, 64, 255, 256][round % 8];
    (
        core::array::from_fn(|i| (i.wrapping_mul(17) ^ round) as u8),
        length,
    )
}
pub(super) fn run() {
    let baseline = resources::budget().unwrap();
    let physical = resources::physical_budget().unwrap();
    protocol::run();
    let launch = Launch::bootstrap().unwrap();
    resources::assign(
        launch,
        Limits {
            private_pages: 4,
            reader_pages: 4,
            channels: 8,
        },
    )
    .unwrap();
    ping_pong();
    for mode in [43, 44] {
        protection(mode);
    }
    retained();
    abandoned();
    resources::assign(
        launch,
        Limits {
            private_pages: 0,
            reader_pages: 0,
            channels: 8,
        },
    )
    .unwrap();
    assert_eq!(resources::budget().unwrap(), baseline);
    assert_eq!(resources::physical_budget().unwrap(), physical);
    cathedral_user_runtime::write(b"Cathedral shared IPC: payloads, backpressure, hostile headers, RO/NX, death and retained funding passed\n").unwrap();
}
fn ping_pong() {
    let output = Writer::allocate(1).unwrap();
    let name = output.handle();
    let request = N::create().unwrap();
    let reply = N::create().unwrap();
    let child = Launch::bootstrap().unwrap().spawn(40).unwrap();
    let (commands, events) = Port::bootstrap().unwrap().connect().unwrap();
    commands.send(&name.to_le_bytes()).unwrap();
    receive(events);
    output.delegate(commands).unwrap();
    let input = Reader::accept(events).unwrap();
    request[0].delegate(commands, b"wait").unwrap();
    receive(events);
    reply[1].delegate(commands, b"signal").unwrap();
    receive(events);
    let mut channel = Mailbox::new(output, input).unwrap();
    let charged = resources::budget().unwrap();
    for round in 0..ROUNDS {
        let (bytes, length) = payload(round);
        channel.try_send(&bytes[..length]).unwrap();
        request[1].signal(false).unwrap();
        let mut received = [0; 256];
        loop {
            if let Some(n) = channel.try_receive(&mut received).unwrap() {
                assert_eq!(n, length);
                assert_eq!(&received[..n], &bytes[..length]);
                break;
            }
            assert_eq!(
                reply[0].wait(Some(time::after(300).unwrap())),
                Ok(Wake::Signaled)
            );
        }
        // Coalesced/early signals are hints; readiness is the published sequence.
        if round % 7 == 0 {
            request[1].signal(false).unwrap();
        }
    }
    assert_eq!(child.wait().unwrap(), Outcome::Returned(0));
    // The child's last reply backing remains readable until this local view dies.
    assert_eq!(resources::budget().unwrap().used, charged.used);
    drop(channel);
    assert_eq!(raw::address(name), Err(Error(abi::BAD_HANDLE as i64)));
    for handle in request.into_iter().chain(reply) {
        handle.release().unwrap();
    }
}
fn protection(mode: u64) {
    let mut output = Writer::allocate(1).unwrap();
    output.store(0, 0xc3).unwrap(); // x86 RET, if execute permission were accidentally granted.
    let child = Launch::bootstrap().unwrap().spawn(mode).unwrap();
    let (commands, events) = Port::bootstrap().unwrap().connect().unwrap();
    output.delegate(commands).unwrap();
    let (bytes, _) = receive(events);
    let expected = u64::from_le_bytes(bytes[..8].try_into().unwrap());
    let Outcome::Fault {
        vector,
        error,
        address,
        ..
    } = child.wait().unwrap()
    else {
        panic!("shared view did not fault");
    };
    assert_eq!(vector, 14);
    assert_eq!(address, expected);
    assert_eq!(error & 0x17, if mode == 43 { 7 } else { 0x15 });
    assert_eq!(output.load(0).unwrap(), 0xc3);
}
fn retained() {
    let child = Launch::bootstrap().unwrap().spawn(41).unwrap();
    let (commands, events) = Port::bootstrap().unwrap().connect().unwrap();
    let mut reader = Reader::accept(events).unwrap();
    commands.send(b"accepted").unwrap();
    assert_eq!(child.wait().unwrap(), Outcome::Returned(0));
    reader.unmap().unwrap();
    assert_eq!(reader.load(0), Err(Error(abi::BAD_ADDRESS as i64)));
    reader.map().unwrap();
    assert_eq!(reader.load(0).unwrap(), 0x12345678);
    let next = Launch::bootstrap().unwrap().spawn(42).unwrap();
    let (commands, events) = Port::bootstrap().unwrap().connect().unwrap();
    receive(events); // Replacement observes the predecessor's pinned page charge.
    drop(reader);
    commands.send(b"released").unwrap();
    assert_eq!(next.wait().unwrap(), Outcome::Returned(0));
}
fn abandoned() {
    let before = resources::budget().unwrap().used;
    let writer = Writer::allocate(1).unwrap();
    let child = Launch::bootstrap().unwrap().spawn(45).unwrap();
    let (commands, events) = Port::bootstrap().unwrap().connect().unwrap();
    receive(events);
    assert_eq!(
        syscall::call(abi::IPC_SEND_SHARED, commands.raw(), 1, 0),
        abi::BAD_ADDRESS
    );
    writer.delegate(commands).unwrap(); // Recipient never accepts this escrow.
    drop(writer);
    child.cancel().unwrap();
    assert_eq!(child.wait().unwrap(), Outcome::Cancelled);
    assert_eq!(resources::budget().unwrap().used, before);
}
