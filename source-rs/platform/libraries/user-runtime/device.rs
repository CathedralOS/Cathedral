//! Caller-local register windows and interrupt notifications. No physical-address selector.
//! A driver clears its hardware interrupt source, consumes the notification, then rearms.
use crate::{Error, abi, arch};

#[derive(Clone, Copy, Debug)]
pub struct Device(u64);
impl Device {
    pub fn at(index: u64) -> Result<Self, Error> {
        raw(abi::DEVICE_HANDLE, index, 0, 0).map(Self)
    }
    /// Local name only: copying these bits to another process does not grant access.
    pub fn from_raw(name: u64) -> Self {
        Self(name)
    }
    pub fn raw(self) -> u64 {
        self.0
    }
    pub fn read8(self, offset: u64) -> Result<u8, Error> {
        raw(abi::DEVICE_READ8, self.0, offset, 0).map(|v| v as u8)
    }
    pub fn write8(self, offset: u64, value: u8) -> Result<(), Error> {
        raw(abi::DEVICE_WRITE8, self.0, offset, value as u64).map(|_| ())
    }
    pub fn arm(self) -> Result<(), Error> {
        raw(abi::DEVICE_IRQ_ARM, self.0, 0, 0).map(|_| ())
    }
    pub fn wait(self) -> Result<(), Error> {
        raw(abi::DEVICE_IRQ_WAIT, self.0, 1, 0).map(|_| ())
    }
    pub fn try_wait(self) -> Result<(), Error> {
        raw(abi::DEVICE_IRQ_WAIT, self.0, 0, 0).map(|_| ())
    }
    pub fn release(self) -> Result<(), Error> {
        raw(abi::DEVICE_RELEASE, self.0, 0, 0).map(|_| ())
    }
}
pub fn raw(number: u64, first: u64, second: u64, third: u64) -> Result<u64, Error> {
    let value = arch::call3(number, first, second, third);
    if (value as i64) < 0 {
        Err(Error(value as i64))
    } else {
        Ok(value)
    }
}
