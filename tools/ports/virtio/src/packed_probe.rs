// Cross-target measurements of actual pinned packed types.
#[used]
#[unsafe(no_mangle)]
pub static CATHEDRAL_VIRTIO_PACKED: [u64; 14] = [
    core::mem::size_of::<virtio_spec::pvirtq::Desc>() as u64, // Desc.size
    core::mem::align_of::<virtio_spec::pvirtq::Desc>() as u64, // Desc.alignment
    core::mem::offset_of!(virtio_spec::pvirtq::Desc, addr) as u64, // Desc.addr.offset
    core::mem::offset_of!(virtio_spec::pvirtq::Desc, len) as u64, // Desc.len.offset
    core::mem::offset_of!(virtio_spec::pvirtq::Desc, id) as u64, // Desc.id.offset
    core::mem::offset_of!(virtio_spec::pvirtq::Desc, flags) as u64, // Desc.flags.offset
    core::mem::size_of::<virtio_spec::pvirtq::EventSuppress>() as u64, // EventSuppress.size
    core::mem::align_of::<virtio_spec::pvirtq::EventSuppress>() as u64, // EventSuppress.alignment
    core::mem::offset_of!(virtio_spec::pvirtq::EventSuppress, desc) as u64, // EventSuppress.desc.offset
    core::mem::offset_of!(virtio_spec::pvirtq::EventSuppress, flags) as u64, // EventSuppress.flags.offset
    core::mem::size_of::<virtio_spec::pvirtq::EventSuppressDesc>() as u64, // EventSuppressDesc.size
    core::mem::align_of::<virtio_spec::pvirtq::EventSuppressDesc>() as u64, // EventSuppressDesc.alignment
    core::mem::size_of::<virtio_spec::pvirtq::EventSuppressFlags>() as u64, // EventSuppressFlags.size
    core::mem::align_of::<virtio_spec::pvirtq::EventSuppressFlags>() as u64, // EventSuppressFlags.alignment
];
