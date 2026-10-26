//! Reconnect to a faulted provider without reusing its transport or authority.
use cathedral_kernel_contracts::user as abi;
use cathedral_platform_contracts::display as wire;
use cathedral_user_runtime::{
    Error,
    connection::Client,
    ipc::Handle,
    server::{Server, SharedSlot},
    task::{Child, Port},
    time, write,
};

pub fn run() -> u64 {
    static mut CONTROL: SharedSlot = SharedSlot::empty();
    let slot = &raw mut CONTROL;
    let mut owner = Server::open_clients(0).unwrap();
    // SAFETY: One entry in this process; the server exclusively owns the slot.
    owner.enable_shared_control(unsafe { &mut *slot }).unwrap();
    let port = Port::bootstrap().unwrap();
    let mut previous: Option<(Client, (Handle, Handle))> = None;
    for round in 0..2 {
        let command = owner.next_request().unwrap();
        assert_eq!(command.len, 8);
        let stolen = Child::from_raw(u64::from_le_bytes(command.bytes[..8].try_into().unwrap()));
        assert_eq!(stolen.cancel(), Err(Error(abi::BAD_HANDLE as i64)));
        if let Some((mut old, (send, receive))) = previous.take() {
            old.retire();
            // Intentional raw-handle probes, never application traffic. The new
            // provider incarnation must not revive either retired local name.
            assert_eq!(send.send(b"stale"), Err(Error(abi::BAD_HANDLE as i64)));
            assert_eq!(
                receive.receive(&mut [0; 64]),
                Err(Error(abi::BAD_HANDLE as i64))
            );
        }
        let pair = port.connect().unwrap();
        let mut display = Client::connect(pair, time::after(200).unwrap()).unwrap();
        write(b"Cathedral display: shared provider connection ready\n").unwrap();
        let info = call(&mut display, [wire::INFO, 0, 0, 0, 0, 0]);
        assert_eq!(info[0], 0);
        let (w, h) = (info[1], info[2]);
        assert!(w >= 640 && h >= 480);
        let (bytes, len) = display.call_retained(b"malformed").unwrap();
        assert_eq!(len, wire::REQUEST_BYTES);
        assert_eq!(
            wire::decode(&bytes[..len]).unwrap()[0],
            abi::INVALID_ARGUMENT
        );
        assert_eq!(
            call(&mut display, [wire::RECT, u64::MAX, 0, 2, 1, 0])[0],
            abi::INVALID_ARGUMENT
        );
        cathedral_boot_scene::draw(&mut display, w, h).unwrap();
        let reply = display.call_retained(&wire::encode([wire::INFO, 0, 0, 0, 0, 0]));
        if round == 0 {
            assert_eq!(reply, Err(Error(abi::PEER_CLOSED as i64)));
        } else {
            let (bytes, len) = reply.unwrap();
            assert_eq!(len, wire::REQUEST_BYTES);
            assert_eq!(wire::decode(&bytes[..len]).unwrap(), info);
        }
        owner.reply(&command, &[round]).unwrap();
        previous = Some((display, pair));
    }
    let stop = owner.next_request().unwrap();
    assert_eq!(&stop.bytes[..stop.len], b"stop");
    // Keep the second connection alive until its supervisor cancels the provider.
    // Dropping it earlier would turn the cancellation proof into a clean exit.
    assert_eq!(
        previous
            .as_mut()
            .unwrap()
            .0
            .call_retained(&wire::encode([wire::INFO, 0, 0, 0, 0, 0])),
        Err(Error(abi::PEER_CLOSED as i64))
    );
    drop(previous);
    owner.reply(&stop, b"stopped").unwrap();
    // Deliberately let the owner observe the reply before our reader is dropped.
    cathedral_user_runtime::yield_now();
    0
}
fn call(display: &mut Client, words: [u64; 6]) -> [u64; 6] {
    let (bytes, len) = display.call_retained(&wire::encode(words)).unwrap();
    assert_eq!(len, wire::REQUEST_BYTES);
    wire::decode(&bytes[..len]).unwrap()
}
