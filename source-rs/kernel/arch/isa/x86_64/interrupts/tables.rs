//! Resident privilege and interrupt tables; all initialization precedes interrupt delivery.
use super::{BootLayout, entries::*};
use ::x86_64::{
    PrivilegeLevel, VirtAddr,
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
    ptr::addr_of_mut,
    sync::atomic::{AtomicU64, Ordering},
};

static USER_SELECTORS: AtomicU64 = AtomicU64::new(0);
static mut TSS: TaskStateSegment = TaskStateSegment::new();
static mut GDT: GlobalDescriptorTable = GlobalDescriptorTable::new();
static mut IDT: [Entry<HandlerFunc>; 256] = [const { Entry::missing() }; 256];
const _: () = assert!(size_of::<Entry<HandlerFunc>>() == 16);

pub(in crate::isa::x86_64) fn user_selectors() -> (u16, u16) {
    let selectors = USER_SELECTORS.load(Ordering::Acquire);
    assert_ne!(selectors, 0, "user context before GDT installation");
    (selectors as u16, (selectors >> 16) as u16)
}

// SAFETY contract: sole CPU, IRQs off, permanent boot layout, exactly one installation.
pub(super) unsafe fn install_segments(layout: &BootLayout) {
    // SAFETY: Caller supplies exclusive initialization and permanent stack mappings.
    unsafe {
        let tss = addr_of_mut!(TSS);
        let mut value = TaskStateSegment::new();
        // Ring-3 exceptions must not overwrite the suspended boot stack.
        value.privilege_stack_table[0] = VirtAddr::new(layout.emergency_tops[3]);
        for (index, top) in layout.emergency_tops.iter().enumerate() {
            value.interrupt_stack_table[index] = VirtAddr::new(*top);
        }
        tss.write(value);
        let gdt = &mut *addr_of_mut!(GDT);
        let code = gdt.append(Descriptor::kernel_code_segment());
        let data = gdt.append(Descriptor::kernel_data_segment());
        let task = gdt.append(Descriptor::tss_segment_unchecked(tss));
        let user_code = gdt.append(Descriptor::user_code_segment());
        let user_data = gdt.append(Descriptor::user_data_segment());
        USER_SELECTORS.store(
            u64::from(user_code.0) | (u64::from(user_data.0) << 16),
            Ordering::Release,
        );
        gdt.load_unsafe();
        CS::set_reg(code);
        DS::set_reg(data);
        ES::set_reg(data);
        SS::set_reg(data);
        FS::set_reg(SegmentSelector(0));
        GS::set_reg(SegmentSelector(0));
        load_tss(task);
    }
}

pub(super) unsafe fn install_gates() {
    // SAFETY: Segment table is active, handler code is resident, IDT is exclusively owned.
    unsafe {
        let idt = &mut *addr_of_mut!(IDT);
        for (vector, entry) in idt.iter_mut().enumerate() {
            let address = if vector < 32 {
                EXCEPTIONS[vector] as *const ()
            } else if vector == 32 {
                cathedral_timer_entry as *const ()
            } else if vector == 33 {
                cathedral_keyboard_entry as *const ()
            } else if vector == 48 {
                cathedral_yield_entry as *const ()
            } else if vector == 128 {
                cathedral_syscall_entry as *const ()
            } else {
                cathedral_exception_255 as *const ()
            };
            let options = entry.set_handler_addr(VirtAddr::from_ptr(address));
            if vector == 128 {
                options.set_privilege_level(PrivilegeLevel::Ring3);
            }
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
}
