//! Recursively ask every direct child what stopping this matrix would lose.
//! The event loop keeps serving their upstream work; no child is stopped by a
//! survey. Missing, malformed, busy or late answers never count as permission.
use super::*;

pub(super) struct Survey {
    request: Request,
    waiting: u8,
    uncertain: bool,
}

impl Children {
    pub(super) fn survey_loss(
        &mut self,
        request: Request,
        parent: &Parent,
        server: &mut Server,
    ) -> Result<(), Error> {
        // An in-flight admission must not install an unexamined child after we
        // report clean. Conservatively refuse while storage/admission is busy.
        if self.loss.is_some() || self.pending.iter().any(Option::is_some) || parent.storing() {
            return server.reply(&request, &wire::encode(wire::error(abi::BUSY)));
        }
        let mut survey = Survey {
            request,
            waiting: 0,
            uncertain: false,
        };
        for (slot, child) in self.slots.iter().enumerate() {
            let Some(child) = child else {
                continue;
            };
            let sent = child.stale < control::MAX_STALE
                && child.control.is_some_and(|control| {
                    control
                        .0
                        .send(&wire::encode([wire::LOSS, 0, 0, 0, 0, 0, 0, 0]))
                        .is_ok()
                });
            if sent {
                survey.waiting |= 1 << slot;
                self.pending[slot] = Some((request, time::after(100)?, Purpose::Loss));
                self.retiring[slot] = None;
            } else {
                survey.uncertain = true;
            }
        }
        self.loss = Some(survey);
        self.finish_survey(server)
    }

    pub(super) fn loss_answer(
        &mut self,
        slot: usize,
        answer: Result<wire::Packet, u64>,
        server: &mut Server,
    ) -> Result<(), Error> {
        let survey = self.loss.as_mut().expect("loss reply without survey");
        survey.waiting &= !(1 << slot);
        survey.uncertain |= !matches!(answer, Ok(words) if words[0] == 0 && words[1] == 0);
        self.finish_survey(server)
    }

    fn finish_survey(&mut self, server: &mut Server) -> Result<(), Error> {
        if self.loss.as_ref().is_some_and(|survey| survey.waiting == 0) {
            let survey = self.loss.take().unwrap();
            // Only a boolean aggregate crosses the boundary; no descendant
            // identity, draft contents or local loss count escapes the matrix.
            server.reply(
                &survey.request,
                &wire::encode([0, u64::from(survey.uncertain), 0, 0, 0, 0, 0, 0]),
            )?;
        }
        Ok(())
    }
}
