//! The actual UART protocol lives in platform/drivers/uart_16550/interrupt.rs.
//! This fixture supplies its syscall transport and deliberately abandons dirty hardware.
use cathedral_contracts::user as abi;
use cathedral_uart_16550::interrupt::{Registers, Uart};
use cathedral_user_runtime::{
    Error,
    device::{self, Device},
    ipc::Handle,
};
struct Port(Device);
impl Registers for Port {
    type Error = Error;
    fn read(&self, offset: u64) -> Result<u8, Error> {
        self.0.read8(offset)
    }
    fn write(&self, offset: u64, value: u8) -> Result<(), Error> {
        self.0.write8(offset, value)
    }
    fn arm(&self) -> Result<(), Error> {
        self.0.arm()
    }
    fn wait(&self) -> Result<(), Error> {
        self.0.wait()
    }
}
pub fn run(previous: u64) -> u64 {
    let device = Device::at(0).unwrap();
    assert_ne!(device.raw(), previous);
    assert_eq!(
        Device::from_raw(previous).read8(0),
        Err(Error(abi::BAD_HANDLE as i64))
    );
    assert_eq!(device.read8(8), Err(Error(abi::INVALID_ARGUMENT as i64)));
    assert_eq!(
        device.write8(u64::MAX, 1),
        Err(Error(abi::INVALID_ARGUMENT as i64))
    );
    assert_eq!(
        device::raw(abi::DEVICE_WRITE8, device.raw(), 7, 256),
        Err(Error(abi::INVALID_ARGUMENT as i64))
    );
    assert_eq!(device.read8(7).unwrap(), 0); // Previous scratch register reset.
    assert_eq!(device.read8(1).unwrap(), 0); // Previous interrupt source disabled.
    assert_eq!(device.try_wait(), Err(Error(abi::WOULD_BLOCK as i64)));
    let requests = Handle::bootstrap(0).unwrap();
    let replies = Handle::bootstrap(1).unwrap();
    replies.send(&device.raw().to_le_bytes()).unwrap();
    let mut command = [0; 64];
    requests.receive(&mut command).unwrap();
    let mode = command[0];
    if mode == 2 {
        device.arm().unwrap();
        replies.send(b"waiting").unwrap();
        device.wait().unwrap(); // No enabled source: supervisor cancels this blocked task.
        panic!("unearned IRQ after replacement");
    }
    let mut uart = Uart::start(Port(device), true).unwrap();
    assert_eq!(device.arm(), Err(Error(abi::BUSY as i64)));
    masked_delivery(&uart, device);
    for byte in 0..16u8 {
        while !uart.try_send(byte ^ 0xa5).unwrap() {
            cathedral_user_runtime::yield_now();
        }
        let mut received = [0; 16];
        loop {
            let count = uart.receive(&mut received).unwrap();
            if count == 0 {
                continue;
            }
            assert_eq!(count, 1);
            assert_eq!(received[0], byte ^ 0xa5);
            break;
        }
    }
    replies.send(b"loopback").unwrap();
    if mode == 1 {
        device.write8(7, 0xee).unwrap();
        // Leave a notification and unread byte behind before faulting.
        while !uart.try_send(0x77).unwrap() {
            cathedral_user_runtime::yield_now();
        }
        device.wait().unwrap();
        // A second byte remains unread with the route masked.
        device.write8(0, 0x88).unwrap();
        // SAFETY: Intentional isolated userspace fault; exercises grant revocation.
        unsafe {
            #[cfg(target_arch = "x86_64")]
            core::arch::asm!("ud2", options(noreturn));
            #[cfg(target_arch = "aarch64")]
            core::arch::asm!("brk #0", options(noreturn));
        }
    }
    device.release().unwrap();
    assert_eq!(device.read8(0), Err(Error(abi::BAD_HANDLE as i64)));
    assert_eq!(device.arm(), Err(Error(abi::BAD_HANDLE as i64)));
    assert_eq!(Device::at(0).unwrap_err(), Error(abi::BAD_HANDLE as i64));
    0
}

fn masked_delivery(uart: &Uart<Port>, device: Device) {
    assert!(uart.try_send(0x33).unwrap());
    device.wait().unwrap();
    assert_eq!(device.read8(0).unwrap(), 0x33);
    // The first notification masked IRQ3. A fresh source must not wake us until rearm.
    while !uart.try_send(0x44).unwrap() {
        cathedral_user_runtime::yield_now();
    }
    while device.read8(5).unwrap() & 1 == 0 {
        cathedral_user_runtime::yield_now();
    }
    for _ in 0..8 {
        cathedral_user_runtime::yield_now();
        assert_eq!(device.try_wait(), Err(Error(abi::WOULD_BLOCK as i64)));
    }
    assert_eq!(device.read8(0).unwrap(), 0x44);
    device.arm().unwrap();
    // A PIC-latched edge can now produce a spurious notification. Uart::receive
    // drains/checks hardware rather than assuming every notification contains a byte.
}
