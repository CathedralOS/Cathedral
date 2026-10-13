//! Board-approved, non-DMA register windows. This is not arbitrary physical access.
//! Add a device only after auditing reset, exclusive IRQ routing and register effects.
use super::DeviceError;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DeviceId {
    AuxiliarySerial,
}
impl DeviceId {
    pub fn available(self) -> bool {
        cfg!(target_arch = "x86_64")
    }
    pub fn register_bytes(self) -> u64 {
        8
    }

    /// # Safety
    /// Sole CPU, interrupts disabled, exclusive custody of this device and IRQ.
    pub unsafe fn reset(self) {
        #[cfg(target_arch = "x86_64")]
        // SAFETY: Caller owns the selected non-DMA UART and route.
        unsafe {
            super::pc::serial::reset();
        }
    }
    /// # Safety
    /// Same custody as reset; a live grant has validated the register offset.
    pub unsafe fn read(self, offset: u64) -> Result<u8, DeviceError> {
        if !self.available() || offset >= self.register_bytes() {
            return Err(DeviceError::InvalidArgument);
        }
        #[cfg(target_arch = "x86_64")]
        // SAFETY: Offset is in the exclusively granted eight-byte window.
        return Ok(unsafe { super::pc::serial::read(offset as u16) });
        #[cfg(not(target_arch = "x86_64"))]
        Err(DeviceError::InvalidArgument)
    }
    /// # Safety
    /// Same custody as read. Every byte value is safe within this approved aperture.
    pub unsafe fn write(self, offset: u64, _value: u8) -> Result<(), DeviceError> {
        if !self.available() || offset >= self.register_bytes() {
            return Err(DeviceError::InvalidArgument);
        }
        #[cfg(target_arch = "x86_64")]
        // SAFETY: Offset is in the exclusively granted non-DMA UART window.
        unsafe {
            super::pc::serial::write(offset as u16, _value);
        }
        Ok(())
    }
    /// # Safety
    /// IRQs off, live grant armed, installed vector and exclusive interrupt route.
    pub unsafe fn arm(self) {
        #[cfg(target_arch = "x86_64")]
        // SAFETY: Core committed one notification before enabling this route.
        unsafe {
            super::pc::serial::arm();
        }
    }
    pub fn take_interrupt(self) -> bool {
        #[cfg(target_arch = "x86_64")]
        return super::pc::serial::take_interrupt();
        #[cfg(not(target_arch = "x86_64"))]
        false
    }
}
