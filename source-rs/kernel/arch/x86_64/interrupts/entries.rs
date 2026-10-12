//! CPU vector bindings into context dispatch; entry.S owns register/root transitions.
use super::{BREAKPOINTS, IRQ_STACK, TICKS};
use core::arch::global_asm;

global_asm!(include_str!("entry.S"), fault = sym super::super::context::dispatch_exception, switch = sym super::super::context::dispatch, ticks = sym TICKS,
    breakpoints = sym BREAKPOINTS, irq_stack = sym IRQ_STACK,
    entry_root = sym super::super::user::ENTRY_ROOT, return_root = sym super::super::user::RETURN_ROOT);

macro_rules! exception_entries {
    ($($name:ident),+ $(,)?) => {
        unsafe extern "C" { $(pub(super) fn $name();)+ }
        pub(super) const EXCEPTIONS: [unsafe extern "C" fn(); 32] = [$($name),+];
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
    pub(super) fn cathedral_exception_255();
    pub(super) fn cathedral_timer_entry();
    pub(super) fn cathedral_keyboard_entry();
    pub(super) fn cathedral_yield_entry();
    pub(super) fn cathedral_syscall_entry();
}
