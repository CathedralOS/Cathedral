//! Interrupt contexts for qemu64: GPRs, return frame and x87/MMX/SSE state.
//! AVX/XSAVE state is not part of this CPU profile.

use super::{StackRange, interrupts};
use ::x86_64::{
    instructions::segmentation::{CS, SS, Segment},
    registers::control::{Cr0, Cr0Flags, Cr4, Cr4Flags},
};
use core::{
    arch::{asm, global_asm},
    sync::atomic::{AtomicPtr, AtomicU64, Ordering},
};

#[repr(C)]
#[derive(Clone, Default)]
struct Registers {
    r15: u64,
    r14: u64,
    r13: u64,
    r12: u64,
    r11: u64,
    r10: u64,
    r9: u64,
    r8: u64,
    rdi: u64,
    rsi: u64,
    rbp: u64,
    rdx: u64,
    rcx: u64,
    rbx: u64,
    rax: u64,
}
#[repr(C)]
#[derive(Clone, Default)]
struct ReturnFrame {
    instruction: u64,
    code: u64,
    flags: u64,
    stack: u64,
    segment: u64,
}
#[repr(C, align(16))]
#[derive(Clone)]
pub struct Context {
    floating: [u8; 512],
    registers: Registers,
    frame: ReturnFrame,
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

/// Enable architectural context saves before publishing the timer/yield gates.
/// # Safety
/// Boot CPU owns floating-point policy, with IRQs off and no tasks running.
pub(super) unsafe fn enable_context_save() {
    let features = core::arch::x86_64::__cpuid(1).edx;
    assert_eq!(features & ((1 << 24) | (1 << 25)), (1 << 24) | (1 << 25));
    // SAFETY: The selected CPU supports FXSAVE/SSE; no lazy FP owner exists.
    unsafe {
        Cr0::update(|flags| {
            flags.remove(Cr0Flags::EMULATE_COPROCESSOR | Cr0Flags::TASK_SWITCHED);
            flags.insert(Cr0Flags::MONITOR_COPROCESSOR);
        });
        Cr4::update(|flags| flags.insert(Cr4Flags::OSFXSR | Cr4Flags::OSXMMEXCPT_ENABLE));
    }
}
extern "win64" fn task_entry(entry: usize, argument: usize) -> ! {
    // SAFETY: Context::new installed this exact function-pointer type.
    let entry: fn(usize) -> ! = unsafe { core::mem::transmute(entry) };
    entry(argument)
}
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum SwitchCause {
    Timer,
    Device,
    Yield,
    Syscall,
    Fault(UserFault),
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct UserFault {
    pub vector: u64,
    pub error: u64,
    pub address: u64,
    pub instruction: u64,
}
pub type SwitchHandler = unsafe fn(&Context, SwitchCause, u64) -> *const Context;
static HANDLER: AtomicPtr<()> = AtomicPtr::new(core::ptr::null_mut());

/// # Safety
/// IRQs are off on the sole CPU. Handler cannot allocate, suspend or enable IRQs.
/// Returned context stays live until assembly returns. Saved contexts cannot
/// refer to the shared IRQ stack after dispatch returns.
pub unsafe fn set_switch_handler(handler: Option<SwitchHandler>) {
    HANDLER.store(
        handler.map_or(core::ptr::null_mut(), |value| value as *mut ()),
        Ordering::Release,
    );
}
pub(super) extern "win64" fn dispatch(context: *const Context, cause: u64) -> *const Context {
    let handler = HANDLER.load(Ordering::Acquire);
    if handler.is_null() {
        return context;
    }
    // SAFETY: Assembly supplies a fully initialized aligned context. Only the
    // setter publishes this exact callback type, with interrupts disabled.
    unsafe {
        let handler: SwitchHandler = core::mem::transmute(handler);
        handler(
            &*context,
            match cause {
                0 => SwitchCause::Yield,
                1 => SwitchCause::Timer,
                2 => SwitchCause::Syscall,
                3 => SwitchCause::Device,
                _ => unreachable!(),
            },
            interrupts::ticks(),
        )
    }
}

// Exception stubs add vector/error between the GPRs and hardware return frame.
#[repr(C, align(16))]
pub(super) struct ExceptionContext {
    floating: [u8; 512],
    registers: Registers,
    vector: u64,
    error: u64,
    frame: ReturnFrame,
}
const _: () = {
    assert!(core::mem::offset_of!(ExceptionContext, vector) == 632);
    assert!(core::mem::offset_of!(ExceptionContext, frame) == 648);
};
pub(super) extern "win64" fn dispatch_exception(
    raw: &ExceptionContext,
    address: u64,
) -> *const Context {
    let handler = HANDLER.load(Ordering::Acquire);
    // NMI, double fault and machine check remain machine-fatal regardless of CPL.
    if raw.frame.code & 3 == 3
        && matches!(raw.vector, 0 | 1 | 4..=7 | 10..=14 | 16 | 17 | 19)
        && !handler.is_null()
    {
        let context = Context {
            floating: raw.floating,
            registers: raw.registers.clone(),
            frame: raw.frame.clone(),
        };
        // SAFETY: Setter publishes this callback; IRQs are masked. The callback
        // must copy this temporary context into its own permanent storage.
        unsafe {
            let handler: SwitchHandler = core::mem::transmute(handler);
            return handler(
                &context,
                SwitchCause::Fault(UserFault {
                    vector: raw.vector,
                    error: raw.error,
                    address,
                    instruction: raw.frame.instruction,
                }),
                interrupts::ticks(),
            );
        }
    }
    interrupts::dispatch_fault(raw.vector, raw.error, raw.frame.instruction, address)
}
/// # Safety
/// Requires installed vector 48 and task-switch callback. Caller holds no
/// references into mutable scheduler state or non-suspendable locks.
pub unsafe fn suspend() {
    // SAFETY: Vector 48 saves/restores the same full context as the timer.
    unsafe {
        asm!("int 0x30");
    }
}

global_asm!(include_str!("register_probe.S"), ticks = sym interrupts::TICKS);
unsafe extern "win64" {
    fn cathedral_register_probe(progress: *const AtomicU64, deadline: u64, pattern: u64) -> u64;
}

/// Busy-loop across timer interrupts, checking GPRs, all sixteen XMM registers,
/// x87 values and MXCSR. It never yields voluntarily.
/// # Safety
/// Kernel task on the qemu64 profile with a live timer and IRQs enabled. Deadline
/// is less than half the tick range ahead; pattern is a small positive integer.
pub unsafe fn probe_registers(progress: &AtomicU64, deadline: u64, pattern: u64) -> bool {
    // SAFETY: Assembly obeys Win64, restores caller FP/nonvolatile registers,
    // and atomically increments the supplied live counter.
    unsafe { cathedral_register_probe(progress, deadline, pattern) != 0 }
}
