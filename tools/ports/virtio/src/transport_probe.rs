// Actual pinned transport types, cross-target LLVM constants.
#[used]
#[unsafe(no_mangle)]
pub static CATHEDRAL_VIRTIO_TRANSPORT: [u64; 28] = [
    core::mem::size_of::<virtio_spec::pci::Cap>() as u64, // Cap.size
    core::mem::align_of::<virtio_spec::pci::Cap>() as u64, // Cap.alignment
    core::mem::offset_of!(virtio_spec::pci::Cap, cap_vndr) as u64, // Cap.cap_vndr.offset
    core::mem::offset_of!(virtio_spec::pci::Cap, cap_next) as u64, // Cap.cap_next.offset
    core::mem::offset_of!(virtio_spec::pci::Cap, cap_len) as u64, // Cap.cap_len.offset
    core::mem::offset_of!(virtio_spec::pci::Cap, cfg_type) as u64, // Cap.cfg_type.offset
    core::mem::offset_of!(virtio_spec::pci::Cap, bar) as u64, // Cap.bar.offset
    core::mem::offset_of!(virtio_spec::pci::Cap, id) as u64, // Cap.id.offset
    core::mem::offset_of!(virtio_spec::pci::Cap, padding) as u64, // Cap.padding.offset
    core::mem::offset_of!(virtio_spec::pci::Cap, offset) as u64, // Cap.offset.offset
    core::mem::offset_of!(virtio_spec::pci::Cap, length) as u64, // Cap.length.offset
    core::mem::size_of::<virtio_spec::pci::Cap64>() as u64, // Cap64.size
    core::mem::align_of::<virtio_spec::pci::Cap64>() as u64, // Cap64.alignment
    core::mem::offset_of!(virtio_spec::pci::Cap64, cap) as u64, // Cap64.cap.offset
    core::mem::offset_of!(virtio_spec::pci::Cap64, offset_hi) as u64, // Cap64.offset_hi.offset
    core::mem::offset_of!(virtio_spec::pci::Cap64, length_hi) as u64, // Cap64.length_hi.offset
    core::mem::size_of::<virtio_spec::pci::NotifyCap>() as u64, // NotifyCap.size
    core::mem::align_of::<virtio_spec::pci::NotifyCap>() as u64, // NotifyCap.alignment
    core::mem::offset_of!(virtio_spec::pci::NotifyCap, cap) as u64, // NotifyCap.cap.offset
    core::mem::offset_of!(virtio_spec::pci::NotifyCap, notify_off_multiplier) as u64, // NotifyCap.notify_off_multiplier.offset
    core::mem::size_of::<virtio_spec::pci::CfgCap>() as u64, // CfgCap.size
    core::mem::align_of::<virtio_spec::pci::CfgCap>() as u64, // CfgCap.alignment
    core::mem::offset_of!(virtio_spec::pci::CfgCap, cap) as u64, // CfgCap.cap.offset
    core::mem::offset_of!(virtio_spec::pci::CfgCap, pci_cfg_data) as u64, // CfgCap.pci_cfg_data.offset
    core::mem::size_of::<virtio_spec::pci::CapCfgType>() as u64, // CapCfgType.size
    core::mem::align_of::<virtio_spec::pci::CapCfgType>() as u64, // CapCfgType.alignment
    core::mem::size_of::<virtio_spec::mmio::DeviceRegisters>() as u64, // DeviceRegisters.size
    core::mem::align_of::<virtio_spec::mmio::DeviceRegisters>() as u64, // DeviceRegisters.alignment
];
