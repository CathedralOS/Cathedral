//! Fund the root and move presentation/source endpoints only on successful spawn.
use super::*;
pub(super) fn start(
    attempt: &mut Attempt,
    display: Pair,
    argument: u64,
) -> Result<(RootControl, Pair), Error> {
    let (source_send, source_input) = attempt.channel()?;
    let (source_output, source_recv) = attempt.channel()?;
    funding::assign(Launch::at(2)?)?;
    attempt.spawn(
        Stage::Root,
        argument,
        &[display.0, display.1, source_input, source_output],
    )?;
    let deadline = time::after(READY_TICKS)?;
    let pair = Port::at(2)?.connect()?;
    let matrix = RootControl::open(pair, deadline)?;
    assert_eq!(task::available_slots()?, 0);
    Ok((matrix, (source_send, source_recv)))
}

#[cfg(feature = "shell")]
pub(super) fn configure(session: &mut Session) -> Result<(), Error> {
    let [width, height] = session.extent;
    root_ok(
        &mut session.matrix,
        [wire::BIND, width, height, KEYS, 0, 0, 0, 0],
    )?;
    for words in shell::workspace(width, height) {
        root_ok(&mut session.matrix, words)?;
    }
    Ok(())
}

#[cfg(feature = "shell")]
fn root_ok(matrix: &mut RootControl, words: wire::Packet) -> Result<(), Error> {
    let reply = matrix.exchange(words, time::after(READY_TICKS)?)?;
    if reply == [0; 8] {
        Ok(())
    } else {
        Err(if reply[0] == 0 {
            BAD_REPLY
        } else {
            Error(reply[0] as i64)
        })
    }
}
