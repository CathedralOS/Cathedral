// Route only boot-granted device operations; each adapter checks the caller's grant.
use super::Session;
use crate::scheduler::Event;
use cathedral_kernel_contracts::user as abi;
pub(super) fn dispatch(
    session: &mut Session,
    slot: usize,
    [number, first, second, third]: [u64; 4],
    now: u64,
    event: &mut Event,
) -> Result<u64, u64> {
    match number {
        abi::DEVICE_HANDLE..=abi::DEVICE_RELEASE => {
            leased::dispatch(session, slot, [number, first, second, third], event)
        }
        abi::INPUT_VIRTUAL_POINTER => {
            keyboard::virtual_pointer(session, slot, first, second, third)
        }
        abi::KEYBOARD_READ | abi::KEYBOARD_WRITE | abi::KEYBOARD_READ_UNTIL => {
            keyboard::syscall(session, slot, (number, first, second), event, now)
        }
        abi::DISK_REGISTER_READ..=abi::DISK_TRANSFER => {
            disk::syscall(session, slot, (number, first, second, third))
        }
        abi::DISPLAY_INFO => display::information(session, slot, first, second),
        _ => Err(abi::UNKNOWN),
    }
}

fn error(error: cathedral_arch::board::DeviceError) -> u64 {
    match error {
        cathedral_arch::board::DeviceError::InvalidArgument => abi::INVALID_ARGUMENT,
        cathedral_arch::board::DeviceError::WouldBlock => abi::WOULD_BLOCK,
    }
}
