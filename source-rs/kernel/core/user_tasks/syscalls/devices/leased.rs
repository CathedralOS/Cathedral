//! Register/IRQ syscall entrance: authenticate, validate, then touch the selected board.
//! Lifecycle resets the device before installing authority; close masks/resets before reuse.
use crate::scheduler::Event;
use crate::user_tasks::{Session, syscalls::complete};
use cathedral_arch::board::DeviceId;
use cathedral_kernel_contracts::user as abi;

fn device(session: &Session, slot: usize) -> Option<DeviceId> {
    session
        .launches
        .iter()
        .find(|l| l.model.child == slot)
        .and_then(|l| l.device)
}
pub(super) fn dispatch(
    session: &mut Session,
    slot: usize,
    [number, name, second, third]: [u64; 4],
    event: &mut Event,
) -> Result<u64, u64> {
    if number == abi::DEVICE_HANDLE {
        if second != 0 || third != 0 {
            return Err(abi::INVALID_ARGUMENT);
        }
        return session.devices.handle(slot, name);
    }
    session.devices.check(slot, name)?;
    let device = device(session, slot).ok_or(abi::BAD_HANDLE)?;
    match number {
        abi::DEVICE_READ8 | abi::DEVICE_WRITE8 => {
            session.devices.register(slot, name, second)?;
            if (number == abi::DEVICE_READ8 && third != 0) || third > 255 {
                return Err(abi::INVALID_ARGUMENT);
            }
            // SAFETY: Authenticated exclusive grant, bounded offset/value, IRQs disabled.
            unsafe {
                if number == abi::DEVICE_READ8 {
                    device.read(second).map(u64::from)
                } else {
                    device.write(second, third as u8).map(|()| 0)
                }
            }
            .map_err(super::error)
        }
        abi::DEVICE_IRQ_WAIT => {
            if second > 1 || third != 0 {
                return Err(abi::INVALID_ARGUMENT);
            }
            if session.devices.wait(slot, name, second == 1)? {
                session.tasks[slot].report.device_waits_blocked += 1;
                *event = Event::Block;
            }
            Ok(0)
        }
        abi::DEVICE_IRQ_ARM => {
            if second != 0 || third != 0 {
                return Err(abi::INVALID_ARGUMENT);
            }
            session.devices.arm(slot, name)?;
            // SAFETY: Installed dedicated vector, exclusive device, committed live arm.
            unsafe {
                device.arm();
            }
            Ok(0)
        }
        abi::DEVICE_RELEASE => {
            if second != 0 || third != 0 {
                return Err(abi::INVALID_ARGUMENT);
            }
            close(session, slot);
            Ok(0)
        }
        _ => Err(abi::UNKNOWN),
    }
}

pub(in crate::user_tasks) fn interrupts(session: &mut Session) {
    for slot in 0..session.tasks.len() {
        let Some(device) = device(session, slot) else {
            continue;
        };
        if !device.take_interrupt() {
            continue;
        }
        if session.devices.handle(slot, 0).is_ok() {
            session.tasks[slot].report.device_interrupts += 1;
        }
        if session.devices.interrupt(slot) {
            complete(session, slot, Ok(0));
            session.scheduler.unblock(slot);
        }
    }
}

pub(in crate::user_tasks) fn close(session: &mut Session, slot: usize) {
    if session.devices.close(slot) {
        let device = device(session, slot).expect("live device grant lost its board binding");
        // SAFETY: IRQs disabled, task cannot run; reset before admission can reuse the slot.
        unsafe {
            device.reset();
        }
    }
}
