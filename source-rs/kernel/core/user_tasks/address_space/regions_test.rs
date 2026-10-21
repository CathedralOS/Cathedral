//! Real-table refusal/retry proof. The fixture root must never execute.
use super::*;
use cathedral_kernel_contracts::memory as wire;

/// # Safety
/// Private inactive fixture root, IRQs off; backing exclusively retained by the
/// fixture. The selected slot is vacant and reserved. No user can observe the
/// deliberately injected translation that is absent from the normal ledger.
pub(in crate::user_tasks) unsafe fn exercise(space: &mut UserSpace, slot: usize, physical: &[u64]) {
    let base = wire::address(slot);
    let before = space.pages;
    // Refuse each possible collision, including a late conflict in the second
    // leaf table. A retry after removing only the injected leaf must succeed:
    // that would fail if the refused installation left even one new leaf behind.
    for collision in 0..physical.len() {
        let address = base + collision as u64 * PAGE;
        // SAFETY: Entire range is vacant; injected leaf borrows retained fixture RAM.
        unsafe {
            space
                .mapping
                .as_mut()
                .unwrap()
                .map_pages(address, &physical[..1], false)
        }
        .unwrap();
        assert_eq!(
            // SAFETY: Same private tree/backing custody; collision is deliberate.
            unsafe { space.map_region(slot, physical, true) },
            Err(MemoryError::OccupiedVirtualRange),
        );
        assert_eq!(space.pages, before);
        assert_eq!(space.backing.len(), before);
        assert!(!space.writable_range(base, 1));
        // SAFETY: Retire only the injected leaf. Then publish and retire a real
        // complete mapping through core, exercising metadata commit too.
        unsafe {
            space.mapping.as_mut().unwrap().unmap_pages(address, 1);
            space.map_region(slot, physical, true).unwrap();
            space.unmap_region(slot, physical.len());
        }
    }
    let mut malformed = physical.to_vec();
    // Aligned/nonzero, so this reaches ISA validation rather than core's shape
    // checks. It is outside both current CPU backends' physical address space.
    *malformed.last_mut().unwrap() = !(PAGE - 1);
    assert_eq!(
        // SAFETY: Invalid address is tested only as data; validation must reject it.
        unsafe { space.map_region(slot, &malformed, false) },
        Err(MemoryError::InvalidFrame),
    );
    assert_eq!(space.pages, before);
    assert_eq!(space.backing.len(), before);
    // SAFETY: Prove refusal left every leaf vacant, then restore the fixture.
    unsafe {
        space.map_region(slot, physical, false).unwrap();
        space.unmap_region(slot, physical.len());
    }
}
