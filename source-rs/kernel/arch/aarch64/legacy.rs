//! The virt proof has no PC PS/2 or ATA adapter. Composition rejects those grants.
//! Empty keyboard polling is needed by the shared event loop; hardware requests cannot reach here.
/// # Safety
/// Same IRQ-masked exclusive device contract as the architecture interface.
/// This board has no PC devices; admission must reject such grants.
pub unsafe fn keyboard_irq(enabled: bool) {
    assert!(!enabled, "no PS/2 on virt");
}
/// # Safety
/// Same IRQ-masked exclusive device contract as the architecture interface.
/// This board has no PC devices; admission must reject such grants.
pub unsafe fn keyboard_read() -> Option<(u8, u8)> {
    None
}
/// # Safety
/// Same IRQ-masked exclusive device contract as the architecture interface.
/// This board has no PC devices; admission must reject such grants.
pub unsafe fn keyboard_write(_: bool, _: u8) -> bool {
    panic!("PS/2 grant rejected at admission")
}
/// # Safety
/// Same IRQ-masked exclusive device contract as the architecture interface.
/// This board has no PC devices; admission must reject such grants.
pub unsafe fn disk_read(_: u8) -> u8 {
    panic!("ATA grant rejected at admission")
}
/// # Safety
/// Same IRQ-masked exclusive device contract as the architecture interface.
/// This board has no PC devices; admission must reject such grants.
pub unsafe fn disk_write(_: u8, _: u8) {
    panic!("ATA grant rejected at admission")
}
/// # Safety
/// Same IRQ-masked exclusive device contract as the architecture interface.
/// This board has no PC devices; admission must reject such grants.
pub unsafe fn disk_transfer(_: &mut [u8; 512], _: bool) {
    panic!("ATA grant rejected at admission")
}
