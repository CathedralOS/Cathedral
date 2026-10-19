// Acquire the enclosing world's services, admit two children, then route events.
// Children receive only interaction; this matrix uses storage solely to load code.
use cathedral_kernel_contracts::user as abi;
use cathedral_matrix::world::{Grant, Rect, World};
use cathedral_platform_contracts::{imports, matrix as wire, services};
use cathedral_user_runtime::{
    Error,
    matrix::upstream::{self, Pair},
    server::{Event, Server},
    services::Services,
    time,
};
use children::Children;
use rendering::Output;

pub fn run(mut services: Services) -> Result<(), Error> {
    let interaction = services.interaction(1, 1)?;
    let storage = services.storage(1, 1)?;
    let mut server = Server::open_clients(2)?;
    server.enable_memory();
    static mut WORLD: World = World::empty();
    let arena = &raw mut WORLD;
    // SAFETY: Single-threaded process entry exclusively owns this arena.
    let world = unsafe { &mut *arena };
    let mut output = Output::new(interaction.pair);
    let mut children = Children::empty();
    children.start(storage, &mut server)?;
    let mut survey: Option<loss::Survey> = None;
    let mut button = false;
    let mut refresh = true;
    loop {
        // Consume the wake before polling. A notification racing the poll stays
        // queued, so sleeping below cannot lose it.
        match interaction.wake.receive_until(&mut [0; 64], time::now()?) {
            Ok(_) => refresh = true,
            Err(Error(code)) if code == abi::TIMED_OUT as i64 => (),
            Err(e) => return Err(e),
        }
        for _ in 0..8 {
            if !refresh {
                break;
            }
            let snapshot = upstream::call(interaction.pair, [wire::POLL, 0, 0, 0, 0, 0, 0, 0])?;
            let grant = world.inherit_poll(snapshot);
            output.arrange(grant, world)?;
            let event = wire::poll_event(snapshot);
            if event[2] == wire::POINTER {
                let pressed = event[5] & 1 != 0;
                if pressed
                    && !button
                    && event[4] < 28
                    && event[3] >= grant.width.saturating_sub(28)
                    && survey.is_none()
                {
                    survey = Some(loss::Survey::start(None, &mut children)?);
                }
                button = pressed;
            } else if event[2] == wire::CANCEL {
                button = false;
            }
            refresh = snapshot[5] & wire::MORE != 0;
        }
        output.flush(world)?;
        children.notify(world)?;
        if let Some(s) = &mut survey
            && let Some(clean) = s.progress(&mut children)?
        {
            if let Some(request) = s.request {
                server.reply(
                    &request,
                    &wire::encode([0, u64::from(!clean), 0, 0, 0, 0, 0, 0]),
                )?;
            } else if clean {
                return Ok(());
            } else {
                output.header(true)?;
            }
            survey = None;
        }
        if refresh {
            continue;
        }
        let mut wake = [interaction.wake; 3];
        let mut count = 1;
        if let Some(s) = &survey {
            for slot in 0..2 {
                if s.waiting & (1 << slot) != 0 {
                    wake[count] = children.slots[slot].as_ref().unwrap().control.1;
                    count += 1;
                }
            }
        }
        server.pause(u8::from(
            survey.as_ref().is_some_and(|s| s.request.is_some()),
        ));
        let Some(event) =
            server.next_event_or(&wake[..count], survey.as_ref().map(|s| s.deadline))?
        else {
            continue;
        };
        requests::dispatch(
            event,
            &mut output,
            world,
            &mut children,
            &mut server,
            &mut survey,
        )?;
    }
}
