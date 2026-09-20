// Data-only execution of pinned allocation test behavior. No device mappings.
use std::alloc::Layout;
use virtio_spec::virtq::{Avail, Used};
fn main() {
    packed_bitfields();
    wide_word_order();
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

fn packed_bitfields() {
    use virtio_spec::{pvirtq::{EventSuppressDesc, EventSuppressFlags}, pci::NotificationData, RingEventFlags, le16, le32};
    for offset in [0u16, 1, 255, 32767] {
        for wrap in [0u8, 1] {
            let desc = EventSuppressDesc::new().with_desc_event_off(offset).with_desc_event_wrap(wrap);
            let raw: le16 = desc.into();
            assert_eq!(raw.to_ne(), offset | ((wrap as u16) << 15));
            assert_eq!(desc.desc_event_off(), offset);
            assert_eq!(desc.desc_event_wrap(), wrap);
            let notification = NotificationData::new().with_vq_notif_config_data(0x1234).with_next_off(offset).with_next_wrap(wrap);
            let data: le32 = notification.into();
            assert_eq!(data.to_ne(), 0x1234 | ((offset as u32) << 16) | ((wrap as u32) << 31));
        }
    }
    for (mode, code) in [(RingEventFlags::Enable,0), (RingEventFlags::Disable,1), (RingEventFlags::Desc,2), (RingEventFlags::Reserved,3)] {
        let flags = EventSuppressFlags::new().with_desc_event_flags(mode).with_reserved(0x3fff);
        let raw: le16 = flags.into();
        assert_eq!(raw.to_ne(), 0xfffc | code);
        assert_eq!(flags.reserved(), 0x3fff);
    }
    println!("Pinned Rust: packed 15+1-bit offsets/wrap, 2+14-bit flags, and notification encodings passed.");
}

fn wide_word_order() {
    use virtio_spec::{le32,le64,be32,be64};
    let little = le64::from([le32::from_ne(0x89abcdef),le32::from_ne(0x12345678)]);
    let big = be64::from([be32::from_ne(0x89abcdef),be32::from_ne(0x12345678)]);
    assert_eq!(little.to_ne(), 0x1234567889abcdef);
    assert_eq!(big.to_ne(), 0x89abcdef12345678);
    let little_words: [le32;2] = little.into();
    let big_words: [be32;2] = big.into();
    assert_eq!(little_words[0].to_ne(),0x89abcdef);
    assert_eq!(big_words[0].to_ne(),0x89abcdef);
    println!("Pinned Rust: wide LE/BE first/second word order passed.");
}
