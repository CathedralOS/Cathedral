//! Contrast exact checked software spans with the hardware page boundary.
use cathedral_rendering_lab::{Error, pixels, rows::Row};
use cathedral_user_runtime::memory::{Private, raw};
pub(super) fn rows() {
    let mut page = Private::allocate(1).unwrap();
    page.bytes_mut()[128] = 0xaa;
    {
        let mut lease = Row::new(&mut page.bytes_mut()[..128]).unwrap();
        assert_eq!(lease.put(32, 0), Err(Error::Bounds));
    }
    assert_eq!(page.bytes_mut()[128], 0xaa);
    // SAFETY: This raw store is inside our actual writable page, with no live
    // Rust borrows. It deliberately ignores the advertised 128-byte row span.
    // Hardware cannot distinguish this writer from its cooperating Rust wrapper.
    unsafe {
        page.bytes_mut().as_mut_ptr().add(128).write_volatile(0xbb);
    }
    assert_eq!(page.bytes_mut()[128], 0xbb);
    assert_eq!(
        pixels::direct(page.bytes_mut(), 64, 16, 4, Some(8)),
        Err(Error::Incomplete)
    );
    // No publication/notification on failure; the private page can be redrawn.
    assert_eq!(pixels::direct(page.bytes_mut(), 64, 16, 4, None), Ok(4096));
    assert!(pixels::verify(page.bytes_mut(), 64, 16));
}
pub(crate) fn guard_fault() -> u64 {
    let handle = raw::allocate(1).unwrap();
    let forbidden = raw::address(handle).unwrap() + 4096;
    // SAFETY: Deliberate ring-3 store to an unmapped adjacent page, forming no
    // Rust reference. Supervisor must observe a contained user page fault.
    unsafe {
        core::arch::asm!("mov byte ptr [{0}], 1", in(reg) forbidden);
    }
    panic!("page boundary unexpectedly writable")
}
