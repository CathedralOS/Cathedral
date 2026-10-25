//! Exercise the common Server, including an app-held Request across disconnection.
use super::*;
use cathedral_user_runtime::{
    mailbox::Mailbox,
    memory::shared::{Reader, Writer},
    server::{Event, Server},
};
pub(crate) fn run(mode: u64, env: Environment) -> u64 {
    let pair = (env.endpoint(0).unwrap(), env.endpoint(1).unwrap());
    if mode == 55 {
        return wrong_reply(pair);
    }
    let mut server = Server::open_clients(1).unwrap();
    server.install(0, pair).unwrap();
    server.enable_shared(0).unwrap();
    let mut held = None;
    loop {
        match server.next_event().unwrap() {
            Event::Request(request) if request.control => {
                assert_eq!(&request.bytes[..request.len], b"alive");
                server.reply(&request, b"alive").unwrap();
            }
            Event::Request(request) => {
                assert_eq!(request.client, Some(0));
                if mode == 54 {
                    return 0;
                }
                if mode == 51 {
                    held = Some(request);
                } else {
                    server
                        .reply(&request, &request.bytes[..request.len])
                        .unwrap();
                }
            }
            Event::Disconnected { client, .. } => {
                assert_eq!(client, 0);
                if let Some(old) = held {
                    let (send, input) = ipc::channel().unwrap();
                    let (output, recv) = ipc::channel().unwrap();
                    server.install(0, (input, output)).unwrap();
                    server.enable_shared(0).unwrap();
                    server.reply(&old, b"late").unwrap();
                    assert_eq!(
                        recv.receive_until(&mut [0; 64], time::now().unwrap()),
                        Err(Error(abi::TIMED_OUT as i64))
                    );
                    server.remove(0);
                    for h in [send, input, output, recv] {
                        h.close().unwrap();
                    }
                }
                Handle::bootstrap(1).unwrap().send(b"gone").unwrap();
                return 0;
            }
            _ => panic!("unexpected attachment"),
        }
    }
}
fn wrong_reply((input, output): (Handle, Handle)) -> u64 {
    let mut bytes = [0; 64];
    let n = input.receive(&mut bytes).unwrap();
    assert_eq!(&bytes[..n], wire::OPEN);
    let writer = Writer::allocate(1).unwrap();
    let bell = N::create().unwrap();
    output.send(wire::ACK).unwrap();
    let reader = Reader::accept(input).unwrap();
    output.send(wire::ACK).unwrap();
    let (_, signal) = N::accept(input, &mut bytes).unwrap();
    let signal = signal.unwrap();
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
    let (id, payload) = wire::decode(&frame[..n]).unwrap();
    let (wrong, n) = wire::encode(id + 1, payload).unwrap();
    mailbox.try_send(&wrong[..n]).unwrap();
    signal.signal(false).unwrap();
    0
}
