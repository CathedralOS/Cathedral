#![no_std]
// Pinned upstream representation measurements; no device or Omega ABI access.
use core::mem::{size_of, align_of, offset_of};
#[used]
#[unsafe(no_mangle)]
pub static CATHEDRAL_VIRTIO_LAYOUT: [u64; 255] = [
    size_of::<virtio_spec::balloon::Config>() as u64, // balloon.Config.size
    align_of::<virtio_spec::balloon::Config>() as u64, // balloon.Config.alignment
    size_of::<virtio_spec::blk::Geometry>() as u64, // blk.Geometry.size
    align_of::<virtio_spec::blk::Geometry>() as u64, // blk.Geometry.alignment
    size_of::<virtio_spec::blk::Topology>() as u64, // blk.Topology.size
    align_of::<virtio_spec::blk::Topology>() as u64, // blk.Topology.alignment
    size_of::<virtio_spec::blk::ZonedCharacteristics>() as u64, // blk.ZonedCharacteristics.size
    align_of::<virtio_spec::blk::ZonedCharacteristics>() as u64, // blk.ZonedCharacteristics.alignment
    size_of::<virtio_spec::blk::Config>() as u64, // blk.Config.size
    align_of::<virtio_spec::blk::Config>() as u64, // blk.Config.alignment
    size_of::<virtio_spec::blk::DiscardWriteZeroes>() as u64, // blk.DiscardWriteZeroes.size
    align_of::<virtio_spec::blk::DiscardWriteZeroes>() as u64, // blk.DiscardWriteZeroes.alignment
    offset_of!(virtio_spec::blk::DiscardWriteZeroes, sector) as u64, // blk.DiscardWriteZeroes.sector.offset
    offset_of!(virtio_spec::blk::DiscardWriteZeroes, num_sectors) as u64, // blk.DiscardWriteZeroes.num_sectors.offset
    offset_of!(virtio_spec::blk::DiscardWriteZeroes, flags) as u64, // blk.DiscardWriteZeroes.flags.offset
    size_of::<virtio_spec::blk::Lifetime>() as u64, // blk.Lifetime.size
    align_of::<virtio_spec::blk::Lifetime>() as u64, // blk.Lifetime.alignment
    offset_of!(virtio_spec::blk::Lifetime, pre_eol_info) as u64, // blk.Lifetime.pre_eol_info.offset
    offset_of!(virtio_spec::blk::Lifetime, device_lifetime_est_typ_a) as u64, // blk.Lifetime.device_lifetime_est_typ_a.offset
    offset_of!(virtio_spec::blk::Lifetime, device_lifetime_est_typ_b) as u64, // blk.Lifetime.device_lifetime_est_typ_b.offset
    size_of::<virtio_spec::blk::ZoneDescriptor>() as u64, // blk.ZoneDescriptor.size
    align_of::<virtio_spec::blk::ZoneDescriptor>() as u64, // blk.ZoneDescriptor.alignment
    size_of::<virtio_spec::console::Config>() as u64, // console.Config.size
    align_of::<virtio_spec::console::Config>() as u64, // console.Config.alignment
    size_of::<virtio_spec::console::Control>() as u64, // console.Control.size
    align_of::<virtio_spec::console::Control>() as u64, // console.Control.alignment
    offset_of!(virtio_spec::console::Control, id) as u64, // console.Control.id.offset
    offset_of!(virtio_spec::console::Control, event) as u64, // console.Control.event.offset
    offset_of!(virtio_spec::console::Control, value) as u64, // console.Control.value.offset
    size_of::<virtio_spec::console::Resize>() as u64, // console.Resize.size
    align_of::<virtio_spec::console::Resize>() as u64, // console.Resize.alignment
    offset_of!(virtio_spec::console::Resize, cols) as u64, // console.Resize.cols.offset
    offset_of!(virtio_spec::console::Resize, rows) as u64, // console.Resize.rows.offset
    size_of::<virtio_spec::fs::Config>() as u64, // fs.Config.size
    align_of::<virtio_spec::fs::Config>() as u64, // fs.Config.alignment
    size_of::<virtio_spec::net::Config>() as u64, // net.Config.size
    align_of::<virtio_spec::net::Config>() as u64, // net.Config.alignment
    size_of::<virtio_spec::net::Hdr>() as u64, // net.Hdr.size
    align_of::<virtio_spec::net::Hdr>() as u64, // net.Hdr.alignment
    offset_of!(virtio_spec::net::Hdr, flags) as u64, // net.Hdr.flags.offset
    offset_of!(virtio_spec::net::Hdr, gso_type) as u64, // net.Hdr.gso_type.offset
    offset_of!(virtio_spec::net::Hdr, hdr_len) as u64, // net.Hdr.hdr_len.offset
    offset_of!(virtio_spec::net::Hdr, gso_size) as u64, // net.Hdr.gso_size.offset
    offset_of!(virtio_spec::net::Hdr, csum_start) as u64, // net.Hdr.csum_start.offset
    offset_of!(virtio_spec::net::Hdr, csum_offset) as u64, // net.Hdr.csum_offset.offset
    offset_of!(virtio_spec::net::Hdr, num_buffers) as u64, // net.Hdr.num_buffers.offset
    size_of::<virtio_spec::net::HdrHash>() as u64, // net.HdrHash.size
    align_of::<virtio_spec::net::HdrHash>() as u64, // net.HdrHash.alignment
    offset_of!(virtio_spec::net::HdrHash, hdr) as u64, // net.HdrHash.hdr.offset
    offset_of!(virtio_spec::net::HdrHash, hash_value) as u64, // net.HdrHash.hash_value.offset
    offset_of!(virtio_spec::net::HdrHash, hash_report) as u64, // net.HdrHash.hash_report.offset
    offset_of!(virtio_spec::net::HdrHash, padding_reserved) as u64, // net.HdrHash.padding_reserved.offset
    size_of::<virtio_spec::net::HdrHashTunnel>() as u64, // net.HdrHashTunnel.size
    align_of::<virtio_spec::net::HdrHashTunnel>() as u64, // net.HdrHashTunnel.alignment
    offset_of!(virtio_spec::net::HdrHashTunnel, hash_hdr) as u64, // net.HdrHashTunnel.hash_hdr.offset
    offset_of!(virtio_spec::net::HdrHashTunnel, outer_th_offset) as u64, // net.HdrHashTunnel.outer_th_offset.offset
    offset_of!(virtio_spec::net::HdrHashTunnel, inner_nh_offset) as u64, // net.HdrHashTunnel.inner_nh_offset.offset
    size_of::<virtio_spec::net::HdrHashTunnelOutNetHdr>() as u64, // net.HdrHashTunnelOutNetHdr.size
    align_of::<virtio_spec::net::HdrHashTunnelOutNetHdr>() as u64, // net.HdrHashTunnelOutNetHdr.alignment
    offset_of!(virtio_spec::net::HdrHashTunnelOutNetHdr, outer_nh_offset) as u64, // net.HdrHashTunnelOutNetHdr.outer_nh_offset.offset
    offset_of!(virtio_spec::net::HdrHashTunnelOutNetHdr, padding_reserved_2) as u64, // net.HdrHashTunnelOutNetHdr.padding_reserved_2.offset
    size_of::<virtio_spec::vsock::Config>() as u64, // vsock.Config.size
    align_of::<virtio_spec::vsock::Config>() as u64, // vsock.Config.alignment
    size_of::<virtio_spec::vsock::Hdr>() as u64, // vsock.Hdr.size
    align_of::<virtio_spec::vsock::Hdr>() as u64, // vsock.Hdr.alignment
    offset_of!(virtio_spec::vsock::Hdr, src_cid) as u64, // vsock.Hdr.src_cid.offset
    offset_of!(virtio_spec::vsock::Hdr, dst_cid) as u64, // vsock.Hdr.dst_cid.offset
    offset_of!(virtio_spec::vsock::Hdr, src_port) as u64, // vsock.Hdr.src_port.offset
    offset_of!(virtio_spec::vsock::Hdr, dst_port) as u64, // vsock.Hdr.dst_port.offset
    offset_of!(virtio_spec::vsock::Hdr, len) as u64, // vsock.Hdr.len.offset
    offset_of!(virtio_spec::vsock::Hdr, type_) as u64, // vsock.Hdr.type_.offset
    offset_of!(virtio_spec::vsock::Hdr, op) as u64, // vsock.Hdr.op.offset
    offset_of!(virtio_spec::vsock::Hdr, flags) as u64, // vsock.Hdr.flags.offset
    offset_of!(virtio_spec::vsock::Hdr, buf_alloc) as u64, // vsock.Hdr.buf_alloc.offset
    offset_of!(virtio_spec::vsock::Hdr, fwd_cnt) as u64, // vsock.Hdr.fwd_cnt.offset
    size_of::<virtio_spec::vsock::Event>() as u64, // vsock.Event.size
    align_of::<virtio_spec::vsock::Event>() as u64, // vsock.Event.alignment
    size_of::<virtio_spec::pci::CommonCfg>() as u64, // configuration.CommonCfg.size
    align_of::<virtio_spec::pci::CommonCfg>() as u64, // configuration.CommonCfg.alignment
    size_of::<virtio_spec::virtq::Desc>() as u64, // split.Desc.size
    align_of::<virtio_spec::virtq::Desc>() as u64, // split.Desc.alignment
    offset_of!(virtio_spec::virtq::Desc, addr) as u64, // split.Desc.addr.offset
    offset_of!(virtio_spec::virtq::Desc, len) as u64, // split.Desc.len.offset
    offset_of!(virtio_spec::virtq::Desc, flags) as u64, // split.Desc.flags.offset
    offset_of!(virtio_spec::virtq::Desc, next) as u64, // split.Desc.next.offset
    size_of::<virtio_spec::virtq::UsedElem>() as u64, // split.UsedElem.size
    align_of::<virtio_spec::virtq::UsedElem>() as u64, // split.UsedElem.alignment
    offset_of!(virtio_spec::virtq::UsedElem, id) as u64, // split.UsedElem.id.offset
    offset_of!(virtio_spec::virtq::UsedElem, len) as u64, // split.UsedElem.len.offset
    size_of::<virtio_spec::F>() as u64, // features.Features.size
    align_of::<virtio_spec::F>() as u64, // features.Features.alignment
    (virtio_spec::F::INDIRECT_DESC.bits().to_ne() >> 0) as u64 as u64, // features.COMMON_INDIRECT_DESC.low
    (virtio_spec::F::INDIRECT_DESC.bits().to_ne() >> 64) as u64 as u64, // features.COMMON_INDIRECT_DESC.high
    (virtio_spec::F::EVENT_IDX.bits().to_ne() >> 0) as u64 as u64, // features.COMMON_EVENT_IDX.low
    (virtio_spec::F::EVENT_IDX.bits().to_ne() >> 64) as u64 as u64, // features.COMMON_EVENT_IDX.high
    (virtio_spec::F::VERSION_1.bits().to_ne() >> 0) as u64 as u64, // features.COMMON_VERSION_1.low
    (virtio_spec::F::VERSION_1.bits().to_ne() >> 64) as u64 as u64, // features.COMMON_VERSION_1.high
    (virtio_spec::F::ACCESS_PLATFORM.bits().to_ne() >> 0) as u64 as u64, // features.COMMON_ACCESS_PLATFORM.low
    (virtio_spec::F::ACCESS_PLATFORM.bits().to_ne() >> 64) as u64 as u64, // features.COMMON_ACCESS_PLATFORM.high
    (virtio_spec::F::RING_PACKED.bits().to_ne() >> 0) as u64 as u64, // features.COMMON_RING_PACKED.low
    (virtio_spec::F::RING_PACKED.bits().to_ne() >> 64) as u64 as u64, // features.COMMON_RING_PACKED.high
    (virtio_spec::F::IN_ORDER.bits().to_ne() >> 0) as u64 as u64, // features.COMMON_IN_ORDER.low
    (virtio_spec::F::IN_ORDER.bits().to_ne() >> 64) as u64 as u64, // features.COMMON_IN_ORDER.high
    (virtio_spec::F::ORDER_PLATFORM.bits().to_ne() >> 0) as u64 as u64, // features.COMMON_ORDER_PLATFORM.low
    (virtio_spec::F::ORDER_PLATFORM.bits().to_ne() >> 64) as u64 as u64, // features.COMMON_ORDER_PLATFORM.high
    (virtio_spec::F::SR_IOV.bits().to_ne() >> 0) as u64 as u64, // features.COMMON_SR_IOV.low
    (virtio_spec::F::SR_IOV.bits().to_ne() >> 64) as u64 as u64, // features.COMMON_SR_IOV.high
    (virtio_spec::F::NOTIFICATION_DATA.bits().to_ne() >> 0) as u64 as u64, // features.COMMON_NOTIFICATION_DATA.low
    (virtio_spec::F::NOTIFICATION_DATA.bits().to_ne() >> 64) as u64 as u64, // features.COMMON_NOTIFICATION_DATA.high
    (virtio_spec::F::NOTIF_CONFIG_DATA.bits().to_ne() >> 0) as u64 as u64, // features.COMMON_NOTIF_CONFIG_DATA.low
    (virtio_spec::F::NOTIF_CONFIG_DATA.bits().to_ne() >> 64) as u64 as u64, // features.COMMON_NOTIF_CONFIG_DATA.high
    (virtio_spec::F::RING_RESET.bits().to_ne() >> 0) as u64 as u64, // features.COMMON_RING_RESET.low
    (virtio_spec::F::RING_RESET.bits().to_ne() >> 64) as u64 as u64, // features.COMMON_RING_RESET.high
    (virtio_spec::F::ADMIN_VQ.bits().to_ne() >> 0) as u64 as u64, // features.COMMON_ADMIN_VQ.low
    (virtio_spec::F::ADMIN_VQ.bits().to_ne() >> 64) as u64 as u64, // features.COMMON_ADMIN_VQ.high
    (virtio_spec::F::SUSPEND.bits().to_ne() >> 0) as u64 as u64, // features.COMMON_SUSPEND.low
    (virtio_spec::F::SUSPEND.bits().to_ne() >> 64) as u64 as u64, // features.COMMON_SUSPEND.high
    (virtio_spec::console::F::SIZE.bits().to_ne() >> 0) as u64 as u64, // features.CONSOLE_SIZE.low
    (virtio_spec::console::F::SIZE.bits().to_ne() >> 64) as u64 as u64, // features.CONSOLE_SIZE.high
    (virtio_spec::console::F::MULTIPORT.bits().to_ne() >> 0) as u64 as u64, // features.CONSOLE_MULTIPORT.low
    (virtio_spec::console::F::MULTIPORT.bits().to_ne() >> 64) as u64 as u64, // features.CONSOLE_MULTIPORT.high
    (virtio_spec::console::F::EMERG_WRITE.bits().to_ne() >> 0) as u64 as u64, // features.CONSOLE_EMERG_WRITE.low
    (virtio_spec::console::F::EMERG_WRITE.bits().to_ne() >> 64) as u64 as u64, // features.CONSOLE_EMERG_WRITE.high
    (virtio_spec::blk::F::SIZE_MAX.bits().to_ne() >> 0) as u64 as u64, // features.BLK_SIZE_MAX.low
    (virtio_spec::blk::F::SIZE_MAX.bits().to_ne() >> 64) as u64 as u64, // features.BLK_SIZE_MAX.high
    (virtio_spec::blk::F::SEG_MAX.bits().to_ne() >> 0) as u64 as u64, // features.BLK_SEG_MAX.low
    (virtio_spec::blk::F::SEG_MAX.bits().to_ne() >> 64) as u64 as u64, // features.BLK_SEG_MAX.high
    (virtio_spec::blk::F::GEOMETRY.bits().to_ne() >> 0) as u64 as u64, // features.BLK_GEOMETRY.low
    (virtio_spec::blk::F::GEOMETRY.bits().to_ne() >> 64) as u64 as u64, // features.BLK_GEOMETRY.high
    (virtio_spec::blk::F::RO.bits().to_ne() >> 0) as u64 as u64, // features.BLK_RO.low
    (virtio_spec::blk::F::RO.bits().to_ne() >> 64) as u64 as u64, // features.BLK_RO.high
    (virtio_spec::blk::F::BLK_SIZE.bits().to_ne() >> 0) as u64 as u64, // features.BLK_BLK_SIZE.low
    (virtio_spec::blk::F::BLK_SIZE.bits().to_ne() >> 64) as u64 as u64, // features.BLK_BLK_SIZE.high
    (virtio_spec::blk::F::FLUSH.bits().to_ne() >> 0) as u64 as u64, // features.BLK_FLUSH.low
    (virtio_spec::blk::F::FLUSH.bits().to_ne() >> 64) as u64 as u64, // features.BLK_FLUSH.high
    (virtio_spec::blk::F::TOPOLOGY.bits().to_ne() >> 0) as u64 as u64, // features.BLK_TOPOLOGY.low
    (virtio_spec::blk::F::TOPOLOGY.bits().to_ne() >> 64) as u64 as u64, // features.BLK_TOPOLOGY.high
    (virtio_spec::blk::F::CONFIG_WCE.bits().to_ne() >> 0) as u64 as u64, // features.BLK_CONFIG_WCE.low
    (virtio_spec::blk::F::CONFIG_WCE.bits().to_ne() >> 64) as u64 as u64, // features.BLK_CONFIG_WCE.high
    (virtio_spec::blk::F::MQ.bits().to_ne() >> 0) as u64 as u64, // features.BLK_MQ.low
    (virtio_spec::blk::F::MQ.bits().to_ne() >> 64) as u64 as u64, // features.BLK_MQ.high
    (virtio_spec::blk::F::DISCARD.bits().to_ne() >> 0) as u64 as u64, // features.BLK_DISCARD.low
    (virtio_spec::blk::F::DISCARD.bits().to_ne() >> 64) as u64 as u64, // features.BLK_DISCARD.high
    (virtio_spec::blk::F::WRITE_ZEROES.bits().to_ne() >> 0) as u64 as u64, // features.BLK_WRITE_ZEROES.low
    (virtio_spec::blk::F::WRITE_ZEROES.bits().to_ne() >> 64) as u64 as u64, // features.BLK_WRITE_ZEROES.high
    (virtio_spec::blk::F::LIFETIME.bits().to_ne() >> 0) as u64 as u64, // features.BLK_LIFETIME.low
    (virtio_spec::blk::F::LIFETIME.bits().to_ne() >> 64) as u64 as u64, // features.BLK_LIFETIME.high
    (virtio_spec::blk::F::SECURE_ERASE.bits().to_ne() >> 0) as u64 as u64, // features.BLK_SECURE_ERASE.low
    (virtio_spec::blk::F::SECURE_ERASE.bits().to_ne() >> 64) as u64 as u64, // features.BLK_SECURE_ERASE.high
    (virtio_spec::blk::F::ZONED.bits().to_ne() >> 0) as u64 as u64, // features.BLK_ZONED.low
    (virtio_spec::blk::F::ZONED.bits().to_ne() >> 64) as u64 as u64, // features.BLK_ZONED.high
    (virtio_spec::net::F::CSUM.bits().to_ne() >> 0) as u64 as u64, // features.NET_CSUM.low
    (virtio_spec::net::F::CSUM.bits().to_ne() >> 64) as u64 as u64, // features.NET_CSUM.high
    (virtio_spec::net::F::GUEST_CSUM.bits().to_ne() >> 0) as u64 as u64, // features.NET_GUEST_CSUM.low
    (virtio_spec::net::F::GUEST_CSUM.bits().to_ne() >> 64) as u64 as u64, // features.NET_GUEST_CSUM.high
    (virtio_spec::net::F::CTRL_GUEST_OFFLOADS.bits().to_ne() >> 0) as u64 as u64, // features.NET_CTRL_GUEST_OFFLOADS.low
    (virtio_spec::net::F::CTRL_GUEST_OFFLOADS.bits().to_ne() >> 64) as u64 as u64, // features.NET_CTRL_GUEST_OFFLOADS.high
    (virtio_spec::net::F::MTU.bits().to_ne() >> 0) as u64 as u64, // features.NET_MTU.low
    (virtio_spec::net::F::MTU.bits().to_ne() >> 64) as u64 as u64, // features.NET_MTU.high
    (virtio_spec::net::F::MAC.bits().to_ne() >> 0) as u64 as u64, // features.NET_MAC.low
    (virtio_spec::net::F::MAC.bits().to_ne() >> 64) as u64 as u64, // features.NET_MAC.high
    (virtio_spec::net::F::GUEST_TSO4.bits().to_ne() >> 0) as u64 as u64, // features.NET_GUEST_TSO4.low
    (virtio_spec::net::F::GUEST_TSO4.bits().to_ne() >> 64) as u64 as u64, // features.NET_GUEST_TSO4.high
    (virtio_spec::net::F::GUEST_TSO6.bits().to_ne() >> 0) as u64 as u64, // features.NET_GUEST_TSO6.low
    (virtio_spec::net::F::GUEST_TSO6.bits().to_ne() >> 64) as u64 as u64, // features.NET_GUEST_TSO6.high
    (virtio_spec::net::F::GUEST_ECN.bits().to_ne() >> 0) as u64 as u64, // features.NET_GUEST_ECN.low
    (virtio_spec::net::F::GUEST_ECN.bits().to_ne() >> 64) as u64 as u64, // features.NET_GUEST_ECN.high
    (virtio_spec::net::F::GUEST_UFO.bits().to_ne() >> 0) as u64 as u64, // features.NET_GUEST_UFO.low
    (virtio_spec::net::F::GUEST_UFO.bits().to_ne() >> 64) as u64 as u64, // features.NET_GUEST_UFO.high
    (virtio_spec::net::F::HOST_TSO4.bits().to_ne() >> 0) as u64 as u64, // features.NET_HOST_TSO4.low
    (virtio_spec::net::F::HOST_TSO4.bits().to_ne() >> 64) as u64 as u64, // features.NET_HOST_TSO4.high
    (virtio_spec::net::F::HOST_TSO6.bits().to_ne() >> 0) as u64 as u64, // features.NET_HOST_TSO6.low
    (virtio_spec::net::F::HOST_TSO6.bits().to_ne() >> 64) as u64 as u64, // features.NET_HOST_TSO6.high
    (virtio_spec::net::F::HOST_ECN.bits().to_ne() >> 0) as u64 as u64, // features.NET_HOST_ECN.low
    (virtio_spec::net::F::HOST_ECN.bits().to_ne() >> 64) as u64 as u64, // features.NET_HOST_ECN.high
    (virtio_spec::net::F::HOST_UFO.bits().to_ne() >> 0) as u64 as u64, // features.NET_HOST_UFO.low
    (virtio_spec::net::F::HOST_UFO.bits().to_ne() >> 64) as u64 as u64, // features.NET_HOST_UFO.high
    (virtio_spec::net::F::MRG_RXBUF.bits().to_ne() >> 0) as u64 as u64, // features.NET_MRG_RXBUF.low
    (virtio_spec::net::F::MRG_RXBUF.bits().to_ne() >> 64) as u64 as u64, // features.NET_MRG_RXBUF.high
    (virtio_spec::net::F::STATUS.bits().to_ne() >> 0) as u64 as u64, // features.NET_STATUS.low
    (virtio_spec::net::F::STATUS.bits().to_ne() >> 64) as u64 as u64, // features.NET_STATUS.high
    (virtio_spec::net::F::CTRL_VQ.bits().to_ne() >> 0) as u64 as u64, // features.NET_CTRL_VQ.low
    (virtio_spec::net::F::CTRL_VQ.bits().to_ne() >> 64) as u64 as u64, // features.NET_CTRL_VQ.high
    (virtio_spec::net::F::CTRL_RX.bits().to_ne() >> 0) as u64 as u64, // features.NET_CTRL_RX.low
    (virtio_spec::net::F::CTRL_RX.bits().to_ne() >> 64) as u64 as u64, // features.NET_CTRL_RX.high
    (virtio_spec::net::F::CTRL_VLAN.bits().to_ne() >> 0) as u64 as u64, // features.NET_CTRL_VLAN.low
    (virtio_spec::net::F::CTRL_VLAN.bits().to_ne() >> 64) as u64 as u64, // features.NET_CTRL_VLAN.high
    (virtio_spec::net::F::CTRL_RX_EXTRA.bits().to_ne() >> 0) as u64 as u64, // features.NET_CTRL_RX_EXTRA.low
    (virtio_spec::net::F::CTRL_RX_EXTRA.bits().to_ne() >> 64) as u64 as u64, // features.NET_CTRL_RX_EXTRA.high
    (virtio_spec::net::F::GUEST_ANNOUNCE.bits().to_ne() >> 0) as u64 as u64, // features.NET_GUEST_ANNOUNCE.low
    (virtio_spec::net::F::GUEST_ANNOUNCE.bits().to_ne() >> 64) as u64 as u64, // features.NET_GUEST_ANNOUNCE.high
    (virtio_spec::net::F::MQ.bits().to_ne() >> 0) as u64 as u64, // features.NET_MQ.low
    (virtio_spec::net::F::MQ.bits().to_ne() >> 64) as u64 as u64, // features.NET_MQ.high
    (virtio_spec::net::F::CTRL_MAC_ADDR.bits().to_ne() >> 0) as u64 as u64, // features.NET_CTRL_MAC_ADDR.low
    (virtio_spec::net::F::CTRL_MAC_ADDR.bits().to_ne() >> 64) as u64 as u64, // features.NET_CTRL_MAC_ADDR.high
    (virtio_spec::net::F::DEVICE_STATS.bits().to_ne() >> 0) as u64 as u64, // features.NET_DEVICE_STATS.low
    (virtio_spec::net::F::DEVICE_STATS.bits().to_ne() >> 64) as u64 as u64, // features.NET_DEVICE_STATS.high
    (virtio_spec::net::F::HASH_TUNNEL.bits().to_ne() >> 0) as u64 as u64, // features.NET_HASH_TUNNEL.low
    (virtio_spec::net::F::HASH_TUNNEL.bits().to_ne() >> 64) as u64 as u64, // features.NET_HASH_TUNNEL.high
    (virtio_spec::net::F::VQ_NOTF_COAL.bits().to_ne() >> 0) as u64 as u64, // features.NET_VQ_NOTF_COAL.low
    (virtio_spec::net::F::VQ_NOTF_COAL.bits().to_ne() >> 64) as u64 as u64, // features.NET_VQ_NOTF_COAL.high
    (virtio_spec::net::F::NOTF_COAL.bits().to_ne() >> 0) as u64 as u64, // features.NET_NOTF_COAL.low
    (virtio_spec::net::F::NOTF_COAL.bits().to_ne() >> 64) as u64 as u64, // features.NET_NOTF_COAL.high
    (virtio_spec::net::F::GUEST_USO4.bits().to_ne() >> 0) as u64 as u64, // features.NET_GUEST_USO4.low
    (virtio_spec::net::F::GUEST_USO4.bits().to_ne() >> 64) as u64 as u64, // features.NET_GUEST_USO4.high
    (virtio_spec::net::F::GUEST_USO6.bits().to_ne() >> 0) as u64 as u64, // features.NET_GUEST_USO6.low
    (virtio_spec::net::F::GUEST_USO6.bits().to_ne() >> 64) as u64 as u64, // features.NET_GUEST_USO6.high
    (virtio_spec::net::F::HOST_USO.bits().to_ne() >> 0) as u64 as u64, // features.NET_HOST_USO.low
    (virtio_spec::net::F::HOST_USO.bits().to_ne() >> 64) as u64 as u64, // features.NET_HOST_USO.high
    (virtio_spec::net::F::HASH_REPORT.bits().to_ne() >> 0) as u64 as u64, // features.NET_HASH_REPORT.low
    (virtio_spec::net::F::HASH_REPORT.bits().to_ne() >> 64) as u64 as u64, // features.NET_HASH_REPORT.high
    (virtio_spec::net::F::GUEST_HDRLEN.bits().to_ne() >> 0) as u64 as u64, // features.NET_GUEST_HDRLEN.low
    (virtio_spec::net::F::GUEST_HDRLEN.bits().to_ne() >> 64) as u64 as u64, // features.NET_GUEST_HDRLEN.high
    (virtio_spec::net::F::RSS.bits().to_ne() >> 0) as u64 as u64, // features.NET_RSS.low
    (virtio_spec::net::F::RSS.bits().to_ne() >> 64) as u64 as u64, // features.NET_RSS.high
    (virtio_spec::net::F::RSC_EXT.bits().to_ne() >> 0) as u64 as u64, // features.NET_RSC_EXT.low
    (virtio_spec::net::F::RSC_EXT.bits().to_ne() >> 64) as u64 as u64, // features.NET_RSC_EXT.high
    (virtio_spec::net::F::STANDBY.bits().to_ne() >> 0) as u64 as u64, // features.NET_STANDBY.low
    (virtio_spec::net::F::STANDBY.bits().to_ne() >> 64) as u64 as u64, // features.NET_STANDBY.high
    (virtio_spec::net::F::SPEED_DUPLEX.bits().to_ne() >> 0) as u64 as u64, // features.NET_SPEED_DUPLEX.low
    (virtio_spec::net::F::SPEED_DUPLEX.bits().to_ne() >> 64) as u64 as u64, // features.NET_SPEED_DUPLEX.high
    (virtio_spec::net::F::RSS_CONTEXT.bits().to_ne() >> 0) as u64 as u64, // features.NET_RSS_CONTEXT.low
    (virtio_spec::net::F::RSS_CONTEXT.bits().to_ne() >> 64) as u64 as u64, // features.NET_RSS_CONTEXT.high
    (virtio_spec::net::F::GUEST_UDP_TUNNEL_GSO.bits().to_ne() >> 0) as u64 as u64, // features.NET_GUEST_UDP_TUNNEL_GSO.low
    (virtio_spec::net::F::GUEST_UDP_TUNNEL_GSO.bits().to_ne() >> 64) as u64 as u64, // features.NET_GUEST_UDP_TUNNEL_GSO.high
    (virtio_spec::net::F::GUEST_UDP_TUNNEL_GSO_CSUM.bits().to_ne() >> 0) as u64 as u64, // features.NET_GUEST_UDP_TUNNEL_GSO_CSUM.low
    (virtio_spec::net::F::GUEST_UDP_TUNNEL_GSO_CSUM.bits().to_ne() >> 64) as u64 as u64, // features.NET_GUEST_UDP_TUNNEL_GSO_CSUM.high
    (virtio_spec::net::F::HOST_UDP_TUNNEL_GSO.bits().to_ne() >> 0) as u64 as u64, // features.NET_HOST_UDP_TUNNEL_GSO.low
    (virtio_spec::net::F::HOST_UDP_TUNNEL_GSO.bits().to_ne() >> 64) as u64 as u64, // features.NET_HOST_UDP_TUNNEL_GSO.high
    (virtio_spec::net::F::HOST_UDP_TUNNEL_GSO_CSUM.bits().to_ne() >> 0) as u64 as u64, // features.NET_HOST_UDP_TUNNEL_GSO_CSUM.low
    (virtio_spec::net::F::HOST_UDP_TUNNEL_GSO_CSUM.bits().to_ne() >> 64) as u64 as u64, // features.NET_HOST_UDP_TUNNEL_GSO_CSUM.high
    (virtio_spec::net::F::OUT_NET_HEADER.bits().to_ne() >> 0) as u64 as u64, // features.NET_OUT_NET_HEADER.low
    (virtio_spec::net::F::OUT_NET_HEADER.bits().to_ne() >> 64) as u64 as u64, // features.NET_OUT_NET_HEADER.high
    (virtio_spec::net::F::IPSEC.bits().to_ne() >> 0) as u64 as u64, // features.NET_IPSEC.low
    (virtio_spec::net::F::IPSEC.bits().to_ne() >> 64) as u64 as u64, // features.NET_IPSEC.high
    (virtio_spec::fs::F::NOTIFICATION.bits().to_ne() >> 0) as u64 as u64, // features.FS_NOTIFICATION.low
    (virtio_spec::fs::F::NOTIFICATION.bits().to_ne() >> 64) as u64 as u64, // features.FS_NOTIFICATION.high
    (virtio_spec::vsock::F::STREAM.bits().to_ne() >> 0) as u64 as u64, // features.VSOCK_STREAM.low
    (virtio_spec::vsock::F::STREAM.bits().to_ne() >> 64) as u64 as u64, // features.VSOCK_STREAM.high
    (virtio_spec::vsock::F::SEQPACKET.bits().to_ne() >> 0) as u64 as u64, // features.VSOCK_SEQPACKET.low
    (virtio_spec::vsock::F::SEQPACKET.bits().to_ne() >> 64) as u64 as u64, // features.VSOCK_SEQPACKET.high
    (virtio_spec::vsock::F::NO_IMPLIED_STREAM.bits().to_ne() >> 0) as u64 as u64, // features.VSOCK_NO_IMPLIED_STREAM.low
    (virtio_spec::vsock::F::NO_IMPLIED_STREAM.bits().to_ne() >> 64) as u64 as u64, // features.VSOCK_NO_IMPLIED_STREAM.high
    (virtio_spec::balloon::F::MUST_TELL_HOST.bits().to_ne() >> 0) as u64 as u64, // features.BALLOON_MUST_TELL_HOST.low
    (virtio_spec::balloon::F::MUST_TELL_HOST.bits().to_ne() >> 64) as u64 as u64, // features.BALLOON_MUST_TELL_HOST.high
    (virtio_spec::balloon::F::STATS_VQ.bits().to_ne() >> 0) as u64 as u64, // features.BALLOON_STATS_VQ.low
    (virtio_spec::balloon::F::STATS_VQ.bits().to_ne() >> 64) as u64 as u64, // features.BALLOON_STATS_VQ.high
    (virtio_spec::balloon::F::DEFLATE_ON_OOM.bits().to_ne() >> 0) as u64 as u64, // features.BALLOON_DEFLATE_ON_OOM.low
    (virtio_spec::balloon::F::DEFLATE_ON_OOM.bits().to_ne() >> 64) as u64 as u64, // features.BALLOON_DEFLATE_ON_OOM.high
    (virtio_spec::balloon::F::FREE_PAGE_HINT.bits().to_ne() >> 0) as u64 as u64, // features.BALLOON_FREE_PAGE_HINT.low
    (virtio_spec::balloon::F::FREE_PAGE_HINT.bits().to_ne() >> 64) as u64 as u64, // features.BALLOON_FREE_PAGE_HINT.high
    (virtio_spec::balloon::F::PAGE_POISON.bits().to_ne() >> 0) as u64 as u64, // features.BALLOON_PAGE_POISON.low
    (virtio_spec::balloon::F::PAGE_POISON.bits().to_ne() >> 64) as u64 as u64, // features.BALLOON_PAGE_POISON.high
    (virtio_spec::balloon::F::PAGE_REPORTING.bits().to_ne() >> 0) as u64 as u64, // features.BALLOON_PAGE_REPORTING.low
    (virtio_spec::balloon::F::PAGE_REPORTING.bits().to_ne() >> 64) as u64 as u64, // features.BALLOON_PAGE_REPORTING.high
];

mod packed_probe;
mod transport_probe;
