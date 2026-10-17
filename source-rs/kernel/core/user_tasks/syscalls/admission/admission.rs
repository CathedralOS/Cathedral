// Resolve executable authority, reserve owned capacity, then publish a fresh launch.
// Mapping still happens through lifecycle::spawn; failed loads keep the reservation retryable.
use super::{Session, abi};
use crate::user_tasks::admission::pool::Code;
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
        abi::TASK_POOL_ASSIGN
            | abi::TASK_IMAGE_ALLOW
            | abi::TASK_CPU_ASSIGN
            | abi::TASK_IMAGE_FROM_OBJECT
    ) && second != 0
    {
        return Err(abi::INVALID_ARGUMENT);
    }
    if operation == abi::TASK_IMAGE_FROM_OBJECT {
        // Code from a sealed object the caller can read. The kernel's own
        // reference outlives the caller's handle until admission or release.
        let region = session.memory.model.access(caller, first)?;
        if !session.memory.model.entries[region].sealed {
            return Err(abi::DENIED);
        }
        let name = session.names.mint(caller)?;
        session.executables.install(
            caller,
            Code::Object {
                region,
                role: second,
            },
            name,
        )?;
        session.memory.hold(region);
        return Ok(name);
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
            // Only a boot template can be exported; object code is single use.
            let Some(Code::Template(template)) = launch.code else {
                return Err(abi::DENIED);
            };
            if let Some(name) = session.executables.find(caller, template) {
                return Ok(name);
            }
            let name = session.names.mint(caller)?;
            session
                .executables
                .install(caller, Code::Template(template), name)
        }
        abi::TASK_IMAGE_ALLOW => {
            if !launch.model.idle() {
                return Err(abi::BUSY);
            }
            let Code::Template(template) = session.executables.resolve(caller, second)? else {
                return Err(abi::DENIED);
            };
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
            crate::user_tasks::resources::assign(
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
            for region in session.executables.close(child).into_iter().flatten() {
                session.memory.unref_later(region);
            }
            if let Some(Code::Object { region, .. }) = session.launches[index].code {
                session.memory.unref_later(region);
            }
            session.launches[index].code = None;
            session.launches[index].images.fill(None);
            session.launches[index].model =
                crate::user_tasks::lifecycle::supervision::Supervisor::vacant(child);
            Ok(0)
        }
        _ => Err(abi::UNKNOWN),
    }
}
