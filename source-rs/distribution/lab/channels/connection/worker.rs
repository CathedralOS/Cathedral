//! Exercise the common Server, including an app-held Request across disconnection.
use super::*;
use cathedral_user_runtime::{
    mailbox::Mailbox,
    memory::shared::{Reader, Writer},
    server::{Event, Server},
};
pub(crate) fn run(mode: u64, env: Environment) -> u64 {
    let pair = (env.endpoint(0).unwrap(), env.endpoint(1).unwrap());
    if mode == 78 {
        return super::replacement::worker(mode, env);
    }
    if mode >= 75 {
        return super::pipeline::worker(mode, env);
    }
    if mode == 74 {
        return super::capacity::worker(env);
    }
    if mode == 73 {
        return super::replacement::worker(mode, env);
    }
    if mode >= 71 {
        return super::multiplex::worker(mode, env);
    }
    if mode >= 68 {
        return super::control::worker(mode, pair);
    }
    if mode == 55 || (60..=64).contains(&mode) {
        return wrong_reply(pair, mode);
    }
    let mut server = Server::open_clients(1).unwrap();
    server.install(0, pair).unwrap();
    static mut SHARED: cathedral_user_runtime::server::SharedSlot =
        cathedral_user_runtime::server::SharedSlot::empty();
    let slot = &raw mut SHARED;
    // SAFETY: Single-entry service owns this connection storage until exit.
    server.enable_shared(0, unsafe { &mut *slot }).unwrap();
    server.enable_memory();
    let mut held_object = None;
    let mut held = None;
    loop {
        match server.next_event().unwrap() {
            Event::Request(request) if request.control => {
                assert_eq!(&request.bytes[..request.len], b"alive");
                server.reply(&request, b"alive").unwrap();
            }
            Event::Memory(request, object) => {
                if mode == 58 {
                    held = Some(request);
                    held_object = Some(object);
                } else {
                    assert_eq!(mode, 56);
                    server
                        .reply_memory(&request, &request.bytes[..request.len], &object)
                        .unwrap();
                }
            }
            Event::Request(request) => {
                if mode == 57 || mode == 59 {
                    let mut page = cathedral_user_runtime::memory::Private::allocate(1).unwrap();
                    page.bytes_mut().fill(mode as u8);
                    let object = page.freeze().unwrap();
                    if let Err(error) =
                        server.reply_memory(&request, &request.bytes[..request.len], &object)
                    {
                        assert_eq!(mode, 57);
                        assert_eq!(error, Error(abi::NO_MEMORY as i64));
                        server.reply(&request, b"capacity").unwrap();
                    }
                    if mode == 59 {
                        return 0;
                    }
                    continue;
                }
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
                    if let Some(object) = held_object.take() {
                        server.reply_memory(&old, b"late", &object).unwrap();
                    } else {
                        server.reply(&old, b"late").unwrap();
                    }
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
        }
    }
}
fn wrong_reply((input, output): (Handle, Handle), mode: u64) -> u64 {
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
    let (id, payload, _) = wire::decode(&frame[..n]).unwrap();
    if mode == 55 {
        let (wrong, n) = wire::encode(id + 1, payload, false).unwrap();
        mailbox.try_send(&wrong[..n]).unwrap();
    } else {
        if mode != 60 {
            let object = cathedral_user_runtime::memory::Private::allocate(1)
                .unwrap()
                .freeze()
                .unwrap();
            output
                .send_memory(
                    &(if mode == 62 || mode == 64 { id + 1 } else { id }).to_le_bytes(),
                    &object,
                )
                .unwrap();
        }
        if mode != 61 {
            let (frame, n) =
                wire::encode(if mode == 64 { id + 1 } else { id }, payload, mode != 63).unwrap();
            mailbox.try_send(&frame[..n]).unwrap();
        }
    }
    signal.signal(false).unwrap();
    if (60..=64).contains(&mode) {
        assert_eq!(
            input.receive(&mut [0; 64]),
            Err(Error(abi::PEER_CLOSED as i64))
        );
    }
    0
}
