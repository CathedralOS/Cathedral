//! Allocate the child's startup bundle; each rollback closes only retained rights.
use super::*;
use crate::ipc;
pub(super) struct Connections {
    pub data: Pair,
    pub notify: Handle,
    pub storage: Option<Pair>,
    initial: [Handle; 5],
    count: usize,
}
impl Connections {
    pub fn create(separate_storage: bool) -> Result<Self, Error> {
        let count = if separate_storage { 5 } else { 3 };
        let empty = Handle::from_raw(0);
        let mut channels = [(empty, empty); 5];
        for index in 0..count {
            match ipc::channel() {
                Ok(pair) => channels[index] = pair,
                Err(error) => {
                    for &(send, receive) in &channels[..index] {
                        send.close()?;
                        receive.close()?;
                    }
                    return Err(error);
                }
            }
        }
        Ok(Self {
            data: (channels[0].1, channels[1].0),
            notify: channels[2].0,
            storage: separate_storage.then_some((channels[3].1, channels[4].0)),
            initial: [
                channels[0].0,
                channels[1].1,
                channels[2].1,
                channels[3].0,
                channels[4].1,
            ],
            count,
        })
    }
    pub fn initial(&self) -> &[Handle] {
        &self.initial[..self.count]
    }
    pub fn broker_initial(&self) -> &[Handle] {
        &self.initial[..2]
    }
    pub fn deferred(&self) -> [Option<Handle>; 3] {
        [
            Some(self.initial[2]),
            (self.count == 5).then_some(self.initial[3]),
            (self.count == 5).then_some(self.initial[4]),
        ]
    }
    pub fn close_deferred(&self) -> Result<(), Error> {
        for endpoint in self.deferred().into_iter().flatten() {
            endpoint.close()?;
        }
        Ok(())
    }
    /// Admission failed: none of the endpoints moved into the child.
    pub fn rollback(self) -> Result<(), Error> {
        for endpoint in self.initial() {
            endpoint.close()?;
        }
        self.close_parent()
    }
    /// After successful admission only these ends still belong to this process.
    pub fn close_parent(self) -> Result<(), Error> {
        for endpoint in [self.data.0, self.data.1, self.notify] {
            endpoint.close()?;
        }
        if let Some((input, output)) = self.storage {
            input.close()?;
            output.close()?;
        }
        Ok(())
    }
}
