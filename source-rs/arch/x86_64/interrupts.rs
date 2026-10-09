//! Single-CPU GDT/TSS/IDT installation and bounded interrupt handlers.
//! No allocator, locks, Rust calls or floating-point state in the timer ISR.

use super::{BootLayout, halt_forever, stack_pointer};
use ::x86_64::{
    VirtAddr,
    instructions::{
        segmentation::{CS, DS, ES, FS, GS, SS, Segment},
        tables::{lidt, load_tss},
    },
    registers::segmentation::SegmentSelector,
    structures::{
        DescriptorTablePointer,
        gdt::{Descriptor, GlobalDescriptorTable},
        idt::{Entry, HandlerFunc},
        tss::TaskStateSegment,
    },
};
use core::{
    arch::{asm, global_asm},
    ptr::addr_of_mut,
    sync::atomic::{AtomicBool, AtomicPtr, AtomicU64, Ordering},
};

static INSTALLED: AtomicBool = AtomicBool::new(false);
static FAULT_HANDLER: AtomicPtr<()> = AtomicPtr::new(core::ptr::null_mut());
static TICKS: AtomicU64 = AtomicU64::new(0);
static BREAKPOINTS: AtomicU64 = AtomicU64::new(0);
static IRQ_STACK: AtomicU64 = AtomicU64::new(0);
static mut TSS: TaskStateSegment = TaskStateSegment::new();
static mut GDT: GlobalDescriptorTable = GlobalDescriptorTable::new();
static mut IDT: [Entry<HandlerFunc>; 256] = [const { Entry::missing() }; 256];
const _: () = assert!(size_of::<Entry<HandlerFunc>>() == 16);

#[derive(Debug)]
pub struct Fault {
    pub vector: u64,
    pub error: u64,
    pub instruction: u64,
    pub address: u64,
    pub stack: u64,
}

extern "win64" fn dispatch_fault(vector: u64, error: u64, instruction: u64, address: u64) -> ! {
    let handler = FAULT_HANDLER.load(Ordering::Acquire);
    if !handler.is_null() {
        // SAFETY: Only install_interrupts writes this pointer, using this exact
        // function type. The target is permanent code and this path never returns.
        let handler: fn(Fault) -> ! = unsafe { core::mem::transmute(handler) };
        handler(Fault {
            vector,
            error,
            instruction,
            address,
            stack: stack_pointer(),
        });
    }
    // SAFETY: A fatal trap runs privileged and cannot resume without a handler.
    unsafe { halt_forever() }
}

global_asm!(include_str!("entry.S"), fault = sym dispatch_fault, ticks = sym TICKS,
    breakpoints = sym BREAKPOINTS, irq_stack = sym IRQ_STACK);

macro_rules! exception_entries {
    ($($name:ident),+ $(,)?) => {
        unsafe extern "C" { $(fn $name();)+ }
        const EXCEPTIONS: [unsafe extern "C" fn(); 32] = [$($name),+];
    };
}
exception_entries!(
    cathedral_exception_0,
    cathedral_exception_1,
    cathedral_exception_2,
    cathedral_exception_3,
    cathedral_exception_4,
    cathedral_exception_5,
    cathedral_exception_6,
    cathedral_exception_7,
    cathedral_exception_8,
    cathedral_exception_9,
    cathedral_exception_10,
    cathedral_exception_11,
    cathedral_exception_12,
    cathedral_exception_13,
    cathedral_exception_14,
    cathedral_exception_15,
    cathedral_exception_16,
    cathedral_exception_17,
    cathedral_exception_18,
    cathedral_exception_19,
    cathedral_exception_20,
    cathedral_exception_21,
    cathedral_exception_22,
    cathedral_exception_23,
    cathedral_exception_24,
    cathedral_exception_25,
    cathedral_exception_26,
    cathedral_exception_27,
    cathedral_exception_28,
    cathedral_exception_29,
    cathedral_exception_30,
    cathedral_exception_31,
);
unsafe extern "C" {
    fn cathedral_exception_255();
    fn cathedral_timer_entry();
}

/// # Safety
/// Called exactly once on the sole boot CPU with IRQs off, after activating
/// layout's mappings and stack. Image, tables and stacks must remain resident.
/// The callback is a non-returning, allocation-free fatal diagnostic path.
pub unsafe fn install_interrupts(layout: &BootLayout, handler: fn(Fault) -> !) {
    assert!(
        !INSTALLED.load(Ordering::Acquire),
        "interrupt tables already installed"
    );
    FAULT_HANDLER.store(handler as *mut (), Ordering::Release);
    // SAFETY: Initialization is exclusive, single-CPU and before IRQ delivery.
    // Static table addresses and all four mapped emergency stacks are permanent.
    unsafe {
        let tss = addr_of_mut!(TSS);
        let mut value = TaskStateSegment::new();
        value.privilege_stack_table[0] = VirtAddr::new(layout.stack_top);
        for (index, top) in layout.emergency_tops.iter().enumerate() {
            value.interrupt_stack_table[index] = VirtAddr::new(*top);
        }
        tss.write(value);
        let gdt = &mut *addr_of_mut!(GDT);
        let code = gdt.append(Descriptor::kernel_code_segment());
        let data = gdt.append(Descriptor::kernel_data_segment());
        let task = gdt.append(Descriptor::tss_segment_unchecked(tss));
        gdt.load_unsafe();
        CS::set_reg(code);
        DS::set_reg(data);
        ES::set_reg(data);
        SS::set_reg(data);
        FS::set_reg(SegmentSelector(0));
        GS::set_reg(SegmentSelector(0));
        load_tss(task);

        let idt = &mut *addr_of_mut!(IDT);
        for (vector, entry) in idt.iter_mut().enumerate() {
            let address = if vector < 32 {
                EXCEPTIONS[vector] as *const ()
            } else if vector == 32 {
                cathedral_timer_entry as *const ()
            } else {
                cathedral_exception_255 as *const ()
            };
            let options = entry.set_handler_addr(VirtAddr::from_ptr(address));
            match vector {
                8 => {
                    options.set_stack_index(0);
                }
                2 => {
                    options.set_stack_index(1);
                }
                18 => {
                    options.set_stack_index(2);
                }
                32..=255 => {
                    options.set_stack_index(3);
                }
                _ => {}
            }
        }
        lidt(&DescriptorTablePointer {
            limit: (size_of_val(idt) - 1) as u16,
            base: VirtAddr::from_ptr(idt.as_ptr()),
        });
    }
    INSTALLED.store(true, Ordering::Release);
}

/// # Safety
/// Requires installed Cathedral exception tables on the boot CPU.
pub unsafe fn test_breakpoint() {
    assert!(INSTALLED.load(Ordering::Acquire));
    let before = BREAKPOINTS.load(Ordering::Acquire);
    // SAFETY: Vector 3 has a returning assembly handler preserving all registers.
    unsafe {
        asm!("int3");
    }
    assert_eq!(BREAKPOINTS.load(Ordering::Acquire), before + 1);
}

/// # Safety
/// Requires installed IDT, exclusive ownership of PC PIC/PIT ports and one CPU.
pub unsafe fn start_timer() {
    assert!(INSTALLED.load(Ordering::Acquire));
    // SAFETY: The selected PC lab grants this CPU exclusive legacy-controller access.
    unsafe {
        crate::x86::legacy_timer::start_100_hz();
    }
}

/// Wait with the interrupt-enable/halt pair adjacent so a wake cannot be lost.
/// Returns with IRQs masked. The host smoke timeout bounds a broken timer.
/// # Safety
/// Requires the timer/IDT installed and no locks held across interrupt delivery.
pub unsafe fn wait_for_ticks(count: u64) -> u64 {
    let start = TICKS.load(Ordering::Acquire);
    loop {
        let elapsed = TICKS.load(Ordering::Acquire).wrapping_sub(start);
        if elapsed >= count {
            return elapsed;
        }
        // SAFETY: All handlers and stacks are installed; STI's interrupt shadow
        // ensures HLT executes before any pending IRQ can be serviced.
        unsafe {
            asm!("sti", "hlt", "cli", options(nostack));
        }
    }
}

pub fn last_irq_stack() -> u64 {
    IRQ_STACK.load(Ordering::Acquire)
}

/// # Safety
/// Requires the installed IDT. Intentionally raises a fatal invalid-opcode trap.
pub unsafe fn probe_invalid_opcode() -> ! {
    // SAFETY: Caller explicitly requests this diagnostic, with a valid trap path.
    unsafe {
        asm!("ud2", options(noreturn));
    }
}

/// # Safety
/// Requires installed exception handlers and double-fault IST. Guard must be
/// unmapped. Abandons the current stack to force a page-fault delivery failure.
pub unsafe fn probe_double_fault(guard: u64) -> ! {
    // SAFETY: The diagnostic deliberately invalidates RSP and never resumes.
    unsafe {
        asm!("mov rsp, rcx", "push rax", "ud2", in("rcx") guard + 8, options(noreturn));
    }
}
