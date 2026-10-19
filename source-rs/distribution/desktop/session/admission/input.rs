//! Admit physical input only after the matrix accepts its source. Readiness is
//! bounded and exact; a failed driver is collected before teardown of its root.
use super::*;
impl Session {
    pub(super) fn input(&mut self, number: u64) -> Result<(), Error> {
        assert!(self.input.is_none());
        for op in [wire::SPAWN, wire::READ_PIXEL, wire::BIND] {
            if matrix_request(self.source, [op, 0, 0, 0, 0, 0, 0, 0])? != wire::error(abi::DENIED) {
                return Err(BAD_REPLY);
            }
        }
        let value = argument(
            Stage::Input,
            number,
            self.extent[0] | (self.extent[1] << 32),
        );
        let deadline = time::after(READY_TICKS)?;
        #[cfg(feature = "bootstrap-probes")]
        probes::refusal(value)?;
        self.input = Some(Launch::at(1)?.spawn_with(value, &[self.source.0, self.source.1])?);
        let control = Port::at(1)?.connect()?;
        let result = receive(control, deadline).and_then(|(bytes, n)| {
            if &bytes[..n] == b"ready" {
                Ok(())
            } else {
                Err(BAD_REPLY)
            }
        });
        control.0.close()?;
        control.1.close()?;
        result
    }
    #[cfg(not(feature = "shell"))]
    pub fn start_input(&mut self) {
        self.input(0).expect("lab input admission");
    }
}
