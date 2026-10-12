//! Saved-register ABI and initial user/kernel contexts; offsets match interrupt entry.S.
use super::interrupts;
use crate::StackRange;
use ::x86_64::instructions::segmentation::{CS, SS, Segment};
use core::arch::asm;
#[repr(C)]
#[derive(Clone, Default)]
pub(super) struct Registers {
    pub(super) r15: u64,
    pub(super) r14: u64,
    pub(super) r13: u64,
    pub(super) r12: u64,
    pub(super) r11: u64,
    pub(super) r10: u64,
    pub(super) r9: u64,
    pub(super) r8: u64,
    pub(super) rdi: u64,
    pub(super) rsi: u64,
    pub(super) rbp: u64,
    pub(super) rdx: u64,
    pub(super) rcx: u64,
    pub(super) rbx: u64,
    pub(super) rax: u64,
}
#[repr(C)]
#[derive(Clone, Default)]
pub(super) struct ReturnFrame {
    pub(super) instruction: u64,
    pub(super) code: u64,
    pub(super) flags: u64,
    pub(super) stack: u64,
    pub(super) segment: u64,
}
#[repr(C, align(16))]
#[derive(Clone)]
pub struct Context {
    pub(super) floating: [u8; 512],
    pub(super) registers: Registers,
    pub(super) frame: ReturnFrame,
}
const _: () = {
    assert!(size_of::<Context>() == 672);
    assert!(core::mem::offset_of!(Context, registers) == 512);
    assert!(core::mem::offset_of!(Context, frame) == 632);
};
impl Default for Context {
    fn default() -> Self {
        Self {
            floating: [0; 512],
            registers: Registers::default(),
            frame: ReturnFrame::default(),
        }
    }
}
impl Context {
    /// Fresh ring-3 state: no inherited kernel registers, FP data or I/O privilege.
    pub fn user(entry: u64, stack: u64, arguments: [u64; 2]) -> Self {
        let mut context = Self::default();
        context.floating[0..2].copy_from_slice(&0x037fu16.to_le_bytes());
        context.floating[24..28].copy_from_slice(&0x1f80u32.to_le_bytes());
        context.registers.rdi = arguments[0];
        context.registers.rsi = arguments[1];
        let (code, data) = interrupts::user_selectors();
        context.frame = ReturnFrame {
            instruction: entry,
            code: u64::from(code),
            flags: 0x202,
            stack,
            segment: u64::from(data),
        };
        context
    }

    pub fn is_user(&self) -> bool {
        self.frame.code & 3 == 3
    }
    pub fn syscall(&self) -> (u64, u64, u64, u64) {
        (
            self.registers.rax,
            self.registers.rdi,
            self.registers.rsi,
            self.registers.rdx,
        )
    }
    pub fn set_result(&mut self, result: u64) {
        self.registers.rax = result;
    }

    /// # Safety
    /// Stack is uniquely owned, writable and mapped throughout the task's life.
    /// Entry never returns. Called with IRQs off, outside the interrupt handler.
    pub unsafe fn new(stack: StackRange, entry: fn(usize) -> !, argument: usize) -> Self {
        assert!(stack.top.is_multiple_of(16) && stack.top - stack.bottom >= 4096);
        let mut context = Self::default();
        // SAFETY: Kernel owns FP policy; qemu64 supports FXSAVE/SSE, not AVX.
        unsafe {
            asm!("fxsave64 [{}]", in(reg) context.floating.as_mut_ptr(), options(nostack));
            // Win64 alignment and shadow space, with a poison return address.
            ((stack.top - 40) as *mut u64).write(0);
        }
        context.registers.rcx = entry as usize as u64;
        context.registers.rdx = argument as u64;
        context.frame = ReturnFrame {
            instruction: task_entry as *const () as u64,
            code: u64::from(CS::get_reg().0),
            flags: 0x202,
            stack: stack.top - 40,
            segment: u64::from(SS::get_reg().0),
        };
        context
    }
}

extern "win64" fn task_entry(entry: usize, argument: usize) -> ! {
    // SAFETY: Context::new installed this exact function-pointer type.
    let entry: fn(usize) -> ! = unsafe { core::mem::transmute(entry) };
    entry(argument)
}
// Exception stubs add vector/error between the GPRs and hardware return frame.
#[repr(C, align(16))]
pub(in crate::isa::x86_64) struct ExceptionContext {
    pub(super) floating: [u8; 512],
    pub(super) registers: Registers,
    pub(super) vector: u64,
    pub(super) error: u64,
    pub(super) frame: ReturnFrame,
}
const _: () = {
    assert!(core::mem::offset_of!(ExceptionContext, vector) == 632);
    assert!(core::mem::offset_of!(ExceptionContext, frame) == 648);
};
