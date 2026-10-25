//! Disposable fault variants. No test control or extra authority is offered by
//! Split: the fixture uses the same acquired production pair as the normal app.
use crate::participant::send;
use cathedral_kernel_contracts::user as abi;
use cathedral_platform_contracts::{input, matrix as wire};
use cathedral_user_runtime::{Error, connection::Client, event, server::Server, time, write};
fn delay(ticks: u64) -> Result<(), Error> {
    match event::wait(&[], time::after(ticks)?) {
        Err(Error(code)) if code == abi::TIMED_OUT as i64 => Ok(()),
        result => Err(result.err().unwrap_or(Error(abi::IO_ERROR as i64))),
    }
}
pub fn run(mut server: Server, mut client: Client, info: wire::Packet) -> Result<(), Error> {
    let generation = info[1];
    time::storage_sample("producer-generation", &[generation]);
    let x = info[2] * 3 / 4;
    let y = info[3] / 2;
    for buttons in [0, 1] {
        send(
            &mut client,
            [wire::PRODUCER_POINTER, generation, 1, x, y, buttons, 0, 0],
        )?;
    }
    if cfg!(feature = "pressure-probe") {
        send(
            &mut client,
            [
                wire::PRODUCER_KEY,
                generation,
                2,
                input::F8 as u64,
                input::PRESS as u64,
                0,
                0,
                0,
            ],
        )?;
        delay(100)?;
        write(b"Cathedral participant probe: flooding one channel\n")?;
        for _ in 0..64 {
            send(
                &mut client,
                [
                    wire::PRODUCER_KEY,
                    generation,
                    2,
                    input::F8 as u64,
                    input::REPEAT as u64,
                    0,
                    0,
                    0,
                ],
            )?;
        }
        send(
            &mut client,
            [
                wire::PRODUCER_KEY,
                generation,
                2,
                input::F8 as u64,
                input::RELEASE as u64,
                0,
                0,
                0,
            ],
        )?;
        write(b"Cathedral participant probe: flood complete\n")?;
        delay(300)?;
        for state in [input::PRESS, input::RELEASE] {
            send(
                &mut client,
                [
                    wire::PRODUCER_KEY,
                    generation,
                    2,
                    input::A as u64,
                    state as u64,
                    0,
                    0,
                    0,
                ],
            )?;
        }
        send(
            &mut client,
            [wire::PRODUCER_POINTER, generation, 1, x, y, 0, 0, 0],
        )?;
        write(b"Cathedral participant probe: fresh input after pressure\n")?;
    } else {
        send(
            &mut client,
            [
                wire::PRODUCER_KEY,
                generation,
                2,
                input::A as u64,
                input::PRESS as u64,
                0,
                0,
                0,
            ],
        )?;
        write(b"Cathedral participant probe: holding pointer and key\n")?;
        // Enough time for a real consumer to observe the hold. The host also
        // requires its trace before accepting the retirement proof.
        if cfg!(feature = "disconnect-probe") {
            delay(150)?;
            client.retire();
            write(b"Cathedral participant probe: producer endpoint closed\n")?;
            return Ok(());
        }
    }
    loop {
        let request = server.next_request()?;
        let response = match wire::decode(&request.bytes[..request.len]) {
            Some([wire::HEALTH | wire::LOSS, 0, 0, 0, 0, 0, 0, 0]) => [0; 8],
            _ => wire::error(abi::DENIED),
        };
        server.reply(&request, &wire::encode(response))?;
    }
}
