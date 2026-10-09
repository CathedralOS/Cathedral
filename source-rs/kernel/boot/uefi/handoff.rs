//! Move boot state across the architecture's non-returning stack switch.

use crate::memory::PreparedMemory;
use cathedral_arch as arch;
use cathedral_uart_16550::SerialPort;

pub struct BootState {
    pub memory: PreparedMemory,
    pub console: SerialPort,
}

struct Continuation {
    boot: BootState,
    entry: fn(BootState) -> !,
}

pub fn enter(memory: PreparedMemory, console: SerialPort, entry: fn(BootState) -> !) -> ! {
    let mut continuation = Continuation {
        boot: BootState { memory, console },
        entry,
    };
    let layout = &continuation.boot.memory.layout;
    // SAFETY: Retained mappings preserve the image and old stack containing
    // continuation. Resume moves it once; old storage remains reserved forever.
    unsafe {
        layout.activate();
        arch::enter_stack(
            layout.stack_top,
            resume,
            (&mut continuation as *mut Continuation).cast(),
        );
    }
}

unsafe fn resume(context: *mut ()) -> ! {
    // SAFETY: enter transfers this exact live continuation and never returns.
    let continuation = unsafe { context.cast::<Continuation>().read() };
    (continuation.entry)(continuation.boot)
}
