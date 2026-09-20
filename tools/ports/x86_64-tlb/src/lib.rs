#![no_std]
// Actual pinned Pcid type/constructors are available on the UEFI-x64 target.
#[cfg(target_arch="x86_64")]
const _: () = {
 use x86_64::instructions::tlb::Pcid;
 assert!(core::mem::size_of::<Pcid>()==2);assert!(core::mem::align_of::<Pcid>()==2);
 assert!(Pcid::new(0).is_ok());assert!(Pcid::new(4095).is_ok());assert!(Pcid::new(4096).is_err());assert!(Pcid::new(65535).is_err());

};
