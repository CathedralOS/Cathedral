//! Fund the root and move presentation/source endpoints only on successful spawn.
use super::*;
pub(super) fn start(
    attempt: &mut Attempt,
    display: Pair,
    argument: u64,
) -> Result<(Pair, Pair), Error> {
    let (source_send, source_input) = attempt.channel()?;
    let (source_output, source_recv) = attempt.channel()?;
    funding::assign(Launch::at(2)?)?;
    attempt.spawn(
        Stage::Root,
        argument,
        &[display.0, display.1, source_input, source_output],
    )?;
    let matrix = attempt.track(Port::at(2)?.connect()?);
    matrix_ok(matrix, [wire::HEALTH, 0, 0, 0, 0, 0, 0, 0])?;
    assert_eq!(task::available_slots()?, 0);
    Ok((matrix, (source_send, source_recv)))
}

#[cfg(feature = "shell")]
pub(super) fn configure(session: &Session) -> Result<(), Error> {
    let [width, height] = session.extent;
    matrix_ok(
        session.matrix,
        [wire::BIND, width, height, KEYS, 0, 0, 0, 0],
    )?;
    for words in shell::workspace(width, height) {
        matrix_ok(session.matrix, words)?;
    }
    Ok(())
}
