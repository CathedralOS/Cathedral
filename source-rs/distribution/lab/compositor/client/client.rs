// Exercise the actual IPC and shared-page path; no display or task-control grant.
use super::{exchange, scene};
use cathedral_kernel_contracts::user as abi;
use cathedral_platform_contracts::display;
use cathedral_platform_contracts::display::composition as w;
use cathedral_user_runtime::{
    Error,
    connection::Pipeline,
    link::Link,
    memory::{Private, Sealed},
    server::{Server, SharedSlot},
    time,
};
type Data = Pipeline<4>;
struct Start {
    mode: u64,
    arg: u64,
    control: Server,
    connection: Data,
    link: Link,
}
pub fn run(role: u64) -> u64 {
    let start = commands(role);
    if start.mode == 14 {
        session::run(role, start.control, start.connection, start.link)
    } else {
        interactive::run(role, start.arg, start.control, start.connection, start.link)
    }
}
fn commands(role: u64) -> Start {
    assert_eq!(
        cathedral_user_runtime::display::mapping(),
        Err(Error(abi::DENIED as i64))
    );
    static mut CONTROL: SharedSlot = SharedSlot::empty();
    let slot = &raw mut CONTROL;
    let mut control = Server::open_clients(0).unwrap();
    // SAFETY: Single-entry app exclusively lends its control slot to this server.
    control
        .enable_shared_control(unsafe { &mut *slot })
        .unwrap();
    let link = Link::at(0).unwrap();
    let mut pair = None;
    let mut offer: Option<Sealed> = None;
    loop {
        let request = control.next_request().unwrap();
        let [op, arg, extra, _, _, _] = display::decode(&request.bytes[..request.len]).unwrap();
        if op == 9 {
            pair = None;
        }
        if pair.is_none() {
            pair = Some(Data::connect(link.connect().unwrap(), time::after(300).unwrap()).unwrap());
        }
        let connection = pair.as_mut().unwrap();
        let reply = match op {
            1..=3 => {
                let snapshot = scene::snapshot(role, op == 2);
                let mut page = Private::allocate(1).unwrap();
                page.bytes_mut()[..snapshot.len()].copy_from_slice(&snapshot);
                let sealed = page.seal(link).unwrap();
                let handle = sealed.handle();
                let result = exchange(connection, [w::SUBMIT, arg, handle, 0, 0, 0]);
                sealed.release().unwrap();
                [result[0], handle, 0, 0, 0, 0]
            }
            4 => exchange(connection, [w::SUBMIT, arg, extra, 0, 0, 0]),
            5 => {
                let snapshot = scene::snapshot(role, false);
                let mut page = Private::allocate(1).unwrap();
                page.bytes_mut()[..snapshot.len()].copy_from_slice(&snapshot);
                let sealed = page.seal(link).unwrap();
                let handle = sealed.handle();
                offer = Some(sealed);
                [0, handle, 0, 0, 0, 0]
            }
            6 => {
                let sealed = offer.take().unwrap();
                let result = exchange(connection, [w::SUBMIT, arg, sealed.handle(), 0, 0, 0]);
                sealed.release().unwrap();
                result
            }
            7 => {
                control.reply(&request, &display::encode([0; 6])).unwrap();
                loop {
                    core::hint::spin_loop();
                }
            }
            8 => {
                // Fill the independent-call window without consuming any replies.
                // The provider must retain replies behind its full output mailbox
                // while continuing to serve control and other clients.
                let packet = display::encode([w::BACKGROUND, 0, 0, 0, 0, 0]);
                let mut sent = 0;
                while sent < 4 {
                    match connection.submit(&packet, None) {
                        Ok(_) => sent += 1,
                        Err(Error(code)) if code == abi::WOULD_BLOCK as i64 => {
                            cathedral_user_runtime::yield_now()
                        }
                        Err(error) => panic!("flood setup: {error:?}"),
                    }
                }
                assert_eq!(
                    connection.submit(&packet, None),
                    Err(Error(abi::BUSY as i64))
                );
                control.reply(&request, &display::encode([0; 6])).unwrap();
                loop {
                    assert_eq!(
                        connection.submit(&packet, None),
                        Err(Error(abi::BUSY as i64))
                    );
                }
            }
            9 => [0; 6],
            10 => {
                // SAFETY: Explicit lab-only invalid-opcode fault; kernel contains this client.
                unsafe {
                    core::arch::asm!("ud2", options(noreturn));
                }
            }
            11 => exchange(connection, [w::PLACE, 1, 0, 0, 256, 256]),
            12 | 14 => {
                control.reply(&request, &display::encode([0; 6])).unwrap();
                return Start {
                    mode: op,
                    arg,
                    control,
                    connection: pair.take().unwrap(),
                    link,
                };
            }
            32..=57 => exchange(
                connection,
                display::decode(&request.bytes[..request.len]).unwrap(),
            ),
            _ => panic!(),
        };
        control.reply(&request, &display::encode(reply)).unwrap();
    }
}
