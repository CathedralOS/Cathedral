use crate::interrupt::{Registers, Uart};
use core::cell::Cell;

#[derive(Default)]
struct Stream {
    remaining: Cell<u8>,
    waits: Cell<u8>,
    arms: Cell<u8>,
}
impl Registers for &Stream {
    type Error = ();
    fn read(&self, offset: u64) -> Result<u8, ()> {
        match offset {
            5 => Ok(0x20 | u8::from(self.remaining.get() != 0)),
            0 => {
                let next = self.remaining.get() - 1;
                self.remaining.set(next);
                Ok(next)
            }
            _ => Err(()),
        }
    }
    fn write(&self, _: u64, _: u8) -> Result<(), ()> {
        Ok(())
    }
    fn arm(&self) -> Result<(), ()> {
        self.arms.set(self.arms.get() + 1);
        Ok(())
    }
    fn wait(&self) -> Result<(), ()> {
        self.waits.set(self.waits.get() + 1);
        Ok(())
    }
}

#[test]
fn ongoing_arrivals_drain_in_bounded_batches_before_rearming() {
    let stream = Stream::default();
    let mut uart = Uart::start(&stream, false).unwrap();
    // Model more bytes arriving while the 16-byte FIFO is being drained.
    stream.remaining.set(32);
    let mut bytes = [0; 16];
    assert_eq!(uart.receive(&mut bytes), Ok(16));
    assert_eq!(stream.waits.get(), 1);
    assert_eq!(stream.arms.get(), 1);
    assert_eq!(uart.receive(&mut bytes), Ok(16));
    assert_eq!(stream.waits.get(), 1); // No second wait while the source stays asserted.
    assert_eq!(stream.arms.get(), 2);
    assert_eq!(uart.receive(&mut bytes), Ok(0)); // Spurious hint: check status and rearm.
    assert_eq!(stream.waits.get(), 2);
    assert_eq!(stream.arms.get(), 3);
}
