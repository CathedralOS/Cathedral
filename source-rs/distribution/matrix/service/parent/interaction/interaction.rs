// Sole reader while tagged input and asynchronous presentation share a link.
// Synchronous operations still wait for both owners to retire.
use super::*;
use cathedral_stock_matrix::transport::{self, Reply};

impl Parent {
    pub(in crate::service) fn can_poll(&self) -> bool {
        !(self.refreshing() || self.poll_retry.is_some())
    }

    pub(in crate::service) fn progress_interaction(&mut self) -> Result<(), Error> {
        if let Some(until) = self.poll_retry
            && time::reached(time::now()?, until)
        {
            self.poll_retry = None;
        }
        if !self.upstream {
            return Ok(());
        }
        let poll = self
            .polling
            .as_ref()
            .filter(|p| p.reply.is_none())
            .map(|p| p.id);
        let presenting = self
            .pending
            .as_ref()
            .is_some_and(|p| p.waiting && p.reply.is_none())
            || self
                .scene
                .as_ref()
                .is_some_and(|p| p.waiting && p.reply.is_none());
        if poll.is_none() && !presenting {
            return Ok(());
        }
        let Some((lane, words)) = self.interaction.as_ref().unwrap().poll()? else {
            return Ok(());
        };
        let classified = transport::classify(words, poll, presenting);
        if !matches!(
            (lane, &classified),
            (Lane::Poll, Reply::Poll(_)) | (Lane::Visual, Reply::Presentation(_))
        ) {
            return Err(Error(abi::IO_ERROR as i64));
        }
        match classified {
            Reply::Poll(reply) => self.polling.as_mut().unwrap().reply = Some(reply),
            Reply::Presentation(status) => {
                if let Some(p) = &mut self.pending {
                    p.reply = Some(status);
                } else {
                    self.scene.as_mut().unwrap().reply = Some(status);
                }
            }
            Reply::Unmatched(tag) => time::storage_sample(
                "matrix-poll-unmatched",
                &[poll.unwrap_or(0) as u64, tag as u64],
            ),
            Reply::Invalid => return Err(Error(abi::IO_ERROR as i64)),
        }
        Ok(())
    }
}
