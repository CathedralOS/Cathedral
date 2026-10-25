//! Recursively ask every direct child what stopping this matrix would lose.
//! The event loop keeps serving their upstream work; no child is stopped by a
//! survey. Missing, malformed, busy or late answers never count as permission.
use super::*;
use cathedral_stock_matrix::policy::lifecycle as decisions;
use cathedral_user_runtime::server::Event;

pub(super) struct Survey {
    request: Request,
    policy: decisions::Survey,
}

impl Children {
    /// A loss survey only uses child control links. It must keep progressing
    /// through an ancestor snapshot wait, just like local child polling.
    pub(in crate::service) fn query_during_refresh(
        &mut self,
        event: &Event,
        parent: &Parent,
        server: &mut Server,
    ) -> Result<bool, Error> {
        if parent.refreshing()
            && let Event::Request(request) = event
            && request.control
            && wire::decode(&request.bytes[..request.len])
                == Some([wire::LOSS, 0, 0, 0, 0, 0, 0, 0])
        {
            self.survey_loss(*request, parent, server)?;
            return Ok(true);
        }
        Ok(false)
    }

    pub(super) fn survey_loss(
        &mut self,
        request: Request,
        parent: &Parent,
        server: &mut Server,
    ) -> Result<(), Error> {
        #[cfg(feature = "trace-storage")]
        time::storage_trace("matrix-loss-begin", request.incarnation, 0);
        // An in-flight admission must not install an unexamined child after we
        // report clean. Conservatively refuse while storage/admission is busy.
        let Some(policy) = decisions::Survey::begin([
            self.loss.is_some(),
            self.pending.iter().any(Option::is_some),
            parent.storing(),
            self.recovery_pending(),
        ]) else {
            time::storage_sample("matrix-loss-busy", &[]);
            return server.reply(&request, &wire::encode(wire::error(abi::BUSY)));
        };
        let mut survey = Survey { request, policy };
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
                #[cfg(feature = "trace-storage")]
                time::storage_trace("matrix-loss-send", request.incarnation, slot as u64);

                self.pending[slot] = Some((request, time::after(100)?, Purpose::Loss));
                self.retiring[slot] = None;
            } else {
                #[cfg(feature = "trace-storage")]
                time::storage_trace("matrix-loss-unsent", request.incarnation, slot as u64);
            }
            survey.policy.sent(slot, sent);
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
        #[cfg(feature = "trace-storage")]
        time::storage_trace(
            "matrix-loss-answer",
            survey.request.incarnation,
            slot as u64,
        );

        if !matches!(decisions::close(false, answer), decisions::Close::Stop) {
            let (status, loss) = match answer {
                Ok(words) => (words[0], words[1]),
                Err(code) => (code, 0),
            };
            time::storage_sample("matrix-loss-uncertain", &[slot as u64, status, loss]);
        }
        survey.policy.answer(slot, answer);
        self.finish_survey(server)
    }

    fn finish_survey(&mut self, server: &mut Server) -> Result<(), Error> {
        if let Some(uncertain) = self
            .loss
            .as_ref()
            .and_then(|survey| survey.policy.finish(self.recovery_pending()))
        {
            let survey = self.loss.take().unwrap();
            #[cfg(feature = "trace-storage")]
            time::storage_trace(
                "matrix-loss-end",
                survey.request.incarnation,
                u64::from(uncertain),
            );
            // Only a boolean aggregate crosses the boundary; no descendant
            // identity, draft contents or local loss count escapes the matrix.
            server.reply(
                &survey.request,
                &wire::encode([0, u64::from(uncertain), 0, 0, 0, 0, 0, 0]),
            )?;
        }
        Ok(())
    }
}
