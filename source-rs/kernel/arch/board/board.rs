// Select the actual board's optional privileged device adapters, not a CPU-shaped fiction.
// These descriptors identify hardware; core still authenticates the caller and owns every grant.
#[derive(Debug, PartialEq, Eq)]
pub enum DeviceError {
    InvalidArgument,
    WouldBlock,
}
pub enum InputEvent {
    Byte(u16),
    Lost,
}
pub struct Devices {
    pub keyboard: Option<Keyboard>,
    pub disk: Option<Disk>,
}
pub fn devices() -> Devices {
    #[cfg(target_arch = "x86_64")]
    {
        pc::devices()
    }
    #[cfg(target_arch = "aarch64")]
    {
        virt::devices()
    }
}
#[derive(Clone, Copy)]
enum KeyboardKind {
    #[cfg(target_arch = "x86_64")]
    Ps2,
}
#[derive(Clone, Copy)]
enum DiskKind {
    #[cfg(target_arch = "x86_64")]
    Ata,
}
pub struct Keyboard {
    kind: KeyboardKind,
}
pub struct Disk {
    kind: DiskKind,
}
impl Keyboard {
    /// # Safety
    /// Sole CPU, IRQs masked, exclusive input custody. Fixed pointer transport only.
    pub unsafe fn virtual_pointer(
        &self,
        _command: u64,
        _data: u64,
    ) -> Result<[u32; 4], DeviceError> {
        match self.kind {
            #[cfg(target_arch = "x86_64")]
            // SAFETY: Adapter validates commands before privileged I/O.
            KeyboardKind::Ps2 => unsafe { pc::vmmouse::exchange(_command, _data) },
        }
    }

    /// # Safety
    /// Sole CPU, IRQs masked; own this controller's interrupt route and installed vectors.
    pub unsafe fn interrupts(&self, _enabled: bool) {
        match self.kind {
            #[cfg(target_arch = "x86_64")]
            // SAFETY: Caller owns the selected board adapter and its interrupt route.
            KeyboardKind::Ps2 => unsafe { pc::keyboard::interrupts(_enabled) },
        }
    }
    /// # Safety
    /// Sole CPU, IRQs masked, exclusive controller custody.
    pub unsafe fn read(&self) -> Option<InputEvent> {
        match self.kind {
            #[cfg(target_arch = "x86_64")]
            // SAFETY: Caller owns the selected controller; bounded nonblocking read.
            KeyboardKind::Ps2 => unsafe { pc::keyboard::read() },
        }
    }
    /// # Safety
    /// Same exclusive controller custody; adapter checks permitted command/data values.
    pub unsafe fn write(&self, _command: u64, _byte: u64) -> Result<(), DeviceError> {
        match self.kind {
            #[cfg(target_arch = "x86_64")]
            // SAFETY: Adapter validates before any port effect; caller owns the device.
            KeyboardKind::Ps2 => unsafe { pc::keyboard::write(_command, _byte) },
        }
    }
}
impl Disk {
    /// # Safety
    /// Sole CPU, IRQs masked, exclusive controller custody; adapter checks the register.
    pub unsafe fn read(&self, _register: u64) -> Result<u8, DeviceError> {
        match self.kind {
            #[cfg(target_arch = "x86_64")]
            // SAFETY: Caller owns this controller; adapter validates the register first.
            DiskKind::Ata => unsafe { pc::disk::read(_register) },
        }
    }
    /// # Safety
    /// Same custody; adapter validates register/value restrictions before any device effect.
    pub unsafe fn write(&self, _register: u64, _value: u64) -> Result<(), DeviceError> {
        match self.kind {
            #[cfg(target_arch = "x86_64")]
            // SAFETY: Caller owns this controller and adapter enforces the command whitelist.
            DiskKind::Ata => unsafe { pc::disk::write(_register, _value) },
        }
    }
    /// # Safety
    /// Exclusive controller, IRQs masked. Buffer is kernel-owned and valid for the entire transfer.
    pub unsafe fn transfer(&self, _bytes: &mut [u8; 512], _write: bool) -> Result<(), DeviceError> {
        match self.kind {
            #[cfg(target_arch = "x86_64")]
            // SAFETY: Caller owns controller and buffer; adapter checks readiness and transfer bounds.
            DiskKind::Ata => unsafe { pc::disk::transfer(_bytes, _write) },
        }
    }
}
