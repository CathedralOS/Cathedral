//! Opt-in hostile peer: exhaust page/object grants and flood a compositor link.
//! Its private control protocol is consumed by supervisor/desktop/pressure.rs.
use cathedral_kernel_contracts::memory as m;
use cathedral_kernel_contracts::user as abi;
use cathedral_platform_contracts::display;
use cathedral_user_runtime::{
    Error,
    connection::Pipeline,
    link::Link,
    memory::Private,
    server::{Event, Server, SharedSlot},
    time,
};
pub const STATUS: u64 = 0;
pub const SPIN: u64 = 1;
pub const FAULT: u64 = 2;

pub fn run() -> u64 {
    static mut CONTROL: SharedSlot = SharedSlot::empty();
    let slot = &raw mut CONTROL;
    let mut control = Server::open_clients(0).unwrap();
    // SAFETY: One pressure process lends its static slot to its only server.
    control
        .enable_shared_control(unsafe { &mut *slot })
        .unwrap();
    control
        .establish_control(time::after(300).unwrap())
        .unwrap();
    let link = Link::at(0).unwrap();
    let mut peer =
        Some(Pipeline::<4>::connect(link.connect().unwrap(), time::after(300).unwrap()).unwrap());
    let mut stats = [0u64; 6]; // status, cycles, page denials, object denials, sends, backpressure
    let mut pending = None;
    let small = core::array::from_fn::<_, { m::MAX_PRIVATE_REGIONS - 2 }, _>(|_| {
        Private::allocate(1).unwrap()
    });
    assert!(matches!(Private::allocate(1), Err(Error(code)) if code == abi::NO_MEMORY as i64));
    stats[3] += 1;
    drop(small);
    loop {
        // Check only the capability/closure endpoint, never the shared reply
        // mailbox. A full local call window must not hide provider death.
        if peer.as_ref().is_some_and(|peer| {
            match cathedral_user_runtime::event::wait(
                &peer.interests().unwrap()[..1],
                time::now().unwrap(),
            ) {
                Ok(_) => true,
                Err(Error(code))
                    if code == abi::BAD_HANDLE as i64 || code == abi::PEER_CLOSED as i64 =>
                {
                    true
                }
                Err(Error(code)) if code == abi::TIMED_OUT as i64 => false,
                Err(error) => panic!("pressure liveness: {error:?}"),
            }
        }) {
            peer = None;
        }
        if peer.is_none() {
            peer = match link.connect() {
                Ok(pair) => Some(Pipeline::<4>::connect(pair, time::after(300).unwrap()).unwrap()),
                Err(Error(code)) if code == abi::PEER_CLOSED as i64 || code == abi::BUSY as i64 => {
                    None
                }
                Err(error) => panic!("pressure connect: {error:?}"),
            };
        }
        // Charge both live mailbox writers separately from the full-size object.
        // During provider recovery its writer is absent, so skip the exact-limit
        // probe until reconnecting instead of assuming it still consumes funding.
        let mut buffer = peer
            .as_ref()
            .map(|_| Private::allocate(m::MAX_PAGES).unwrap());
        if let Some(buffer) = &mut buffer {
            assert!(buffer.bytes_mut().iter().all(|&byte| byte == 0));
            buffer.bytes_mut().fill(0xa5);
            assert!(
                matches!(Private::allocate(1), Err(Error(code)) if code == abi::NO_MEMORY as i64)
            );
            stats[1] += 1;
            stats[2] += 1;
        }
        if let Some(pair) = &mut peer {
            // No scene token/authority. Never drain replies: a full response
            // queue must not stall control, legitimate clients or input.
            for _ in 0..8 {
                match pair.submit(&display::encode([0; 6]), None) {
                    Ok(_) => stats[4] += 1,
                    Err(Error(code))
                        if code == abi::WOULD_BLOCK as i64 || code == abi::BUSY as i64 =>
                    {
                        stats[5] += 1
                    }
                    Err(Error(code))
                        if code == abi::PEER_CLOSED as i64 || code == abi::BAD_HANDLE as i64 =>
                    {
                        peer = None;
                        break;
                    }
                    Err(error) => panic!("pressure send: {error:?}"),
                }
            }
        }
        if pending.is_none() {
            match control.next_event_for(&[], Some(time::now().unwrap())) {
                Ok(Some(Event::Request(request))) => pending = Some(request),
                Ok(_) => (),
                Err(error) => panic!("pressure control: {error:?}"),
            }
        }
        if buffer.is_some()
            && let Some(request) = pending.take()
        {
            let command = display::decode(&request.bytes[..request.len]).unwrap()[0];
            control.reply(&request, &display::encode(stats)).unwrap();
            match command {
                STATUS => (),
                SPIN => loop {
                    core::hint::spin_loop(); // No syscall/yield; timer must let the session run.
                },
                FAULT => {
                    // SAFETY: Deliberate isolated fault with live private pages and queued IPC.
                    unsafe {
                        core::arch::asm!("ud2", options(noreturn));
                    }
                }
                _ => panic!("unknown pressure control"),
            }
        }
        drop(buffer);
    }
}
