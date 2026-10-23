use super::*;
use std::vec;
#[test]
fn funded_extent_alignment_exhaustion_and_reuse() {
    let mut storage = vec![0usize; 32768];
    let heap = Heap::empty();
    let layout = Layout::from_size_align(1001, 4096).unwrap();
    // SAFETY: The vector outlives the heap and every allocation; only the heap
    // accesses its bytes. Each returned pointer is freed with its original layout.
    unsafe {
        assert!(heap.alloc(layout).is_null());
        heap.initialize(
            storage.as_mut_ptr().cast(),
            storage.len() * core::mem::size_of::<usize>(),
        )
        .unwrap();
        assert!(heap.initialize(storage.as_mut_ptr().cast(), 1024).is_err());
        let mut pointers = std::vec::Vec::new();
        loop {
            let ptr = heap.alloc(layout);
            if ptr.is_null() {
                break;
            }
            assert_eq!(ptr as usize % 4096, 0);
            ptr.write_bytes(0xab, layout.size());
            pointers.push(ptr);
        }
        assert!(pointers.len() > 16);
        for ptr in pointers.drain(..).rev() {
            heap.dealloc(ptr, layout);
        }
        assert_eq!(heap.stats().live, 0);
        let large = Layout::from_size_align(128 * 1024, 16).unwrap();
        let ptr = heap.alloc(large);
        assert!(!ptr.is_null(), "freed holes did not coalesce");
        heap.dealloc(ptr, large);
    }
}
#[test]
fn mixed_lifetimes_preserve_live_data_and_refuse_oversized_requests() {
    let mut storage = vec![0u64; 4096];
    let heap = Heap::empty();
    // SAFETY: Exclusively owned vector backing remains live throughout all operations.
    unsafe {
        heap.initialize(storage.as_mut_ptr().cast(), storage.len() * 8)
            .unwrap();
        let a = Layout::from_size_align(333, 16).unwrap();
        let b = Layout::from_size_align(8192, 64).unwrap();
        let first = heap.alloc(a);
        let second = heap.alloc(b);
        assert!(!first.is_null() && !second.is_null());
        second.write_bytes(0x72, b.size());
        heap.dealloc(first, a);
        let third = heap.alloc(a);
        assert!(!third.is_null());
        assert!(
            heap.alloc(Layout::from_size_align(65536, 8).unwrap())
                .is_null()
        );
        assert!(
            core::slice::from_raw_parts(second, b.size())
                .iter()
                .all(|b| *b == 0x72)
        );
        heap.dealloc(second, b);
        heap.dealloc(third, a);
        assert_eq!(heap.stats().live, 0);
        assert_eq!(heap.stats().failures, 1);
    }
}
#[test]
fn further_extents_take_overflow_and_frees_return_to_their_extent() {
    let mut first = vec![0u64; 2048];
    let mut second = vec![0u64; 2048];
    let heap = Heap::empty();
    let range = |extent: &std::vec::Vec<u64>| {
        extent.as_ptr() as usize..extent.as_ptr() as usize + extent.len() * 8
    };
    // SAFETY: Both vectors outlive the heap and every allocation; only the heap
    // accesses their bytes, and each pointer is freed with its original layout.
    unsafe {
        assert!(heap.extend(second.as_mut_ptr().cast(), 16384).is_err());
        heap.initialize(first.as_mut_ptr().cast(), first.len() * 8)
            .unwrap();
        assert!(heap.extend(first.as_mut_ptr().cast(), 1024).is_err());
        heap.extend(second.as_mut_ptr().cast(), second.len() * 8)
            .unwrap();
        assert_eq!(heap.stats().capacity, 32768);
        let big = Layout::from_size_align(12 * 1024, 16).unwrap();
        let a = heap.alloc(big);
        let b = heap.alloc(big);
        assert!(range(&first).contains(&(a as usize)));
        assert!(range(&second).contains(&(b as usize)));
        a.write_bytes(0xa5, big.size());
        b.write_bytes(0x5a, big.size());
        heap.dealloc(a, big);
        let c = heap.alloc(big);
        assert!(range(&first).contains(&(c as usize)));
        assert!(heap.alloc(big).is_null());
        assert!(
            core::slice::from_raw_parts(b, big.size())
                .iter()
                .all(|byte| *byte == 0x5a)
        );
        heap.dealloc(b, big);
        heap.dealloc(c, big);
        assert_eq!(heap.stats().live, 0);
        assert_eq!(heap.stats().failures, 1);
        let mut spares: std::vec::Vec<std::vec::Vec<u64>> =
            (0..3).map(|_| vec![0u64; 64]).collect();
        for spare in spares.iter_mut().take(MAX_EXTENTS - 2) {
            heap.extend(spare.as_mut_ptr().cast(), 512).unwrap();
        }
        assert!(heap.extend(spares[2].as_mut_ptr().cast(), 512).is_err());
    }
}
