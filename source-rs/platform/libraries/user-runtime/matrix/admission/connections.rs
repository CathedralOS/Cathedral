//! Allocate the child's startup bundle; each rollback closes only retained rights.
use super::*;
use crate::ipc;
pub(super) struct Connections {
    pub data: Pair,
    pub notify: Handle,
    pub storage: Option<Pair>,
    pub producer: Option<Pair>,
    pub capture: Option<Pair>,
    producer_at: usize,
    capture_at: usize,
    initial: [Handle; 9],
    count: usize,
}
impl Connections {
    pub fn create(separate_storage: bool, producer: bool, capture: bool) -> Result<Self, Error> {
        let producer_at = if separate_storage { 5 } else { 3 };
        let capture_at = producer_at + if producer { 2 } else { 0 };
        let count = capture_at + if capture { 2 } else { 0 };
        let empty = Handle::from_raw(0);
        let mut channels = [(empty, empty); 9];
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
        let mut initial = [empty; 9];
        initial[..3].copy_from_slice(&[channels[0].0, channels[1].1, channels[2].1]);
        if separate_storage {
            initial[3..5].copy_from_slice(&[channels[3].0, channels[4].1]);
        }
        if producer {
            initial[producer_at..producer_at + 2]
                .copy_from_slice(&[channels[producer_at].0, channels[producer_at + 1].1]);
        }
        if capture {
            initial[capture_at..capture_at + 2]
                .copy_from_slice(&[channels[capture_at].0, channels[capture_at + 1].1]);
        }
        Ok(Self {
            data: (channels[0].1, channels[1].0),
            notify: channels[2].0,
            storage: separate_storage.then_some((channels[3].1, channels[4].0)),
            producer: producer.then_some((channels[producer_at].1, channels[producer_at + 1].0)),
            capture: capture.then_some((channels[capture_at].1, channels[capture_at + 1].0)),
            producer_at,
            capture_at,
            initial,
            count,
        })
    }
    pub fn initial(&self) -> &[Handle] {
        &self.initial[..self.count]
    }
    pub fn broker_initial(&self) -> &[Handle] {
        &self.initial[..2]
    }
    pub fn deferred(&self) -> [Option<Handle>; 7] {
        [
            Some(self.initial[2]),
            self.storage.map(|_| self.initial[3]),
            self.storage.map(|_| self.initial[4]),
            self.producer.map(|_| self.initial[self.producer_at]),
            self.producer.map(|_| self.initial[self.producer_at + 1]),
            self.capture.map(|_| self.initial[self.capture_at]),
            self.capture.map(|_| self.initial[self.capture_at + 1]),
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
        if let Some((input, output)) = self.producer {
            input.close()?;
            output.close()?;
        }
        if let Some((input, output)) = self.capture {
            input.close()?;
            output.close()?;
        }
        Ok(())
    }
}
