//! Interrupt-driven non-DMA UART protocol, independent of the kernel or syscall ABI.
//! The supplied transport owns a register capability and a one-shot IRQ notification.
pub trait Registers {
    type Error;
    fn read(&self, offset: u64) -> Result<u8, Self::Error>;
    fn write(&self, offset: u64, value: u8) -> Result<(), Self::Error>;
    fn arm(&self) -> Result<(), Self::Error>;
    fn wait(&self) -> Result<(), Self::Error>;
}
pub struct Uart<R> {
    registers: R,
    receiving: bool,
}
impl<R: Registers> Uart<R> {
    /// Configure 9600 baud, 8N1 and one-byte receive interrupts. Loopback is a lab option.
    pub fn start(registers: R, loopback: bool) -> Result<Self, R::Error> {
        registers.write(3, 3)?;
        registers.write(1, 0)?;
        registers.write(3, 0x80)?;
        registers.write(0, 12)?;
        registers.write(1, 0)?;
        registers.write(3, 3)?;
        registers.write(2, 7)?;
        registers.write(4, if loopback { 0x18 } else { 0x0b })?;
        registers.write(1, 1)?;
        registers.arm()?;
        Ok(Self {
            registers,
            receiving: false,
        })
    }
    /// One bounded transmit attempt; caller chooses retry or backpressure policy.
    pub fn try_send(&self, byte: u8) -> Result<bool, R::Error> {
        if self.registers.read(5)? & 0x20 == 0 {
            return Ok(false);
        }
        self.registers.write(0, byte)?;
        Ok(true)
    }
    /// Drain the FIFO to clear the source before rearming. Notifications may be spurious.
    pub fn receive(&mut self, bytes: &mut [u8; 16]) -> Result<usize, R::Error> {
        if !self.receiving {
            self.registers.wait()?;
            self.receiving = true;
        }
        let mut count = 0;
        while count < bytes.len() && self.registers.read(5)? & 1 != 0 {
            bytes[count] = self.registers.read(0)?;
            count += 1;
        }
        // New bytes may arrive during draining. Keep custody across bounded batches
        // until the source clears; rearming an asserted edge-triggered line can stall.
        if self.registers.read(5)? & 1 == 0 {
            self.registers.arm()?;
            self.receiving = false;
        }
        Ok(count)
    }
}
