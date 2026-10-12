use super::*;
use cathedral_contracts::boot::MemoryRegion;

#[test]
fn allocations_skip_reserved_and_null_frames_and_exhaust_without_reuse() {
    let mut map = BootMemory::new();
    for (base, page_count, kind) in [
        (0, 2, MemoryKind::Usable),
        (8192, 2, MemoryKind::Reserved),
        (16384, 2, MemoryKind::Usable),
    ] {
        map.push(MemoryRegion {
            base,
            page_count,
            kind,
        })
        .unwrap();
    }
    let mut allocator = FrameAllocator::new(map);
    for expected in [4096, 16384, 20480] {
        assert_eq!(allocator.allocate().unwrap().address(), expected);
    }
    assert!(allocator.allocate().is_none());
    assert!(allocator.allocate().is_none());
}

#[test]
fn empty_inventory_is_exhausted() {
    assert!(FrameAllocator::new(BootMemory::new()).allocate().is_none());
}

#[test]
fn reclaim_reuses_frames_but_rejects_boot_reserved_null_and_double_free() {
    let mut map = BootMemory::new();
    // Deliberately unsorted, crossing a bitmap word boundary.
    for (base, page_count, kind) in [
        (0x100000, 65, MemoryKind::Usable),
        (0, 3, MemoryKind::Usable),
        (0x3000, 5, MemoryKind::Reserved),
    ] {
        map.push(MemoryRegion {
            base,
            page_count,
            kind,
        })
        .unwrap();
    }
    let mut allocator = FrameAllocator::new(map);
    let boot = allocator.allocate().unwrap();
    allocator.enable_reclamation();
    let mut frames = Vec::new();
    while let Some(frame) = allocator.allocate() {
        frames.push(frame);
    }
    assert_eq!(frames.len(), 66);
    assert_eq!(allocator.release(boot), Err(ReleaseError::PermanentFrame));
    for bad in [0, 0x3000, 0x100001] {
        assert_eq!(
            allocator.release_address(bad),
            Err(ReleaseError::InvalidFrame)
        );
    }
    let frame = frames.pop().unwrap();
    let address = frame.address();
    allocator.release(frame).unwrap();
    assert_eq!(
        allocator.release_address(address),
        Err(ReleaseError::AlreadyFree)
    );
    assert_eq!(allocator.allocate().unwrap().address(), address);
    assert!(allocator.allocate().is_none());
    for frame in frames {
        allocator.release(frame).unwrap();
    }
    assert_eq!(allocator.allocated(), 2);
}
