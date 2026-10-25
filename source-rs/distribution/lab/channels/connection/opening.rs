//! Client setup must yield to the peer's upstream work and reclaim every partial handoff.
use super::*;
use cathedral_user_runtime::{
    connection::Connecting,
    memory::shared::{Reader, Writer},
    notification,
    server::{OwnerControl, SharedSlot},
};

pub(super) fn run(launch: Launch) {
    resources::assign(
        launch,
        Limits {
            private_pages: 2,
            reader_pages: 2,
            channels: 8,
        },
    )
    .unwrap();
    let baseline = resources::budget().unwrap();
    let physical = resources::physical_budget().unwrap();
    let handles = notification::budget().unwrap();
    dependency(launch);
    assert_eq!(resources::budget().unwrap(), baseline);
    assert_eq!(resources::physical_budget().unwrap(), physical);
    assert_eq!(notification::budget().unwrap(), handles);
    bad_owner(launch);
    queued_grant();
    assert_eq!(resources::budget().unwrap(), baseline);
    assert_eq!(resources::physical_budget().unwrap(), physical);
    assert_eq!(notification::budget().unwrap(), handles);
    for phase in 0..=4 {
        for failure in 0..if phase == 3 { 5 } else { 4 } {
            partial(phase, failure);
            assert_eq!(resources::budget().unwrap(), baseline);
            assert_eq!(resources::physical_budget().unwrap(), physical);
            assert_eq!(notification::budget().unwrap(), handles);
        }
    }
    cathedral_user_runtime::write(b"Cathedral shared opening: upstream progress, partial handoffs, malformed replies, expiry and exact refunds passed\n").unwrap();
}
fn upstream(input: Handle, reply: Handle) -> bool {
    let mut bytes = [0; 64];
    match input.receive_until(&mut bytes, time::now().unwrap()) {
        Ok(n) => {
            assert_eq!(&bytes[..n], b"upstream");
            reply.send(b"served").unwrap();
            true
        }
        Err(Error(c)) if c == abi::TIMED_OUT as i64 => false,
        Err(error) => panic!("upstream service: {error:?}"),
    }
}
fn dependency(launch: Launch) {
    let (request, input) = ipc::channel().unwrap();
    let (reply, response) = ipc::channel().unwrap();
    let child = launch.spawn_with(101, &[request, response]).unwrap();
    let pair = Port::bootstrap().unwrap().connect().unwrap();
    let until = time::after(300).unwrap();
    let mut opening = Client::begin_connect(pair, until).unwrap();
    let mut serviced = 0;
    let mut client = loop {
        serviced += usize::from(upstream(input, reply));
        if let Some(client) = opening.poll().unwrap() {
            break client;
        }
        event::wait(
            &[opening.interest().unwrap(), Interest::Ipc(input)],
            opening.deadline(),
        )
        .unwrap();
    };
    assert_eq!(serviced, 1); // The child would not accept before its parent answered.
    assert!(matches!(opening.poll(), Err(Error(c)) if c == abi::PEER_CLOSED as i64));
    for _ in 0..8 {
        client.begin(b"control", None).unwrap();
        loop {
            serviced += usize::from(upstream(input, reply));
            if let Some((bytes, n, object)) = client.poll().unwrap() {
                assert_eq!(&bytes[..n], b"control");
                assert!(object.is_none());
                break;
            }
            let interests = client.interests().unwrap();
            event::wait(&[interests[0], interests[1], Interest::Ipc(input)], until).unwrap();
        }
    }
    assert_eq!(serviced, 9);
    client.retire();
    reply.send(b"control closed; independent wake").unwrap();
    assert_eq!(
        child.wait_until(time::after(300).unwrap()).unwrap(),
        Outcome::Returned(0)
    );
    input.close().unwrap();
    reply.close().unwrap();
}
pub(super) fn worker(env: Environment) -> u64 {
    let call_parent = || {
        env.endpoint(0).unwrap().send(b"upstream").unwrap();
        let mut bytes = [0; 64];
        let n = env
            .endpoint(1)
            .unwrap()
            .receive_until(&mut bytes, time::after(300).unwrap())
            .unwrap();
        assert_eq!(&bytes[..n], b"served");
    };
    call_parent();
    static mut SHARED: SharedSlot = SharedSlot::empty();
    let slot = &raw mut SHARED;
    // SAFETY: One worker entry owns the slot until process exit.
    let mut owner = OwnerControl::open(unsafe { &mut *slot }).unwrap();
    let independent = env.endpoint(1).unwrap();
    let mut calls = 0;
    loop {
        match owner
            .next(Interest::Ipc(independent), Some(time::after(300).unwrap()))
            .unwrap()
        {
            Some((bytes, len)) => {
                assert_eq!(
                    owner.next(Interest::Ipc(independent), None),
                    Err(Error(abi::BUSY as i64))
                );
                call_parent();
                owner.reply(&bytes[..len]).unwrap();
                assert_eq!(owner.reply(&bytes[..len]), Err(Error(abi::DENIED as i64)));
                calls += 1;
            }
            None => {
                assert_eq!(calls, 8);
                let mut bytes = [0; 64];
                let n = independent
                    .receive_until(&mut bytes, time::now().unwrap())
                    .unwrap();
                assert_eq!(&bytes[..n], b"control closed; independent wake");
                return 0;
            }
        }
    }
}

struct Peer {
    input: Handle,
    output: Handle,
    writer: Writer,
    reader: Option<Reader>,
    signal: Option<N>,
    rights: [N; 3],
}
impl Peer {
    fn start() -> (Self, Connecting<1>) {
        let (send, input) = ipc::channel().unwrap();
        let (output, receive) = ipc::channel().unwrap();
        let peer = Self {
            input,
            output,
            writer: Writer::allocate(1).unwrap(),
            reader: None,
            signal: None,
            rights: N::create().unwrap(),
        };
        let opening = Client::begin_connect((send, receive), time::after(30).unwrap()).unwrap();
        peer.message(wire::OPEN);
        (peer, opening)
    }
    fn message(&self, expected: &[u8]) {
        let mut bytes = [0; 64];
        let n = self
            .input
            .receive_until(&mut bytes, time::now().unwrap())
            .unwrap();
        assert_eq!(&bytes[..n], expected);
    }
    fn advance(&mut self, opening: &mut Connecting<1>, phase: usize) {
        match phase {
            0 | 1 => self.output.send(wire::ACK).unwrap(),
            2 => self.writer.delegate(self.output).unwrap(),
            3 => self.rights[1].delegate(self.output, b"").unwrap(),
            _ => unreachable!(),
        }
        assert!(opening.poll().unwrap().is_none());
        match phase {
            0 => self.reader = Some(Reader::accept(self.input).unwrap()),
            1 => self.signal = N::accept(self.input, &mut [0; 64]).unwrap().1,
            2 => self.message(wire::ACK),
            3 => self.message(wire::COMMIT),
            _ => unreachable!(),
        }
    }
}
impl Drop for Peer {
    fn drop(&mut self) {
        self.input.close().ok();
        self.output.close().ok();
        for right in self.rights.into_iter().chain(self.signal.take()) {
            right.release().unwrap();
        }
    }
}
fn partial(phase: usize, failure: usize) {
    let (mut peer, mut opening) = Peer::start();
    for step in 0..phase {
        peer.advance(&mut opening, step);
    }
    assert!(opening.poll().unwrap().is_none());
    if failure == 0 {
        drop(opening); // Abandon each partial handoff, including before final ACK.
        if let Some(reader) = &peer.reader {
            assert_eq!(reader.load(0).unwrap(), 0); // Already accepted backing survives.
        }
        return;
    }
    match failure {
        1 => peer.output.send(b"malformed").unwrap(),
        2 => {
            assert_eq!(
                ipc::wait_many(&[peer.input], opening.deadline()),
                Err(Error(abi::TIMED_OUT as i64))
            );
        }
        3 => peer.output.close().unwrap(),
        4 => peer.rights[0].delegate(peer.output, b"").unwrap(),
        _ => unreachable!(),
    }
    let error = opening.poll().err().expect("incomplete setup must fail");
    if failure == 2 {
        assert_eq!(error, Error(abi::TIMED_OUT as i64));
    }
    if failure == 4 {
        assert_eq!(error, Error(abi::DENIED as i64));
    }
    assert!(matches!(opening.interest(), Err(Error(c)) if c == abi::PEER_CLOSED as i64));
    assert!(matches!(opening.poll(), Err(Error(c)) if c == abi::PEER_CLOSED as i64));
    let budget = resources::budget().unwrap();
    let handles = notification::budget().unwrap();
    drop(opening);
    assert_eq!(resources::budget().unwrap(), budget); // Error already released custody.
    assert_eq!(notification::budget().unwrap(), handles);
}

fn queued_grant() {
    let (peer, mut opening) = Peer::start();
    peer.output.send(wire::ACK).unwrap();
    assert!(opening.poll().unwrap().is_none()); // Writer grant published but not accepted.
    drop(opening);
    let reader = Reader::accept(peer.input).unwrap();
    assert_eq!(reader.load(0).unwrap(), 0);
    drop(reader);
}

// Invalid setup must disable only the owner receiver, without spinning on its
// now-closed endpoints or consuming the independent input source.
fn bad_owner(launch: Launch) {
    let (wake, input) = ipc::channel().unwrap();
    let child = launch.spawn_with(102, &[input]).unwrap();
    let pair = Port::bootstrap().unwrap().connect().unwrap();
    pair.0.send(wire::OPEN).unwrap();
    let mut bytes = [0; 64];
    let n = pair
        .1
        .receive_until(&mut bytes, time::after(300).unwrap())
        .unwrap();
    assert_eq!(&bytes[..n], wire::ACK);
    pair.0.send(b"not a writer grant").unwrap();
    assert_eq!(
        pair.1.receive_until(&mut bytes, time::after(300).unwrap()),
        Err(Error(abi::PEER_CLOSED as i64))
    );
    wake.send(b"still alive").unwrap();
    assert_eq!(
        child.wait_until(time::after(300).unwrap()).unwrap(),
        Outcome::Returned(0)
    );
    pair.0.close().unwrap();
    pair.1.close().unwrap();
    wake.close().unwrap();
}
pub(super) fn worker_fault(env: Environment) -> u64 {
    static mut SHARED: SharedSlot = SharedSlot::empty();
    let slot = &raw mut SHARED;
    // SAFETY: This process enters this worker once.
    let mut owner = OwnerControl::open(unsafe { &mut *slot }).unwrap();
    let input = env.endpoint(0).unwrap();
    assert!(
        owner
            .next(Interest::Ipc(input), Some(time::after(300).unwrap()))
            .unwrap()
            .is_none()
    );
    let mut bytes = [0; 64];
    let n = input
        .receive_until(&mut bytes, time::now().unwrap())
        .unwrap();
    assert_eq!(&bytes[..n], b"still alive");
    0
}
