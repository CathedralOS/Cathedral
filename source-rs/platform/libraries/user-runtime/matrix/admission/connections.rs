//! Allocate the child's startup bundle; each rollback closes only retained rights.
use super::*;
use crate::{ipc, notification::Notification, services::WakeSource};
pub(super) struct Connections {
    pub data: Pair,
    pub notify: WakeSource,
    pub wake: Option<Notification>,
    pub storage: Option<Pair>,
    pub producer: Option<Pair>,
    pub capture: Option<Pair>,
    producer_at: usize,
    capture_at: usize,
    initial: [Handle; 9],
    count: usize,
}
impl Connections {
    pub fn create(
        separate_storage: bool,
        producer: bool,
        capture: bool,
        brokered: bool,
    ) -> Result<Self, Error> {
        let producer_at = if separate_storage { 5 } else { 3 };
        let capture_at = producer_at + if producer { 2 } else { 0 };
        let count = capture_at + if capture { 2 } else { 0 };
        let empty = Handle::from_raw(0);
        let mut channels = [(empty, empty); 9];
        for index in 0..count {
            if brokered && index == 2 {
                continue;
            }
            match ipc::channel() {
                Ok(pair) => channels[index] = pair,
                Err(error) => {
                    close_channels(&channels[..index])?;
                    return Err(error);
                }
            }
        }
        let (notify, wake) = if brokered {
            match WakeSource::create() {
                Ok((source, wait)) => (source, Some(wait)),
                Err(error) => {
                    close_channels(&channels[..count])?;
                    return Err(error);
                }
            }
        } else {
            (WakeSource::Copied(channels[2].0), None)
        };
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
            notify,
            wake,
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
            self.wake.is_none().then_some(self.initial[2]),
            self.storage.map(|_| self.initial[3]),
            self.storage.map(|_| self.initial[4]),
            self.producer.map(|_| self.initial[self.producer_at]),
            self.producer.map(|_| self.initial[self.producer_at + 1]),
            self.capture.map(|_| self.initial[self.capture_at]),
            self.capture.map(|_| self.initial[self.capture_at + 1]),
        ]
    }
    pub fn close_deferred(&self) -> Result<(), Error> {
        if let Some(wake) = self.wake {
            wake.release()?;
        }
        for endpoint in self.deferred().into_iter().flatten() {
            endpoint.close()?;
        }
        Ok(())
    }
    /// Admission failed: none of the endpoints moved into the child.
    pub fn rollback(self) -> Result<(), Error> {
        for endpoint in self.initial() {
            if endpoint.raw() != 0 {
                endpoint.close()?;
            }
        }
        if let Some(wake) = self.wake {
            wake.release()?;
        }
        self.close_parent()
    }
    /// After successful admission only these ends still belong to this process.
    pub fn close_parent(self) -> Result<(), Error> {
        self.notify.close()?;
        for endpoint in [self.data.0, self.data.1] {
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

fn close_channels(channels: &[Pair]) -> Result<(), Error> {
    for &(send, receive) in channels {
        if send.raw() != 0 {
            send.close()?;
            receive.close()?;
        }
    }
    Ok(())
}
