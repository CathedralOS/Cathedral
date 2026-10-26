//! Wait on actual connections/deadlines; retain exactly one authenticated event.
use super::*;
impl Host {
    pub(super) fn wait_event(&mut self) -> Result<bool, Error> {
        assert!(self.event.is_none(), "cannot replace an undispatched event");
        let parent = &mut *self.parent;
        let world = &mut *self.world;
        let server = &mut *self.server;
        let children = &mut *self.children;
        let deferred = &mut *self.deferred;
        let hold_storage = self.hold_storage;
        #[cfg(feature = "interaction-delay")]
        let delay = &mut self.delay;
        let (interests, mut deadline) = children.wait(
            parent.upstream_busy() || parent.storing(),
            cathedral_user_runtime::time::now()?,
        );
        children.notify(world)?;
        let mut extra =
            [Interest::Ipc(cathedral_user_runtime::ipc::Handle::from_raw(0)); CHILDREN * 2 + 6];
        let mut count = 0;
        if !hold_storage && let Some((receive, until)) = parent.storage_wait() {
            extra[count] = Interest::Ipc(receive);
            count += 1;
            deadline = Some(deadline.map_or(until, |d| {
                if cathedral_user_runtime::time::reached(d, until) {
                    until
                } else {
                    d
                }
            }));
        }
        #[cfg(any(feature = "recovery-delay", feature = "launch-delay"))]
        if let Some(until) = parent.recovery_delay_deadline() {
            deadline = Some(deadline.map_or(until, |d| {
                if cathedral_user_runtime::time::reached(d, until) {
                    until
                } else {
                    d
                }
            }));
        }
        #[cfg(feature = "binding-delay")]
        if let Some(until) = parent.binding_delay_deadline() {
            deadline = Some(deadline.map_or(until, |d| {
                if cathedral_user_runtime::time::reached(d, until) {
                    until
                } else {
                    d
                }
            }));
        }
        for (receive, until) in [
            parent.refresh_wait(),
            parent.presentation_wait(),
            parent.scene_wait(),
        ]
        .into_iter()
        .flatten()
        {
            // Both exchanges may wait on the same interaction receiver.
            if !extra[..count]
                .iter()
                .any(|h| matches!(h,Interest::Ipc(old) if old.raw()==receive.raw()))
            {
                extra[count] = Interest::Ipc(receive);
                count += 1;
            }
            if let Some(until) = until {
                deadline = Some(deadline.map_or(until, |d| {
                    if cathedral_user_runtime::time::reached(d, until) {
                        until
                    } else {
                        d
                    }
                }));
            }
        }
        if !hold_storage {
            if let Some(interest) = parent.storage_wake() {
                extra[count] = interest;
                count += 1;
            }
            if let Some(until) = parent.storage_transfer_deadline() {
                deadline = Some(deadline.map_or(until, |d| {
                    if cathedral_user_runtime::time::reached(d, until) {
                        until
                    } else {
                        d
                    }
                }));
            }
        }
        if let Some(interest) = parent.output_wake()? {
            extra[count] = interest;
            count += 1;
        }
        if let Some(until) = parent.output_deadline() {
            deadline = Some(deadline.map_or(until, |d| {
                if cathedral_user_runtime::time::reached(d, until) {
                    until
                } else {
                    d
                }
            }));
        }
        if let Some(until) = parent.poll_retry {
            deadline = Some(deadline.map_or(until, |d| {
                if cathedral_user_runtime::time::reached(d, until) {
                    until
                } else {
                    d
                }
            }));
        }
        for interest in interests.into_iter().flatten() {
            extra[count] = interest;
            count += 1;
        }
        let input_ready = world.input_ready();
        if parent.can_poll() && parent.refresh_due && input_ready {
            deadline = Some(cathedral_user_runtime::time::now()?);
        }
        #[cfg(feature = "interaction-delay")]
        for until in delay
            .deadline()
            .into_iter()
            .chain(parent.hold_storage_until)
        {
            deadline = Some(deadline.map_or(until, |d| {
                if cathedral_user_runtime::time::reached(d, until) {
                    until
                } else {
                    d
                }
            }));
        }
        if parent.can_poll()
            && input_ready
            && let Some(wake) = parent.wake
        {
            extra[count] = wake.interest();
            count += 1;
        }
        #[cfg(not(feature = "interaction-delay"))]
        let queued = deferred.take(parent);
        #[cfg(feature = "interaction-delay")]
        let queued = delay.take(server)?.or_else(|| deferred.take(parent));
        deferred.pause(server, parent);
        #[cfg(feature = "trace-storage")]
        let from_queue = queued.is_some();
        let event = match queued {
            Some(event) => event,
            None => match server.next_event_for(&extra[..count], deadline)? {
                Some(event) => event,
                // An upstream reply or a child's wake made an extra handle
                // ready; the next turn's progress steps take it from there.
                None => return Ok(false),
            },
        };
        #[cfg(feature = "interaction-delay")]
        let event = if !parent.upstream {
            match delay.intercept(event, server, world)? {
                Some(event) => event,
                None => return Ok(false),
            }
        } else {
            event
        };
        #[cfg(feature = "trace-storage")]
        if !from_queue
            && let Event::Request(r) = &event
            && r.client.is_some_and(|c| parent.storage_stream(c))
        {
            let op = wire::decode(&r.bytes[..r.len]).map_or(0, |w| w[0] & 0xffff);
            cathedral_user_runtime::time::storage_trace(
                if parent.upstream {
                    "nested-receive"
                } else {
                    "root-receive"
                },
                r.client.unwrap() as u64,
                op,
            );
        }
        self.event = Some(event);
        Ok(true)
    }
}
