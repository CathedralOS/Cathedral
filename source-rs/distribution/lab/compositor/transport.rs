//! Lab wire exchange over retained shared connections; ownership stays with callers.
use cathedral_kernel_contracts::user as abi;
use cathedral_platform_contracts::display;
use cathedral_user_runtime::{Error, connection::Pipeline, event, time};

pub fn exchange<const N: usize>(client: &mut Pipeline<N>, words: [u64; 6]) -> [u64; 6] {
    try_exchange(client, words).unwrap_or_else(|error| panic!("exchange {words:?}: {error:?}"))
}

pub fn try_exchange<const N: usize>(
    client: &mut Pipeline<N>,
    words: [u64; 6],
) -> Result<[u64; 6], Error> {
    let (bytes, len) = client.call(&display::encode(words), time::after(300)?)?;
    display::decode(&bytes[..len]).ok_or(Error(abi::IO_ERROR as i64))
}

pub fn finish<const N: usize>(client: &mut Pipeline<N>, deadline: u64) -> Result<[u64; 6], Error> {
    let result = (|| loop {
        if let Some((bytes, len, object)) = client.poll()? {
            if object.is_some() {
                return Err(Error(abi::IO_ERROR as i64));
            }
            return display::decode(&bytes[..len]).ok_or(Error(abi::IO_ERROR as i64));
        }
        event::wait(&client.interests()?, deadline)?;
    })();
    if result.is_err() {
        client.retire();
    }
    result
}

pub fn idle(ticks: u64) {
    assert_eq!(
        event::wait(&[], time::after(ticks).unwrap()),
        Err(Error(abi::TIMED_OUT as i64))
    );
}
