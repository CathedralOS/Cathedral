//! Acquire one owner-selected producer, select the right-hand document, then
//! type once and move a pointer. No storage, global input or framebuffer access.
use cathedral_kernel_contracts::user as abi;
use cathedral_platform_contracts::{input, matrix as wire};
use cathedral_user_runtime::{
    Error,
    matrix::upstream::{self, Pair},
    server::Server,
    services::Services,
    time,
};
fn send(pair: Pair, packet: wire::Packet) -> Result<wire::Packet, Error> {
    let reply = upstream::call(pair, packet)?;
    if reply[0] != 0 {
        return Err(Error(reply[0] as i64));
    }
    Ok(reply)
}
pub fn run(mut services: Services) -> Result<(), Error> {
    let mut server = Server::open()?;
    let pair = services.input_producer(1, 1)?;
    let info = send(pair, [wire::PRODUCER_INFO, 0, 0, 0, 0, 0, 0, 0])?;
    let generation = info[1];
    if info[2] < 80 || info[3] < 80 {
        return Err(Error(abi::DENIED as i64));
    }
    let x = info[2] * 3 / 4;
    let y = info[3] / 2;
    for buttons in [0, 1, 0] {
        send(
            pair,
            [wire::PRODUCER_POINTER, generation, 1, x, y, buttons, 0, 0],
        )?;
    }
    cathedral_user_runtime::write(b"Cathedral participant: private route selected\n")?;
    let mut step = 0usize;
    let mut deadline = time::after(50)?;
    loop {
        if let Some(event) = server.next_event_or(&[], Some(deadline))? {
            if let cathedral_user_runtime::server::Event::Request(request) = event {
                let response = match wire::decode(&request.bytes[..request.len]) {
                    Some([wire::LOSS | wire::HEALTH, 0, 0, 0, 0, 0, 0, 0]) => [0; 8],
                    _ => wire::error(abi::DENIED),
                };
                server.reply(&request, &wire::encode(response))?;
            }
            continue;
        }
        // Each letter is a captured press and release, independent of the
        // physical keyboard's current selection. After three letters, idle motion.
        if let Some(letter) = b"bot".get(step) {
            let key = u64::from(input::A + *letter - b'a');
            for state in [input::PRESS, input::RELEASE] {
                send(
                    pair,
                    [
                        wire::PRODUCER_KEY,
                        generation,
                        2,
                        key,
                        u64::from(state),
                        0,
                        0,
                        0,
                    ],
                )?;
            }
        }
        send(
            pair,
            [
                wire::PRODUCER_POINTER,
                generation,
                1,
                x + (step % 16) as u64,
                y,
                0,
                0,
                0,
            ],
        )?;
        step = step.saturating_add(1);
        deadline = time::after(50)?;
    }
}
