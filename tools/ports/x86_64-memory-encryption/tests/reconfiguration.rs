// SPDX-License-Identifier: MIT OR Apache-2.0
use x86_64::{PhysAddr,structures::{mem_encrypt::{enable_memory_encryption,MemoryEncryptionConfiguration as Config},paging::{page_table::PageTableEntry,PageTableFlags}}};
#[test]
fn previous_bit_is_physical_but_stays_out_of_pte_address() {
    // No PTE/address values or live mappings exist across these configurations.
    unsafe {
        enable_memory_encryption(Config::EncryptedBit(47));
        enable_memory_encryption(Config::SharedBit(48));
        enable_memory_encryption(Config::EncryptedBit(47));
    }
    let old=1u64<<48;
    assert_eq!(PhysAddr::try_new(old).unwrap().as_u64(),old);
    assert_eq!(PhysAddr::new_truncate(u64::MAX).as_u64(),((1u64<<52)-1)&!(1u64<<47));
    let mut entry=PageTableEntry::new();entry.set_addr(PhysAddr::new(old),PageTableFlags::PRESENT);
    assert_eq!(entry.addr().as_u64(),0);
    assert_eq!(entry.flags().bits(),old|1);
    assert_eq!(entry.frame().unwrap().start_address().as_u64(),0);
    entry.set_flags(PageTableFlags::PRESENT|PageTableFlags::WRITABLE);
    assert_eq!(entry.addr().as_u64(),0);assert_eq!(entry.flags().bits(),3);
    entry.set_addr(PhysAddr::new(0x12345000),PageTableFlags::PRESENT|PageTableFlags::WRITABLE);
    assert_eq!(entry.addr().as_u64(),0x12345000);assert_eq!(entry.flags().bits(),3);
}
