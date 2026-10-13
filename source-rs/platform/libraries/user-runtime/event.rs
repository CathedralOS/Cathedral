//! One wait for driver control, IRQ readiness and a deadline. The selected event
//! remains pending until the caller receives the message or consumes the IRQ.
use crate::{Error, abi, arch, device::Device, ipc::Handle};
#[derive(Clone, Copy)]
pub enum Interest {
    Ipc(Handle),
    Device(Device),
}

pub fn wait(interests: &[Interest], deadline: u64) -> Result<usize, Error> {
    if interests.len() > abi::MAX_WAIT_HANDLES {
        return Err(Error(abi::INVALID_ARGUMENT as i64));
    }
    #[repr(align(128))]
    struct Records([u64; abi::MAX_WAIT_HANDLES * 2]);
    let mut records = Records([0; abi::MAX_WAIT_HANDLES * 2]);
    for (interest, words) in interests.iter().zip(records.0.chunks_exact_mut(2)) {
        let (kind, handle) = match *interest {
            Interest::Ipc(h) => (abi::WAIT_IPC, h.raw()),
            Interest::Device(d) => (abi::WAIT_DEVICE, d.raw()),
        };
        words[0] = kind.to_le();
        words[1] = handle.to_le();
    }
    let result = arch::call3(
        abi::EVENT_WAIT,
        records.0.as_ptr() as u64,
        interests.len() as u64,
        deadline,
    );
    if (result as i64) < 0 {
        Err(Error(result as i64))
    } else {
        Ok(result as usize)
    }
}
