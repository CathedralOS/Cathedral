//! One wait for IPC, IRQ/notification readiness and a deadline. Observing a ready
//! interest does not consume its message, IRQ or notification indication.
use crate::{Error, abi, arch, device::Device, ipc::Handle};
#[derive(Clone, Copy)]
pub enum Interest {
    Ipc(Handle),
    Device(Device),
    Notification(crate::notification::Notification),
}

pub fn wait(interests: &[Interest], deadline: u64) -> Result<usize, Error> {
    if interests.len() > abi::MAX_WAIT_HANDLES {
        return Err(Error(abi::INVALID_ARGUMENT as i64));
    }
    // The kernel copies a wait record set within one page. Align the entire
    // maximum-size set so a larger live wait cannot straddle a page boundary.
    #[repr(align(1024))]
    struct Records([u64; abi::MAX_WAIT_HANDLES * 2]);
    const _: () = assert!(
        core::mem::size_of::<[u64; abi::MAX_WAIT_HANDLES * 2]>()
            <= core::mem::align_of::<Records>()
    );
    let mut records = Records([0; abi::MAX_WAIT_HANDLES * 2]);
    for (interest, words) in interests.iter().zip(records.0.chunks_exact_mut(2)) {
        let (kind, handle) = match *interest {
            Interest::Ipc(h) => (abi::WAIT_IPC, h.raw()),
            Interest::Device(d) => (abi::WAIT_DEVICE, d.raw()),
            Interest::Notification(n) => (abi::WAIT_NOTIFICATION, n.raw()),
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
