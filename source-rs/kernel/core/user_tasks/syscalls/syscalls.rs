// Syscall entrance: route by authority/resource family, then complete and wake.
// Platform APIs reach this transport through platform/libraries/user-runtime/;
// display/input/storage request semantics remain in their service executables.
use super::Session;
use crate::scheduler::Event;
use cathedral_kernel_contracts::user as abi;
pub(super) fn syscall(session: &mut Session, slot: usize, now: u64) -> Event {
    let (number, first, second, third) = session.tasks[slot].context.syscall();
    let mut event = Event::Continue;
    let call = [number, first, second, third];
    let result = match number {
        abi::TASK_EXECUTABLE..=abi::TASK_CPU_ASSIGN
        | abi::TASK_IMAGE_FROM_OBJECT
        | abi::TASK_IMAGE_RELEASE => admission::dispatch(session, slot, call),
        abi::RESOURCE_BUDGET
        | abi::RESOURCE_GROW
        | abi::RESOURCE_ASSIGN
        | abi::PHYSICAL_BUDGET
        | abi::PHYSICAL_ASSIGN => resources::dispatch(session, slot, call),
        abi::WRITE => diagnostics::write(session, slot, first, second),
        abi::IPC_HANDLE..=abi::IPC_REVOKE
        | abi::LINK_HANDLE
        | abi::LINK_CONNECT
        | abi::IPC_RECEIVE_UNTIL
        | abi::IPC_WAIT_TWO
        | abi::IPC_CREATE..=abi::IPC_RECEIVE_CAP
        | abi::IPC_WAIT_MANY
        | abi::EVENT_WAIT
        | abi::IPC_BUDGET
        | abi::IPC_SEND_MEMORY..=abi::IPC_TRY_RECEIVE_MEMORY => {
            ipc::dispatch(session, slot, call, now, &mut event)
        }
        abi::DEVICE_HANDLE..=abi::DEVICE_RELEASE
        | abi::KEYBOARD_READ
        | abi::INPUT_VIRTUAL_POINTER
        | abi::KEYBOARD_WRITE
        | abi::KEYBOARD_READ_UNTIL
        | abi::DISK_REGISTER_READ..=abi::DISK_TRANSFER
        | abi::DISPLAY_INFO => devices::dispatch(session, slot, call, now, &mut event),
        abi::CLOCK_HANDLE | abi::CLOCK_READ | abi::CPU_READ => time::read(session, slot, call, now),
        abi::YIELD
        | abi::EXIT
        | abi::TASK_LAUNCH..=abi::TASK_WAIT_UNTIL
        | abi::TASK_SPAWN_WITH
        | abi::TASK_ENVIRONMENT
        | abi::TASK_SPAWN_TREE => tasks::dispatch(
            session,
            slot,
            (number, first, second, third),
            &mut event,
            now,
        ),
        abi::MEMORY_ALLOCATE..=abi::MEMORY_PAGES
        | abi::MEMORY_FREEZE..=abi::MEMORY_UNMAP_OBJECT => {
            memory::syscall(session, slot, call, &mut event)
        }
        _ => Err(abi::UNKNOWN),
    };
    complete(session, slot, result);
    super::execution::waits::wake(session, now);
    event
}

pub(super) fn complete(session: &mut Session, slot: usize, result: Result<u64, u64>) {
    let task = &mut session.tasks[slot];
    if result.is_err() {
        task.report.rejected += 1;
    }
    if result == Err(abi::TIMED_OUT) {
        task.report.wait_timeouts += 1;
    }
    task.context
        .set_result(result.unwrap_or_else(|error| error));
}
