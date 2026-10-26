#![no_std]
//! Distribution-owned test pattern over an already established display connection.
use cathedral_kernel_contracts::user as abi;
use cathedral_platform_contracts::display as wire;
use cathedral_user_runtime::{Error, connection::Client};

pub fn draw(client: &mut Client, w: u64, h: u64) -> Result<(), Error> {
    if !(640..=8192).contains(&w) || !(480..=8192).contains(&h) {
        return Err(Error(abi::INVALID_ARGUMENT as i64));
    }
    request(client, [wire::CLEAR, 0, 0, 0, 0, 0x101827])?;
    for (x, y, width, height, color) in [
        (w / 16, h / 12, w * 7 / 8, h / 96, 0x59d9cc),
        (w / 16, h * 5 / 24, w / 4, h / 2, 0xe96f6f),
        (w * 6 / 16, h * 5 / 24, w / 4, h / 2, 0x79c99e),
        (w * 11 / 16, h * 5 / 24, w / 4, h / 2, 0x779bea),
    ] {
        request(client, [wire::RECT, x, y, width, height, color])?;
    }
    Ok(())
}

fn request(client: &mut Client, words: [u64; 6]) -> Result<(), Error> {
    let (bytes, length) = client.call_retained(&wire::encode(words))?;
    let reply = wire::decode(&bytes[..length]).ok_or(Error(abi::IO_ERROR as i64))?;
    if reply[0] != 0 {
        return Err(Error(reply[0] as i64));
    }
    Ok(())
}
