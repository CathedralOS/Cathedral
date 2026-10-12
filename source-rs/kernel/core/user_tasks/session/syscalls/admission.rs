//! Resolve executable authority, reserve owned capacity, then publish a fresh launch.
//! Mapping still happens through lifecycle::spawn; failed loads keep the reservation retryable.
mod reserve;
use super::{Session, abi};
pub(super) fn dispatch(
    session: &mut Session,
    caller: usize,
    [operation, first, second, third]: [u64; 4],
) -> Result<u64, u64> {
    if third != 0 {
        return Err(abi::INVALID_ARGUMENT);
    }
    if operation == abi::TASK_RESERVE {
        return reserve::reserve(session, caller, first, second);
    }
    if !matches!(
        operation,
        abi::TASK_POOL_ASSIGN | abi::TASK_IMAGE_ALLOW | abi::TASK_CPU_ASSIGN
    ) && second != 0
    {
        return Err(abi::INVALID_ARGUMENT);
    }
    if operation == abi::TASK_IMAGE_AT {
        return session
            .executables
            .at(caller, usize::try_from(first).map_err(|_| abi::BAD_HANDLE)?);
    }
    if operation == abi::TASK_POOL_AVAILABLE {
        return if first == 0 {
            Ok(session.pool.available(caller) as u64)
        } else {
            Err(abi::INVALID_ARGUMENT)
        };
    }
    let index = session
        .launches
        .iter()
        .position(|l| l.model.launch(caller) == Ok(first))
        .ok_or(abi::BAD_HANDLE)?;
    let launch = &session.launches[index];
    let child = launch.model.child;
    match operation {
        abi::TASK_EXECUTABLE => {
            if launch.keyboard || launch.disk || launch.framebuffer.is_some() {
                return Err(abi::DENIED);
            }
            let template = launch.template.ok_or(abi::BAD_HANDLE)?;
            if let Some(name) = session.executables.find(caller, template) {
                return Ok(name);
            }
            let name = session.names.mint(caller)?;
            session.executables.install(caller, template, name)
        }
        abi::TASK_IMAGE_ALLOW => {
            if !launch.model.idle() {
                return Err(abi::BUSY);
            }
            let template = session.executables.resolve(caller, second)?;
            let images = &mut session.launches[index].images;
            if !images.contains(&Some(template)) {
                *images
                    .iter_mut()
                    .find(|i| i.is_none())
                    .ok_or(abi::NO_MEMORY)? = Some(template);
            }
            Ok(0)
        }
        abi::TASK_CPU_ASSIGN => {
            if !launch.model.idle() {
                return Err(abi::BUSY);
            }
            session.scheduler.account_cpu(cathedral_arch::cpu_time_us());
            if !session.scheduler.assign_cpu(child, second) {
                return Err(abi::INVALID_ARGUMENT);
            }
            Ok(0)
        }
        abi::TASK_LAUNCH_PORT => launch.model.port(caller),
        abi::TASK_POOL_ASSIGN => {
            if !launch.model.idle() {
                return Err(abi::BUSY);
            }
            let count = usize::try_from(second).map_err(|_| abi::INVALID_ARGUMENT)?;
            session.pool.assign(caller, child, count)?;
            Ok(0)
        }
        abi::TASK_RELEASE => {
            if !launch.dynamic {
                return Err(abi::DENIED);
            }
            if !launch.model.idle() {
                return Err(abi::BUSY);
            }
            session.memory.funding.check_assign(caller, child, 0)?;
            crate::resources::assign(
                &mut session.ipc,
                &mut session.memory.model,
                caller,
                child,
                [0; 3],
            )?;
            session
                .memory
                .funding
                .assign(caller, child, 0)
                .expect("validated physical refund");
            session.pool.release(caller, child);
            session.clock.allow(child, false);
            session.executables.close(child);
            session.launches[index].template = None;
            session.launches[index].images.fill(None);
            session.launches[index].model = crate::supervision::Supervisor::vacant(child);
            Ok(0)
        }
        _ => Err(abi::UNKNOWN),
    }
}
