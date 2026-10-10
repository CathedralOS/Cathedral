//! Init's application-facing status/readiness protocol; fault controls are test-only.
#[cfg(feature = "recovery-lab")]
use super::super::{child::Service, recovery::Probes};
use cathedral_session_protocol as session;
use cathedral_user_runtime::Error;
pub(super) fn response<'a>(
    request: &[u8],
    encoded: &'a [u8],
    #[cfg(feature = "recovery-lab")] services: [&Service; 3],
    #[cfg(feature = "recovery-lab")] app: &Service,
    #[cfg(feature = "recovery-lab")] probes: &mut Probes,
) -> Result<&'a [u8], Error> {
    Ok(match request {
        session::STATUS => encoded,
        session::READY => b"ok",
        #[cfg(feature = "recovery-lab")]
        [0xf0, index @ 0..=2] => {
            probes.inject(*index as usize, services, app)?;
            b"ok"
        }
        #[cfg(feature = "recovery-lab")]
        [0xf1, phase @ 1..=4] => {
            services[2].send.send(&[0xf1, *phase])?;
            let (reply, len) = services[2].receive()?;
            assert_eq!(&reply[..len], b"armed");
            b"armed"
        }
        _ => b"denied",
    })
}
