//! QEMU/VMware virtual mouse transport only. Driver sequencing stays in userspace.
//! Protocol reference: QEMU hw/i386/vmmouse.c and hw/i386/vmport.c.
use crate::board::DeviceError;
pub fn valid(command: u64, data: u64) -> bool {
    matches!(
        (command, data),
        (10, 0) | (39, 1 | 4) | (40, 0) | (41, 0xf5 | 0x45414552 | 0x53424152)
    )
}
/// # Safety
/// Boot CPU, IRQs masked, exclusive input grant. No arbitrary hypervisor commands.
pub unsafe fn exchange(command: u64, data: u64) -> Result<[u32; 4], DeviceError> {
    if !valid(command, data) {
        return Err(DeviceError::InvalidArgument);
    }
    // Avoid probing the port on physical PCs or unrelated hypervisors.
    let features = core::arch::x86_64::__cpuid(1);
    if features.ecx & (1 << 31) == 0 {
        return Err(DeviceError::InvalidArgument);
    }
    let vendor = core::arch::x86_64::__cpuid(0x40000000);
    let mut name = [0u8; 12];
    name[..4].copy_from_slice(&vendor.ebx.to_le_bytes());
    name[4..8].copy_from_slice(&vendor.ecx.to_le_bytes());
    name[8..].copy_from_slice(&vendor.edx.to_le_bytes());
    if &name != b"TCGTCGTCGTCG" && &name != b"VMwareVMware" {
        return Err(DeviceError::InvalidArgument);
    }
    let (a, b, c, d): (u32, u32, u32, u32);
    // SAFETY: Fixed port and whitelisted input commands. Save LLVM's reserved
    // RBX, explicitly capture all four reply registers, and restore the stack.
    unsafe {
        core::arch::asm!(
            "push rbx", "mov ebx, r8d", "in eax, dx", "mov r8d, ebx", "pop rbx",
            inout("eax") 0x564d5868u32 => a,
            inout("r8d") data as u32 => b,
            inout("ecx") command as u32 => c,
            inout("edx") 0x5658u32 => d,
            options(preserves_flags),
        );
    }
    Ok([a, b, c, d])
}
#[cfg(test)]
mod tests {
    use super::valid;
    #[test]
    fn only_pointer_operations_are_exposed() {
        for command in 0..256 {
            assert_eq!(valid(command, 0), matches!(command, 10 | 40));
        }
        assert!(valid(39, 4));
        assert!(valid(41, 0x53424152));
        for (c, d) in [
            (39, 0),
            (39, 6),
            (41, 0),
            (10, 1),
            (40, 1),
            (0x1000a, 0),
            (41, 0x153424152),
        ] {
            assert!(!valid(c, d));
        }
    }
}
