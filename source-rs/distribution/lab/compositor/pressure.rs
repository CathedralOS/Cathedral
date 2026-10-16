//! Opt-in hostile peer: exhaust page/object grants and flood a compositor link.
//! Its private control protocol is consumed by supervisor/desktop/pressure.rs.
use cathedral_kernel_contracts::memory as m;
use cathedral_kernel_contracts::user as abi;
use cathedral_platform_contracts::display;
use cathedral_user_runtime::{Error, ipc::Handle, link::Link, memory::Private, time};
pub const STATUS: u64 = 0;
pub const SPIN: u64 = 1;
pub const FAULT: u64 = 2;

pub fn run() -> u64 {
    let control = (Handle::bootstrap(0).unwrap(), Handle::bootstrap(1).unwrap());
    let link = Link::at(0).unwrap();
    let mut peer = None;
    let mut stats = [0u64; 6]; // status, cycles, page denials, object denials, sends, backpressure
    let small =
        core::array::from_fn::<_, { m::MAX_PRIVATE_REGIONS }, _>(|_| Private::allocate(1).unwrap());
    assert!(matches!(Private::allocate(1), Err(Error(code)) if code == abi::NO_MEMORY as i64));
    stats[3] += 1;
    drop(small);
    loop {
        let mut buffer = Private::allocate(m::MAX_PAGES).unwrap();
        assert!(buffer.bytes_mut().iter().all(|&byte| byte == 0));
        buffer.bytes_mut().fill(0xa5);
        assert!(matches!(Private::allocate(1), Err(Error(code)) if code == abi::NO_MEMORY as i64));
        stats[1] += 1;
        stats[2] += 1;
        if peer.is_none() {
            peer = match link.connect() {
                Ok(pair) => Some(pair),
                Err(Error(code)) if code == abi::PEER_CLOSED as i64 || code == abi::BUSY as i64 => {
                    None
                }
                Err(error) => panic!("pressure connect: {error:?}"),
            };
        }
        if let Some(pair) = peer {
            // No scene token/authority. Never drain replies: a full response
            // queue must not stall control, legitimate clients or input.
            for _ in 0..8 {
                match pair.0.send(&display::encode([0; 6])) {
                    Ok(()) => stats[4] += 1,
                    Err(Error(code)) if code == abi::WOULD_BLOCK as i64 => stats[5] += 1,
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
        let mut packet = [0; 64];
        match control.0.receive_until(&mut packet, time::now().unwrap()) {
            Ok(len) => {
                let command = display::decode(&packet[..len]).unwrap()[0];
                control.1.send(&display::encode(stats)).unwrap();
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
            Err(Error(code)) if code == abi::TIMED_OUT as i64 => (),
            Err(error) => panic!("pressure control: {error:?}"),
        }
        drop(buffer);
    }
}
