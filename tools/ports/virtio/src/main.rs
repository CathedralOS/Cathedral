// Data-only execution of pinned allocation test behavior. No device mappings.
use std::alloc::Layout;
use virtio_spec::virtq::{Avail, Used};
fn main() {
    for q in [0u16, 255, 256, 257, 65535] {
        for event in [false, true] {
            let avail = Avail::new(q, event);
            let used = Used::new(q, event);
            let n = q as usize;
            assert_eq!(Layout::for_value(&*avail).size(), 4 + 2*n + if event {2} else {0});
            assert_eq!(Layout::for_value(&*avail).align(), 2);
            assert_eq!(Layout::for_value(&*used).size(), 4 + 8*n + if event {4} else {0});
            assert_eq!(Layout::for_value(&*used).align(), 4);
            assert_eq!(avail.ring(event).len(), n);
            assert_eq!(used.ring().len(), n);
            assert!(avail.ring(event).iter().all(|x| x.to_ne() == 0));
            assert!(used.ring().iter().all(|x| x.id.to_ne() == 0 && x.len.to_ne() == 0));
            assert_eq!(avail.used_event(event).is_some(), event);
            assert_eq!(used.avail_event().is_some(), event);
        }
    }
    println!("Pinned Rust: 20 split-ring allocations/layouts, zeroed elements, optional event words passed.");
}
