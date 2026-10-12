//! Status application entrance: restore -> receive -> update -> save -> render.
//! connection follows init health; input/storage/view follow the three platform peers.
mod connection;
mod input;
#[cfg(feature = "recovery-lab")]
mod lab;
mod state;
mod storage;
mod view;
use cathedral_session_protocol as session;
use cathedral_user_runtime::Error;
use connection::Connections;
use state::State;
pub fn run(_generation: u64) -> Result<(), Error> {
    let mut connections = Connections::open()?;
    connections.refresh()?;
    #[cfg(feature = "recovery-lab")]
    lab::authority(&mut connections)?;
    let mut state = State::restore(storage::load(&mut connections)?)?;
    state.records = storage::catalog(&mut connections)?;
    view::render(&state, &mut connections)?;
    connections.request(session::READY)?;
    loop {
        let event = input::next(&mut connections)?;
        let redraw = connections.refresh()?;
        let changed = state.update(event);
        #[cfg(feature = "recovery-lab")]
        let redraw = redraw | lab::handle(event, _generation, &mut connections)?;
        if changed {
            state.saved =
                storage::save(&mut connections, state.saved, state.selected, state.active)?;
            state.records = storage::catalog(&mut connections)?;
        }
        if changed || redraw {
            view::render(&state, &mut connections)?;
        }
    }
}
