//! Resume publication after each upstream reply. One in-flight batch owns the
//! visual exchange. Tagged input shares the receiver through interaction.rs.
//! Slow replies retain ownership and never trigger a replay.
use super::*;
use cathedral_platform_contracts::display;
use cathedral_platform_contracts::display::composition;
use cathedral_user_runtime::server::Request;

pub(super) struct Pending {
    pub(super) request: Request,
    revision: u64,
    local_revision: u64,
    step: usize, // 0 begin, 1..=LIMIT updates, LIMIT+1 commit, LIMIT+2 abort.
    status: u64,
    compact: bool,
    pub(super) waiting: bool,
    pub(super) reply: Option<u64>,
    deadline: u64,
    late: bool,
    retry: Option<u64>,
}
impl Parent {
    pub(in super::super) fn publishing(&self) -> bool {
        self.pending.is_some()
    }
    pub(in super::super) fn presentation_wait(&self) -> Option<(Handle, Option<u64>)> {
        self.pending.as_ref().map(|p| {
            (
                self.output().unwrap().1,
                p.retry.or_else(|| (!p.late).then_some(p.deadline)),
            )
        })
    }
    pub(in super::super) fn prune_batches(&mut self, world: &World) {
        for (slot, batch) in self.batches.iter_mut().enumerate() {
            if !world.binding_matches(slot, batch.incarnation, batch.epoch) {
                batch.reset();
            }
        }
    }
    pub(in super::super) fn present(
        &mut self,
        request: Request,
        local_revision: u64,
        prepared: [wire::Packet; wire::PRESENT_LIMIT],
    ) -> Result<(), Error> {
        assert!(self.pending.is_none());
        self.next_revision = self
            .next_revision
            .checked_add(1)
            .ok_or(Error(abi::NO_MEMORY as i64))?;
        self.pending = Some(Pending {
            request,
            local_revision,
            revision: self.next_revision,
            step: 0,
            status: 0,
            compact: false,
            waiting: false,
            reply: None,
            deadline: time::after(300)?,
            late: false,
            retry: None,
        });
        #[cfg(feature = "trace-input")]
        time::trace("present-start");
        let mut count = 0;
        for i in 0..wire::PRESENT_LIMIT {
            count += usize::from(self.batches[request.client.unwrap()].entries[i].is_some());
        }
        if self.upstream
            && count <= 2
            && wire::rectangles(self.grant.epoch, self.next_revision, &prepared[..count]).is_some()
        {
            let pending = self.pending.as_mut().unwrap();
            pending.step = wire::PRESENT_LIMIT + 1;
            pending.compact = true;
        }
        // Validation unwinds before sending: the service loop owns all upstream
        // waits and avoids nesting encoding under the request dispatch stack.
        Ok(())
    }
    pub(in super::super) fn progress_presentation(
        &mut self,
        world: &mut World,
        server: &mut Server,
    ) -> Result<(), Error> {
        let Some(p) = &self.pending else {
            return Ok(());
        };
        if !p.waiting {
            match self.send_presentation(world) {
                Ok(()) => {
                    let p = self.pending.as_mut().unwrap();
                    p.waiting = true;
                    p.retry = None;
                }
                Err(Error(code)) if code == abi::WOULD_BLOCK as i64 => {
                    self.pending.as_mut().unwrap().retry = Some(time::after(1)?);
                }
                Err(error) => return Err(error),
            }
            return Ok(());
        }
        let now = time::now()?;
        let status = if self.upstream {
            self.pending.as_mut().unwrap().reply.take()
        } else {
            let mut bytes = [0; 64];
            match self.output()?.1.receive_until(&mut bytes, now) {
                Ok(len) => {
                    Some(display::decode(&bytes[..len]).ok_or(Error(abi::IO_ERROR as i64))?[0])
                }
                Err(Error(code)) if code == abi::TIMED_OUT as i64 => None,
                Err(error) => return Err(error),
            }
        };
        let Some(status) = status else {
            let p = self.pending.as_mut().unwrap();
            if !p.late && time::reached(now, p.deadline) {
                time::storage_sample("matrix-present-delayed", &[p.revision]);
                p.late = true;
            }
            return Ok(());
        };
        if status != 0 {
            let p = self.pending.as_ref().unwrap();
            time::storage_sample(
                "present-upstream-refused",
                &[u64::from(self.upstream), p.step as u64, status],
            );
        }
        let p = self.pending.as_mut().unwrap();
        let batch = &self.batches[p.request.client.unwrap()];
        let done = if p.step == wire::PRESENT_LIMIT + 2 {
            // A failed abort leaves upstream ownership uncertain; retire the stream.
            if status != 0 {
                return Err(Error(status as i64));
            }
            true
        } else if status != 0 {
            p.status = status;
            if p.step == 0 || p.compact {
                true
            } else {
                p.step = wire::PRESENT_LIMIT + 2;
                false
            }
        } else if p.step == wire::PRESENT_LIMIT + 1 {
            true
        } else {
            p.step += 1;
            while p.step <= wire::PRESENT_LIMIT && batch.entries[p.step - 1].is_none() {
                p.step += 1;
            }
            false
        };
        if !done {
            self.pending.as_mut().unwrap().waiting = false;
            // Continue without parking before the next step has been sent.
            return self.progress_presentation(world, server);
        }
        let p = self.pending.take().unwrap();
        if p.late {
            time::storage_sample("matrix-present-resumed", &[p.revision]);
        }
        let slot = p.request.client.unwrap();
        let batch = &mut self.batches[slot];
        if p.status == 0 {
            // Index the small fixed arena directly; avoid large unoptimized iterator
            // machinery and tuple temporaries in the bounded guest image/stack.
            for i in 0..wire::PRESENT_LIMIT {
                let Some((local, object)) = &mut batch.entries[i] else {
                    continue;
                };
                let words = world
                    .preview_surface(slot, p.request.incarnation, *local)
                    .expect("publication holds geometry stable");
                world
                    .surface(slot, p.request.incarnation, *local)
                    .expect("publication holds geometry stable");
                self.images.commit(words[2], object.take());
            }
            batch.committed(p.local_revision);
        } else if wire::decode(&p.request.bytes[..p.request.len])
            .is_some_and(|w| w[0] == wire::PRESENT_RECTS)
        {
            // The self-contained operation leaves no staged transaction on rejection.
            batch
                .abort(p.local_revision)
                .expect("local compact batch is active");
        }
        server.reply(&p.request, &wire::encode(wire::error(p.status)))?;
        #[cfg(feature = "trace-input")]
        time::trace("present-done");
        Ok(())
    }
    fn output(&self) -> Result<(Handle, Handle), Error> {
        (if self.upstream {
            self.pair
        } else {
            self.display
        })
        .ok_or(Error(abi::DENIED as i64))
    }
    fn send_presentation(&self, world: &mut World) -> Result<(), Error> {
        let p = self.pending.as_ref().unwrap();
        let batch = &self.batches[p.request.client.unwrap()];
        let (send, _) = self.output()?;
        if p.compact {
            let mut updates = [[0; 8]; 2];
            let mut count = 0;
            for (i, update) in updates.iter_mut().enumerate() {
                if let Some((local, _)) = &batch.entries[i] {
                    *update = world
                        .preview_surface(p.request.client.unwrap(), p.request.incarnation, *local)
                        .expect("publication holds geometry stable");
                    count += 1;
                }
            }
            let packet = wire::rectangles(self.grant.epoch, p.revision, &updates[..count])
                .expect("validated compact rectangles");
            return send.send(&wire::encode(packet));
        }
        let op = match p.step {
            0 => wire::PRESENT_BEGIN,
            n if n == wire::PRESENT_LIMIT + 1 => wire::PRESENT_COMMIT,
            n if n == wire::PRESENT_LIMIT + 2 => wire::PRESENT_ABORT,
            _ => 0,
        };
        let (bytes, object) = if op != 0 {
            let bytes = if self.upstream {
                wire::encode([op, self.grant.epoch, p.revision, 0, 0, 0, 0, 0])
            } else {
                let op = match op {
                    wire::PRESENT_BEGIN => composition::MATRIX_BEGIN,
                    wire::PRESENT_COMMIT => composition::MATRIX_COMMIT,
                    _ => composition::MATRIX_ABORT,
                };
                let mut bytes = [0; 64];
                bytes[..48].copy_from_slice(&display::encode([op, p.revision, 0, 0, 0, 0]));
                bytes
            };
            (bytes, None)
        } else {
            let local = batch.entries[p.step - 1].as_ref().unwrap().0;
            let words = world
                .preview_surface(p.request.client.unwrap(), p.request.incarnation, local)
                .expect("publication holds geometry stable");
            let object = batch.entries[p.step - 1].as_ref().unwrap().1.as_ref();
            let bytes = if self.upstream {
                wire::encode(words)
            } else {
                let [_, _, key, x, y, w, h, value] = words;
                if key == 0 || key > composition::MATRIX_TILES as u64 {
                    return Err(Error(abi::BAD_HANDLE as i64));
                }
                let op = if object.is_some() {
                    composition::MATRIX_IMAGE
                } else if words[0] == wire::SURFACE_CURSOR {
                    composition::MATRIX_CURSOR
                } else {
                    composition::MATRIX_RECT
                };
                let mut values = [op | ((key - 1) << 16), x, y, w, h, value];
                if words[0] == wire::ERASE {
                    values[1..].fill(0);
                }
                let mut bytes = [0; 64];
                bytes[..48].copy_from_slice(&display::encode(values));
                bytes
            };
            (bytes, object)
        };
        let bytes = &bytes[..if self.upstream { 64 } else { 48 }];
        match object {
            Some(object) => send.send_memory(bytes, object),
            None => send.send(bytes),
        }
    }
}
