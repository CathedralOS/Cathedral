// Own the firmware mapping topology, replacing its execution permissions.
// New stack/heap mappings are 4-KiB, writable and NX, with unmapped guards.

use ::x86_64::{
    PhysAddr, VirtAddr,
    registers::{
        control::{Cr0, Cr0Flags, Cr3, Cr4, Cr4Flags},
        model_specific::{Efer, EferFlags},
    },
    structures::paging::{
        FrameAllocator, Mapper, OffsetPageTable, Page, PageTable, PageTableFlags, PhysFrame,
        Size4KiB,
    },
};

const PAGE_SIZE: u64 = 4096;
const STACK_BASE: u64 = 0xffff_8000_0000_0000;
const STACK_PAGES: u64 = 16;
const EMERGENCY_PAGES: u64 = 4;
const HEAP_BASE: u64 = 0xffff_9000_0000_0000;
// Includes the full eight-account matrix session and persistent CPU accounting.
pub const HEAP_BYTES: usize = 128 * 1024;
// OVMF can map the qemu64 CPU's complete physical envelope, not just guest RAM.
const TABLE_BUDGET: usize = 4096;

use crate::{MemoryError, memory::KernelImage};

pub struct BootLayout {
    root: PhysFrame,
    pub table_count: usize,
    pub stack_bottom: u64,
    pub stack_top: u64,
    pub stack_guard: u64,
    /// Double fault, NMI, machine check, maskable IRQ, in that order.
    pub emergency_tops: [u64; 4],
    pub heap_start: u64,
    pub heap_bytes: usize,
}

impl BootLayout {
    /// Resolve the physical backing of a boot mapping.
    ///
    /// # Safety
    /// Sole CPU, IRQs off, kernel root active; no overlapping table access.
    pub unsafe fn physical_address(&self, address: u64) -> Option<u64> {
        use ::x86_64::structures::paging::Translate;
        // SAFETY: BootLayout owns permanent tables; boot mappings are never retired.
        let mapper = unsafe {
            OffsetPageTable::new(
                &mut *(self.root_address() as *mut PageTable),
                VirtAddr::zero(),
            )
        };
        mapper
            .translate_addr(VirtAddr::try_new(address).ok()?)
            .map(|p| p.as_u64())
    }
    /// Mapped stack used for maskable interrupt dispatch; no CPU-specific slot leaks to boot.
    pub fn interrupt_stack(&self) -> crate::StackRange {
        let top = self.emergency_tops[3];
        crate::StackRange {
            top,
            bottom: top - 4 * 4096,
            guard: top - 5 * 4096,
        }
    }
    pub fn root_address(&self) -> u64 {
        self.root.start_address().as_u64()
    }

    /// # Safety
    /// The unique frame source used during preparation still owns every table
    /// and backing frame. Execution remains single-CPU with IRQs disabled.
    pub unsafe fn activate(&self) {
        // SAFETY: The profile checked NX support, 4-level paging and no PCID.
        // Preserved identity mappings retain the live image, stack and tables.
        unsafe {
            Efer::update(|flags| flags.insert(EferFlags::NO_EXECUTE_ENABLE));
            Cr0::update(|flags| flags.insert(Cr0Flags::WRITE_PROTECT));
            // Firmware may have cached global translations; CR3 alone cannot retire them.
            let cr4 = Cr4::read();
            Cr4::write(cr4 & !Cr4Flags::PAGE_GLOBAL);
            Cr3::write(self.root, Cr3::read().1);
            Cr4::write(cr4);
        }
    }
}

struct Frames<'a, F> {
    next: &'a mut F,
    tables: usize,
}

impl<F: FnMut() -> Option<u64>> Frames<'_, F> {
    fn frame(&mut self) -> Result<PhysFrame, MemoryError> {
        let address = (self.next)().ok_or(MemoryError::OutOfFrames)?;
        if address == 0 || !address.is_multiple_of(PAGE_SIZE) || address >= (1 << 48) {
            return Err(MemoryError::InvalidFrame);
        }
        Ok(PhysFrame::from_start_address(PhysAddr::new(address)).unwrap())
    }
}

// SAFETY: prepare_memory's caller guarantees unique, unused, identity-mapped
// physical frames. No safe API constructs Frames or invokes its mapper.
unsafe impl<F: FnMut() -> Option<u64>> FrameAllocator<Size4KiB> for Frames<'_, F> {
    fn allocate_frame(&mut self) -> Option<PhysFrame> {
        if self.tables >= TABLE_BUDGET {
            return None;
        }
        let frame = self.frame().ok()?;
        self.tables += 1;
        Some(frame)
    }
}

/// # Safety
/// Requires QEMU/OVMF's identity-mapped physical memory, a valid active paging
/// hierarchy, ring 0, single CPU and IRQs off. Each callback result must be a
/// unique unused writable RAM frame, disjoint from all live firmware resources.
/// Failure consumes any frames already requested; boot must stop on failure.
pub unsafe fn prepare_memory(
    image: &KernelImage<'_>,
    next: &mut impl FnMut() -> Option<u64>,
) -> Result<BootLayout, MemoryError> {
    let cr4 = Cr4::read();
    let nx = core::arch::x86_64::__cpuid(0x8000_0000).eax >= 0x8000_0001
        && core::arch::x86_64::__cpuid(0x8000_0001).edx & (1 << 20) != 0;
    if !nx || cr4.intersects(Cr4Flags::L5_PAGING | Cr4Flags::PCID) {
        return Err(MemoryError::UnsupportedCpu);
    }
    let mut frames = Frames { next, tables: 0 };
    // SAFETY: Active tables and callback backing satisfy the caller contract.
    let root = unsafe { inherited::copy(Cr3::read().0, 4, image, &mut frames)? };
    // SAFETY: The newly allocated root is exclusively owned and identity mapped.
    let table = unsafe { &mut *(root.start_address().as_u64() as *mut PageTable) };
    for base in [STACK_BASE, HEAP_BASE, super::stacks::TASK_BASE] {
        if !table[VirtAddr::new(base).p4_index()].is_unused() {
            return Err(MemoryError::OccupiedVirtualRange);
        }
    }
    // SAFETY: All copied table pages are exclusively owned and identity mapped.
    let mut mapper = unsafe { OffsetPageTable::new(table, VirtAddr::zero()) };
    let stack_bottom = STACK_BASE + PAGE_SIZE;
    let stack_top = stack_bottom + STACK_PAGES * PAGE_SIZE;
    // SAFETY: New virtual ranges are vacant and backing comes from unique frames.
    unsafe {
        map_range(&mut mapper, &mut frames, stack_bottom, STACK_PAGES)?;
    }
    let mut emergency_tops = [0; 4];
    let mut cursor = stack_top + PAGE_SIZE;
    for top in &mut emergency_tops {
        let bottom = cursor + PAGE_SIZE;
        // SAFETY: Each emergency stack has its own frames and unmapped guards.
        unsafe {
            map_range(&mut mapper, &mut frames, bottom, EMERGENCY_PAGES)?;
        }
        *top = bottom + EMERGENCY_PAGES * PAGE_SIZE;
        cursor = *top + PAGE_SIZE;
    }
    let heap_start = HEAP_BASE + PAGE_SIZE;
    // SAFETY: Heap range is vacant; backing is uniquely allocated and writable.
    unsafe {
        map_range(
            &mut mapper,
            &mut frames,
            heap_start,
            HEAP_BYTES as u64 / PAGE_SIZE,
        )?;
    }
    Ok(BootLayout {
        root,
        table_count: frames.tables,
        stack_bottom,
        stack_top,
        stack_guard: STACK_BASE,
        emergency_tops,
        heap_start,
        heap_bytes: HEAP_BYTES,
    })
}

unsafe fn map_range<F: FnMut() -> Option<u64>>(
    mapper: &mut OffsetPageTable<'_>,
    frames: &mut Frames<'_, F>,
    base: u64,
    count: u64,
) -> Result<(), MemoryError> {
    for index in 0..count {
        let frame = frames.frame()?;
        // SAFETY: Fresh exclusively owned RAM is identity mapped and writable.
        unsafe {
            core::ptr::write_bytes(
                frame.start_address().as_u64() as *mut u8,
                0,
                PAGE_SIZE as usize,
            );
        }
        let page =
            Page::<Size4KiB>::from_start_address(VirtAddr::new(base + index * PAGE_SIZE)).unwrap();
        let flags = PageTableFlags::PRESENT | PageTableFlags::WRITABLE | PageTableFlags::NO_EXECUTE;
        // SAFETY: Unique backing and previously unmapped virtual range. This
        // hierarchy is not active yet, so no current TLB entry requires flushing.
        unsafe { mapper.map_to(page, frame, flags, frames) }
            .map_err(|_| MemoryError::MapFailed)?
            .ignore();
    }
    Ok(())
}
