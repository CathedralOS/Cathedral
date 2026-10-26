//! Admit physical input only after the matrix accepts its source. Readiness is
//! bounded and exact; a failed driver is collected before teardown of its root.
use super::*;
impl Session {
    pub(super) fn input(&mut self, number: u64) -> Result<(), Error> {
        assert!(self.input.is_none());
        let value = argument(
            Stage::Input,
            number,
            self.extent[0] | (self.extent[1] << 32),
        );
        let deadline = time::after(READY_TICKS)?;
        #[cfg(feature = "bootstrap-probes")]
        probes::refusal(value)?;
        // Only an untouched pair can cross this process boundary. A lab that
        // established shared transport must retire its tree instead of dropping
        // the client state and handing the driver's new client stale endpoints.
        let (send, receive) = self.source.ok_or(Error(abi::DENIED as i64))?;
        self.input = Some(Launch::at(1)?.spawn_with(value, &[send, receive])?);
        self.source = None;
        let mut control =
            cathedral_user_runtime::connection::Client::connect(Port::at(1)?.connect()?, deadline)?;
        let (bytes, n) = control.call(&cathedral_platform_contracts::input::HEALTH, deadline)?;
        if bytes[..n] != cathedral_platform_contracts::input::Event::idle().encode() {
            return Err(BAD_REPLY);
        }
        // The startup connection ends here; the driver's source is independent.
        drop(control);
        cathedral_user_runtime::write(b"Cathedral desktop: shared input control ready\n")?;
        Ok(())
    }
    #[cfg(not(feature = "shell"))]
    pub fn start_input(&mut self) -> Result<(), Error> {
        self.input(0)
    }
}
