// SPDX-License-Identifier: MIT OR Apache-2.0
// Calls actual public pinned GDT APIs; never loads a table or segment selector.
use std::panic::{catch_unwind, AssertUnwindSafe};
use x86_64::structures::gdt::{Descriptor, GlobalDescriptorTable};
fn sample<const N: usize>(name: &str, initial: &[u64], actions: &[(u64, Option<u64>)]) {
    let mut table = if initial.is_empty() { GlobalDescriptorTable::<N>::empty() }
        else { GlobalDescriptorTable::<N>::from_raw_entries(initial) };
    print!("{{\"name\":{name:?},\"capacity\":{N},\"initial\":{initial:?},\"actions\":[");
    for (i, &(low, high)) in actions.iter().enumerate() {
        let before: Vec<_> = table.entries().iter().map(|entry| entry.raw()).collect();
        let descriptor = match high { Some(high) => Descriptor::SystemSegment(low, high), None => Descriptor::UserSegment(low) };
        let result = catch_unwind(AssertUnwindSafe(||table.append(descriptor).0));
        if i != 0 { print!(","); }
        let outcome = match result { Ok(selector) => selector as i32, Err(_) => -1 };
        let after: Vec<_> = table.entries().iter().map(|entry| entry.raw()).collect();
        if outcome == -1 { assert_eq!(before, after, "failed append modified entries"); }
        let hi = high.map_or("null".to_owned(), |value|value.to_string());
        print!("{{\"low\":{low},\"high\":{hi},\"selector\":{outcome},\"length\":{},\"limit\":{},\"words\":{after:?}}}",after.len(),table.limit());
    }
    let words: Vec<_> = table.entries().iter().map(|entry|entry.raw()).collect();
    println!("],\"length\":{},\"limit\":{},\"words\":{words:?}}}", words.len(), table.limit());
}
fn main() {
    std::panic::set_hook(Box::new(|_|{}));
    sample::<1>("null_only", &[], &[(7,None),(8,Some(9))]);
    sample::<2>("single_slot", &[], &[(7,Some(9)),(7,None),(9,None)]);
    sample::<3>("system_fits", &[], &[(0x600000000005,Some(u64::MAX)),(9,None)]);
    sample::<8>("mixed", &[], &[(0x00af9b000000ffff,None),(0x00cff3000000ffff,None),(0x200000001234,Some(0xdeadbeef)),(u64::MAX,None),(0,Some(0xfeed)),(0,None)]);
    sample::<8>("import", &[0,u64::MAX,0,0x1234], &[(0x400000000055,Some(0x0102030405060708)),(99,None),(100,Some(101)),(100,None)]);
    sample::<8>("full_import", &[0,1,2,3,4,5,6,7], &[(42,None),(43,Some(44))]);
    sample::<8192>("maximum", &[0], &[(1,Some(2))]);
    let mut almost_full = vec![0;8190]; almost_full[8189]=0xf00d;
    sample::<8192>("last_system", &almost_full, &[(0x6000000000aa,Some(0xfeedface)),(7,None)]);
    almost_full.push(0xabcd);
    sample::<8192>("last_user", &almost_full, &[(9,Some(10)),(0x600000000001,None)]);
    let mut full = vec![0;8192]; full[4095]=0x1234; full[4096]=0x5678; full[8191]=0x9abc;
    sample::<8192>("full_maximum", &full, &[(17,Some(18))]);
    for length in [15,16,17,31,32,33] {
        let mut entries: Vec<u64> = (0..length).map(|i| (i as u64)*0x01020304050607).collect(); entries[0]=0;
        sample::<64>(&format!("block_{length}"), &entries, &[(0xaa,Some(0xbb))]);
    }
    let default = GlobalDescriptorTable::default();
    assert_eq!(default.entries().len(),1); assert_eq!(default.entries()[0].raw(),0); assert_eq!(default.limit(),7);
    for input in [&[][..],&[1][..],&[0,1,2][..]] {
        assert!(catch_unwind(||GlobalDescriptorTable::<2>::from_raw_entries(input)).is_err());
    }
    assert!(catch_unwind(GlobalDescriptorTable::<0>::empty).is_err());
    assert!(catch_unwind(GlobalDescriptorTable::<8193>::empty).is_err());
    eprintln!("PASS 16 actual GDT scenarios, default initialization and 5 invalid constructions");
}
