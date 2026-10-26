//! Startup-only owner control. Called after source validation and initialization.
//! Flush readiness until the owner closes, then release all bootstrap custody;
//! the independent source connection continues without a control-polling loop.
use cathedral_kernel_contracts::user as abi;
use cathedral_platform_contracts::input;
use cathedral_user_runtime::{
    Error,
    ipc::Handle,
    server::{Event, Server, SharedSlot},
    time,
};

pub fn serve() -> Result<(), Error> {
    let bootstrap = (Handle::bootstrap(0)?, Handle::bootstrap(1)?);
    let mut server = Server::open_clients(0)?;
    server.enable_memory();
    static mut CONTROL: SharedSlot = SharedSlot::empty();
    let slot = &raw mut CONTROL;
    // SAFETY: This process enters startup once and lends the slot until drop.
    server.enable_shared_control(unsafe { &mut *slot })?;
    let deadline = time::after(300)?;
    loop {
        match server.next_event_for(&[], Some(deadline)) {
            Ok(Some(Event::Request(request))) => {
                let reply = if request.bytes[..request.len] == input::HEALTH {
                    input::Event::idle().encode()
                } else {
                    [0xff, 0]
                };
                server.reply(&request, &reply)?;
            }
            Ok(Some(Event::Memory(request, object))) => {
                drop(object);
                server.reply(&request, &[0xff, 0])?;
            }
            Err(Error(code)) if code == abi::PEER_CLOSED as i64 => break,
            // The owner cannot hold physical delivery behind a startup stream.
            Ok(None) => break,
            Err(Error(code)) if code == abi::TIMED_OUT as i64 => break,
            Err(error) => return Err(error),
            Ok(Some(Event::Disconnected { .. })) => unreachable!(),
        }
    }
    drop(server);
    // Shared custody already closes these once setup starts. Close only any
    // still-live raw endpoints if the owner never initiated setup; no allocation
    // occurs between dropping that custody and this best-effort cleanup.
    bootstrap.0.close().ok();
    bootstrap.1.close().ok();
    cathedral_user_runtime::write(b"Cathedral input: startup control retired\n")?;
    Ok(())
}
