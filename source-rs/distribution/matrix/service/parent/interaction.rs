//! Sole reader while tagged input and asynchronous presentation share a link.
//! Synchronous operations still wait for both owners to retire.
use super::*;
use cathedral_stock_matrix::transport::{self, Reply};

impl Parent {
    pub(in crate::service) fn can_poll(&self) -> bool {
        !(self.refreshing()
            || self.poll_retry.is_some()
            || self.upstream && !self.separate_storage && self.storing())
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
            .is_some_and(|p| p.waiting && p.reply.is_none());
        if poll.is_none() && !presenting {
            return Ok(());
        }
        let mut bytes = [0; 64];
        let len = match self.pair.unwrap().1.receive_until(&mut bytes, time::now()?) {
            Ok(len) => len,
            Err(Error(code)) if code == abi::TIMED_OUT as i64 => return Ok(()),
            Err(error) => return Err(error),
        };
        let words = wire::decode(&bytes[..len]).ok_or(Error(abi::IO_ERROR as i64))?;
        match transport::classify(words, poll, presenting) {
            Reply::Poll(reply) => self.polling.as_mut().unwrap().reply = Some(reply),
            Reply::Presentation(status) => self.pending.as_mut().unwrap().reply = Some(status),
            Reply::Unmatched(tag) => time::storage_sample(
                "matrix-poll-unmatched",
                &[poll.unwrap_or(0) as u64, tag as u64],
            ),
            Reply::Invalid => return Err(Error(abi::IO_ERROR as i64)),
        }
        Ok(())
    }
}
