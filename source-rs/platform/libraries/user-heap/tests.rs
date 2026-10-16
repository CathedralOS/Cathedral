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
