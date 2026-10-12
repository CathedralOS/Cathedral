//! Eager general-register and FP/SIMD context; offsets must match entry.S.
#[repr(C, align(16))]
#[derive(Clone, Default)]
pub struct Context {
    floating: [u128; 32],
    registers: [u64; 31],
    pub(super) stack: u64,
    pub(super) instruction: u64,
    status: u64,
    fpcr: u64,
    fpsr: u64,
}
const _: () = {
    assert!(size_of::<Context>() == 800);
    assert!(align_of::<Context>() == 16);
    assert!(core::mem::offset_of!(Context, floating) == 0);
    assert!(core::mem::offset_of!(Context, registers) == 512);
    assert!(core::mem::offset_of!(Context, stack) == 760);
    assert!(core::mem::offset_of!(Context, instruction) == 768);
    assert!(core::mem::offset_of!(Context, status) == 776);
    assert!(core::mem::offset_of!(Context, fpcr) == 784);
    assert!(core::mem::offset_of!(Context, fpsr) == 792);
};
impl Context {
    /// # Safety
    /// Exclusive mapped kernel stack; entry and argument remain valid until task retirement.
    /// Called on the sole EL1 CPU with IRQs masked. Entry never returns.
    pub unsafe fn new(stack: crate::StackRange, entry: fn(usize) -> !, argument: usize) -> Self {
        assert!(stack.top.is_multiple_of(16) && stack.top - stack.bottom >= 4096);
        let mut context = Self {
            instruction: task_entry as *const () as u64,
            stack: stack.top,
            // EL1h with IRQ enabled; debug, SError and FIQ remain masked.
            status: 5 | (1 << 9) | (1 << 8) | (1 << 6),
            ..Self::default()
        };
        context.registers[0] = entry as usize as u64;
        context.registers[1] = argument as u64;
        context
    }
    pub fn user(entry: u64, stack: u64, arguments: [u64; 2]) -> Self {
        let mut c = Self {
            instruction: entry,
            stack,
            ..Self::default()
        };
        c.registers[..2].copy_from_slice(&arguments);
        c
    }
    pub fn is_user(&self) -> bool {
        self.status & 15 == 0
    }
    pub fn syscall(&self) -> (u64, u64, u64, u64) {
        (
            self.registers[8],
            self.registers[0],
            self.registers[1],
            self.registers[2],
        )
    }
    pub fn set_result(&mut self, result: u64) {
        self.registers[0] = result;
    }
}

extern "C" fn task_entry(entry: usize, argument: usize) -> ! {
    // SAFETY: Context::new installs this exact function type and a live argument.
    let entry: fn(usize) -> ! = unsafe { core::mem::transmute(entry) };
    entry(argument)
}
