// Own a 4-KiB, 48-bit EL1 translation tree; keep QEMU virt RAM identity mapped.
// The board proof fixes RAM below 2 GiB. Low peripherals are Device-nGnRnE.
use core::arch::asm;
use tables::{NORMAL, NX, PageTables, map};
pub const PAGE: u64 = 4096;
#[derive(Debug)]
pub enum MemoryError {
    UnsupportedCpu,
    OutOfFrames,
    InvalidFrame,
    OccupiedVirtualRange,
    MapFailed,
}
/// # Safety
/// Frames are unique, unused, identity-mapped RAM; release follows every alias's retirement.
pub unsafe trait StackFrames {
    fn allocate(&mut self) -> Option<u64>;
    /// # Safety
    /// The caller has retired all mappings and users of this frame.
    unsafe fn release(&mut self, address: u64);
}
pub struct BootLayout {
    root: u64,
    pub table_count: usize,
    pub stack_bottom: u64,
    pub stack_top: u64,
    pub stack_guard: u64,
    pub emergency_tops: [u64; 4],
    pub heap_start: u64,
    pub heap_bytes: usize,
}
impl BootLayout {
    pub fn root_address(&self) -> u64 {
        self.root
    }
    /// # Safety
    /// Sole EL1 CPU, IRQs off; image and old stack are identity mapped in both regimes.
    pub unsafe fn activate(&self) {
        let control: u64;
        // SAFETY: Identity code/stack remain valid while MMU is briefly disabled.
        // 48-bit VA, 4-KiB granule, 40-bit PA, inner-shareable WB walks; TTBR1 disabled.
        unsafe {
            asm!("mrs {c}, sctlr_el1", c=out(reg) control, options(nostack));
            asm!("dsb sy", "msr sctlr_el1, {off}", "isb",
                "msr mair_el1, {mair}", "msr tcr_el1, {tcr}", "msr ttbr0_el1, {root}",
                "isb", "tlbi vmalle1", "dsb sy", "isb", "msr sctlr_el1, {on}", "isb",
                off=in(reg) (control & !1), on=in(reg) (control | 1), mair=in(reg) 0xffu64,
                tcr=in(reg) ((2u64<<32) | (1<<23) | (3<<12) | (1<<10) | (1<<8) | 16),
                root=in(reg) self.root, options(nostack));
        }
    }
}
/// # Safety
/// QEMU virt, EL1, IRQs masked, firmware exited; callback supplies unused conventional RAM.
pub unsafe fn prepare_memory(
    next: &mut impl FnMut() -> Option<u64>,
) -> Result<BootLayout, MemoryError> {
    let level: u64;
    let features: u64;
    // SAFETY: Privileged firmware handoff; inspect the required EL1/4-KiB/40-bit profile.
    unsafe {
        asm!("mrs {}, CurrentEL", "mrs {}, id_aa64mmfr0_el1",out(reg) level,out(reg) features,options(nostack));
    }
    if level != 4 || features & 15 < 2 || (features >> 28) & 15 == 15 {
        return Err(MemoryError::UnsupportedCpu);
    }
    let mut tables = PageTables::new(next);
    let root = tables.frame()?;
    let l1 = tables.frame()?;
    let devices = tables.frame()?;
    // SAFETY: Fresh exclusive tables and fixed QEMU board identity ranges.
    unsafe {
        (root as *mut u64).write(l1 | 3);
        (l1 as *mut u64).write(devices | 3);
        for i in 0..512 {
            (devices as *mut u64)
                .add(i)
                .write((i as u64 * 0x20_0000) | 1 | (1 << 10) | (1 << 2) | NX);
        }
        // RAM is normal WB and supervisor-only. EL1 image executes from its identity alias.
        (l1 as *mut u64)
            .add(1)
            .write(0x4000_0000 | 1 | NORMAL | (1 << 54));
    }
    let mut table_count = 3;
    let stack_guard = 0x10_0000_0000;
    let stack_bottom = stack_guard + PAGE;
    let stack_top = stack_bottom + 16 * PAGE;
    let emergency_tops = core::array::from_fn(|i| stack_top + (i as u64 + 1) * 6 * PAGE);
    let heap_start = 0x20_0000_1000;
    let heap_bytes = 256 * 1024;
    for (base, pages) in [(stack_bottom, 16), (heap_start, heap_bytes / PAGE as usize)]
        .into_iter()
        .chain(emergency_tops.iter().map(|top| (top - 4 * PAGE, 4)))
    {
        for i in 0..pages {
            let frame = tables.frame()?;
            // SAFETY: Fresh backing at vacant guarded kernel virtual addresses.
            unsafe {
                map(
                    root,
                    base + i as u64 * PAGE,
                    frame,
                    NORMAL | NX,
                    &mut || {
                        let frame = tables.frame()?;
                        table_count += 1;
                        Ok(frame)
                    },
                )?;
            }
        }
    }
    Ok(BootLayout {
        root,
        table_count,
        stack_bottom,
        stack_top,
        stack_guard,
        emergency_tops,
        heap_start,
        heap_bytes,
    })
}
