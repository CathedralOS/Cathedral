//! One asynchronous upstream binding refresh; local authority commits only in the launch owner.
use super::*;
use cathedral_stock_matrix::policy::operation::Ticket;
pub(in crate::service::parent) struct Binding {
    ticket: Ticket,
    slot: usize,
    receive: Option<Handle>,
    deadline: u64,
    ready: Option<Result<usize, u64>>,
    #[cfg(feature = "binding-drop-reply")]
    dropped: bool,
    #[cfg(feature = "binding-delay")]
    hold_until: Option<u64>,
}
pub(in crate::service) struct Bound {
    pub ticket: Ticket,
    pub result: Result<usize, u64>,
}
impl Parent {
    pub(in crate::service) fn begin_storage_binding(
        &mut self,
        ticket: Ticket,
        slot: usize,
    ) -> Result<(), Error> {
        assert!(!self.storing());
        let deadline = time::after(if cfg!(feature = "binding-delay") && slot == 2 {
            400
        } else {
            100
        })?;
        let mut p = Binding {
            ticket,
            slot,
            deadline,
            receive: None,
            ready: None,
            #[cfg(feature = "binding-drop-reply")]
            dropped: false,
            #[cfg(feature = "binding-delay")]
            hold_until: if slot == 2 {
                time::storage_sample("launch-bind-held", &[]);
                Some(time::after(300)?)
            } else {
                None
            },
        };
        if !self.upstream {
            p.ready = Some(Ok(self.storage_links.len()));
        } else {
            let sent = self
                .storage_upstream
                .as_mut()
                .ok_or(Error(abi::PEER_CLOSED as i64))
                .and_then(|connection| {
                    connection.begin([wire::STORAGE_INFO, 0, 0, 0, 0, 0, 0, deadline], None)
                });
            match sent {
                Ok(receive) => p.receive = Some(receive),
                Err(e) => {
                    self.retire_binding(Error(e.0))?;
                    p.ready = Some(Err(e.0 as u64));
                }
            }
        }
        self.binding_pending = Some(p);
        Ok(())
    }
    pub(in crate::service) fn progress_binding(&mut self) -> Result<Option<Bound>, Error> {
        let Some(mut p) = self.binding_pending.take() else {
            return Ok(None);
        };
        #[cfg(feature = "binding-delay")]
        if let Some(until) = p.hold_until {
            if !time::reached(time::now()?, until) {
                self.binding_pending = Some(p);
                return Ok(None);
            }
            p.hold_until = None;
            time::storage_sample("launch-bind-released", &[]);
        }
        if p.ready.is_none() {
            #[cfg(feature = "binding-drop-reply")]
            let discarded = p.dropped;
            #[cfg(not(feature = "binding-drop-reply"))]
            let discarded = false;
            // The fixture discards a completed transport reply, so it must wait
            // at the application boundary instead of polling a completed ID again.
            let response = if discarded {
                if time::reached(time::now()?, p.deadline) {
                    Err(Error(abi::TIMED_OUT as i64))
                } else {
                    Ok(None)
                }
            } else {
                self.storage_upstream
                    .as_mut()
                    .ok_or(Error(abi::PEER_CLOSED as i64))?
                    .poll(p.deadline)
            };
            match response {
                Ok(None) => {
                    self.binding_pending = Some(p);
                    return Ok(None);
                }
                Ok(Some((words, object))) => {
                    // Disposable fixture: discard one real reply, then let the
                    // ordinary empty-channel deadline and retirement path run.
                    #[cfg(feature = "binding-drop-reply")]
                    if p.slot == 2 && !p.dropped {
                        drop(object);
                        p.dropped = true;
                        time::storage_sample("launch-bind-reply-dropped", &[]);
                        self.binding_pending = Some(p);
                        return Ok(None);
                    }
                    if object.is_some() || words[3..] != [0; 5] || (words[0] == 0 && words[1] == 0)
                    {
                        self.retire_binding(Error(abi::IO_ERROR as i64))?;
                        p.ready = Some(Err(abi::IO_ERROR));
                    } else {
                        self.adopt_upstream(words);
                        p.ready = Some(if words[0] == 0 { Ok(1) } else { Err(words[0]) });
                    }
                }
                Err(e) => {
                    self.retire_binding(Error(e.0))?;
                    p.ready = Some(Err(e.0 as u64));
                }
            }
        }
        Ok(Some(Bound {
            ticket: p.ticket,
            result: p.ready.unwrap(),
        }))
    }
    fn retire_binding(&mut self, error: Error) -> Result<(), Error> {
        time::storage_sample("launch-bind-retired", &[error.0 as u64]);
        self.retire_storage(0)
    }
    pub(super) fn binding_wait(&self) -> Option<(Handle, u64)> {
        let p = self.binding_pending.as_ref()?;
        #[cfg(feature = "binding-delay")]
        if p.hold_until.is_some() {
            return None;
        }
        p.receive.map(|h| (h, p.deadline))
    }
    #[cfg(feature = "binding-delay")]
    pub(in crate::service) fn binding_delay_deadline(&self) -> Option<u64> {
        self.binding_pending.as_ref().and_then(|p| p.hold_until)
    }
    pub(super) fn binding_client(&self) -> Option<usize> {
        self.binding_pending.as_ref().map(|p| p.slot)
    }
}
