#![no_std]
//! Cathedral's initial scene, shared by ordinary startup and its display fixture.
use cathedral_contracts::{display as wire, user as abi};
use cathedral_user_runtime::{Error, ipc::Handle};

pub fn dimensions(send: Handle, receive: Handle) -> Result<(u64, u64), Error> {
    let reply = call(send, receive, [wire::INFO, 0, 0, 0, 0, 0])?;
    Ok((reply[1], reply[2]))
}
pub fn draw(send: Handle, receive: Handle, w: u64, h: u64) -> Result<(), Error> {
    if !(640..=8192).contains(&w) || !(480..=8192).contains(&h) {
        return Err(Error(abi::INVALID_ARGUMENT as i64));
    }
    call(send, receive, [wire::CLEAR, 0, 0, 0, 0, 0x101827])?;
    for (x, y, width, height, color) in [
        (w / 16, h / 12, w * 7 / 8, h / 96, 0x59d9cc),
        (w / 16, h * 5 / 24, w / 4, h / 2, 0xe96f6f),
        (w * 6 / 16, h * 5 / 24, w / 4, h / 2, 0x79c99e),
        (w * 11 / 16, h * 5 / 24, w / 4, h / 2, 0x779bea),
    ] {
        call(send, receive, [wire::RECT, x, y, width, height, color])?;
    }
    Ok(())
}
pub fn draw_interactive(
    send: Handle,
    receive: Handle,
    w: u64,
    h: u64,
    selected: u8,
    active: u8,
) -> Result<(), Error> {
    draw(send, receive, w, h)?;
    for index in 0..3 {
        let x = w * (1 + 5 * index) / 16;
        let y = h * 5 / 24;
        if active & (1 << index) != 0 {
            call(
                send,
                receive,
                [wire::RECT, x + 16, y + h / 4 - 8, w / 4 - 32, 16, 0x101827],
            )?;
        }
        if u64::from(selected) == index {
            for (rx, ry, rw, rh) in [
                (x - 4, y - 4, w / 4 + 8, 4),
                (x - 4, y + h / 2, w / 4 + 8, 4),
                (x - 4, y, 4, h / 2),
                (x + w / 4, y, 4, h / 2),
            ] {
                call(send, receive, [wire::RECT, rx, ry, rw, rh, 0xe8edf4])?;
            }
        }
    }
    Ok(())
}
fn call(send: Handle, receive: Handle, words: [u64; 6]) -> Result<[u64; 6], Error> {
    send.send(&wire::encode(words))?;
    let mut bytes = [0; 64];
    let length = receive.receive(&mut bytes)?;
    let reply = wire::decode(&bytes[..length]).ok_or(Error(abi::IO_ERROR as i64))?;
    if reply[0] != 0 {
        return Err(Error(reply[0] as i64));
    }
    Ok(reply)
}
