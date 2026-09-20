// SPDX-License-Identifier: MIT OR Apache-2.0
#![no_std]
use core::mem::{size_of, align_of, offset_of};
#[unsafe(no_mangle)]
pub static CATHEDRAL_MACHINE_LAYOUT: [u64; 305] = [
    size_of::<uefi_raw::protocol::pci::root_bridge::PciRootBridgeIoProtocolWidth>() as u64, // PciRootBridgeIoProtocolWidth.size
    align_of::<uefi_raw::protocol::pci::root_bridge::PciRootBridgeIoProtocolWidth>() as u64, // PciRootBridgeIoProtocolWidth.alignment
    size_of::<uefi_raw::protocol::pci::root_bridge::PciRootBridgeIoProtocolOperation>() as u64, // PciRootBridgeIoProtocolOperation.size
    align_of::<uefi_raw::protocol::pci::root_bridge::PciRootBridgeIoProtocolOperation>() as u64, // PciRootBridgeIoProtocolOperation.alignment
    size_of::<uefi_raw::protocol::pci::root_bridge::PciRootBridgeIoProtocolAttributes>() as u64, // PciRootBridgeIoProtocolAttributes.size
    align_of::<uefi_raw::protocol::pci::root_bridge::PciRootBridgeIoProtocolAttributes>() as u64, // PciRootBridgeIoProtocolAttributes.alignment
    size_of::<uefi_raw::protocol::pci::root_bridge::PciRootBridgeIoAccess>() as u64, // PciRootBridgeIoAccess.size
    align_of::<uefi_raw::protocol::pci::root_bridge::PciRootBridgeIoAccess>() as u64, // PciRootBridgeIoAccess.alignment
    offset_of!(uefi_raw::protocol::pci::root_bridge::PciRootBridgeIoAccess, read) as u64, // PciRootBridgeIoAccess.read.offset
    offset_of!(uefi_raw::protocol::pci::root_bridge::PciRootBridgeIoAccess, write) as u64, // PciRootBridgeIoAccess.write.offset
    size_of::<uefi_raw::protocol::pci::root_bridge::PciRootBridgeIoProtocol>() as u64, // PciRootBridgeIoProtocol.size
    align_of::<uefi_raw::protocol::pci::root_bridge::PciRootBridgeIoProtocol>() as u64, // PciRootBridgeIoProtocol.alignment
    offset_of!(uefi_raw::protocol::pci::root_bridge::PciRootBridgeIoProtocol, parent_handle) as u64, // PciRootBridgeIoProtocol.parent_handle.offset
    offset_of!(uefi_raw::protocol::pci::root_bridge::PciRootBridgeIoProtocol, poll_mem) as u64, // PciRootBridgeIoProtocol.poll_mem.offset
    offset_of!(uefi_raw::protocol::pci::root_bridge::PciRootBridgeIoProtocol, poll_io) as u64, // PciRootBridgeIoProtocol.poll_io.offset
    offset_of!(uefi_raw::protocol::pci::root_bridge::PciRootBridgeIoProtocol, mem) as u64, // PciRootBridgeIoProtocol.mem.offset
    offset_of!(uefi_raw::protocol::pci::root_bridge::PciRootBridgeIoProtocol, io) as u64, // PciRootBridgeIoProtocol.io.offset
    offset_of!(uefi_raw::protocol::pci::root_bridge::PciRootBridgeIoProtocol, pci) as u64, // PciRootBridgeIoProtocol.pci.offset
    offset_of!(uefi_raw::protocol::pci::root_bridge::PciRootBridgeIoProtocol, copy_mem) as u64, // PciRootBridgeIoProtocol.copy_mem.offset
    offset_of!(uefi_raw::protocol::pci::root_bridge::PciRootBridgeIoProtocol, map) as u64, // PciRootBridgeIoProtocol.map.offset
    offset_of!(uefi_raw::protocol::pci::root_bridge::PciRootBridgeIoProtocol, unmap) as u64, // PciRootBridgeIoProtocol.unmap.offset
    offset_of!(uefi_raw::protocol::pci::root_bridge::PciRootBridgeIoProtocol, allocate_buffer) as u64, // PciRootBridgeIoProtocol.allocate_buffer.offset
    offset_of!(uefi_raw::protocol::pci::root_bridge::PciRootBridgeIoProtocol, free_buffer) as u64, // PciRootBridgeIoProtocol.free_buffer.offset
    offset_of!(uefi_raw::protocol::pci::root_bridge::PciRootBridgeIoProtocol, flush) as u64, // PciRootBridgeIoProtocol.flush.offset
    offset_of!(uefi_raw::protocol::pci::root_bridge::PciRootBridgeIoProtocol, get_attributes) as u64, // PciRootBridgeIoProtocol.get_attributes.offset
    offset_of!(uefi_raw::protocol::pci::root_bridge::PciRootBridgeIoProtocol, set_attributes) as u64, // PciRootBridgeIoProtocol.set_attributes.offset
    offset_of!(uefi_raw::protocol::pci::root_bridge::PciRootBridgeIoProtocol, configuration) as u64, // PciRootBridgeIoProtocol.configuration.offset
    offset_of!(uefi_raw::protocol::pci::root_bridge::PciRootBridgeIoProtocol, segment_number) as u64, // PciRootBridgeIoProtocol.segment_number.offset
    size_of::<uefi_raw::protocol::usb::DataDirection>() as u64, // DataDirection.size
    align_of::<uefi_raw::protocol::usb::DataDirection>() as u64, // DataDirection.alignment
    size_of::<uefi_raw::protocol::usb::DeviceRequest>() as u64, // DeviceRequest.size
    align_of::<uefi_raw::protocol::usb::DeviceRequest>() as u64, // DeviceRequest.alignment
    offset_of!(uefi_raw::protocol::usb::DeviceRequest, request_type) as u64, // DeviceRequest.request_type.offset
    offset_of!(uefi_raw::protocol::usb::DeviceRequest, request) as u64, // DeviceRequest.request.offset
    offset_of!(uefi_raw::protocol::usb::DeviceRequest, value) as u64, // DeviceRequest.value.offset
    offset_of!(uefi_raw::protocol::usb::DeviceRequest, index) as u64, // DeviceRequest.index.offset
    offset_of!(uefi_raw::protocol::usb::DeviceRequest, length) as u64, // DeviceRequest.length.offset
    size_of::<uefi_raw::protocol::usb::UsbTransferStatus>() as u64, // UsbTransferStatus.size
    align_of::<uefi_raw::protocol::usb::UsbTransferStatus>() as u64, // UsbTransferStatus.alignment
    size_of::<uefi_raw::protocol::usb::DeviceDescriptor>() as u64, // DeviceDescriptor.size
    align_of::<uefi_raw::protocol::usb::DeviceDescriptor>() as u64, // DeviceDescriptor.alignment
    offset_of!(uefi_raw::protocol::usb::DeviceDescriptor, length) as u64, // DeviceDescriptor.length.offset
    offset_of!(uefi_raw::protocol::usb::DeviceDescriptor, descriptor_type) as u64, // DeviceDescriptor.descriptor_type.offset
    offset_of!(uefi_raw::protocol::usb::DeviceDescriptor, bcd_usb) as u64, // DeviceDescriptor.bcd_usb.offset
    offset_of!(uefi_raw::protocol::usb::DeviceDescriptor, device_class) as u64, // DeviceDescriptor.device_class.offset
    offset_of!(uefi_raw::protocol::usb::DeviceDescriptor, device_subclass) as u64, // DeviceDescriptor.device_subclass.offset
    offset_of!(uefi_raw::protocol::usb::DeviceDescriptor, device_protocol) as u64, // DeviceDescriptor.device_protocol.offset
    offset_of!(uefi_raw::protocol::usb::DeviceDescriptor, max_packet_size) as u64, // DeviceDescriptor.max_packet_size.offset
    offset_of!(uefi_raw::protocol::usb::DeviceDescriptor, id_vendor) as u64, // DeviceDescriptor.id_vendor.offset
    offset_of!(uefi_raw::protocol::usb::DeviceDescriptor, id_product) as u64, // DeviceDescriptor.id_product.offset
    offset_of!(uefi_raw::protocol::usb::DeviceDescriptor, bcd_device) as u64, // DeviceDescriptor.bcd_device.offset
    offset_of!(uefi_raw::protocol::usb::DeviceDescriptor, str_manufacturer) as u64, // DeviceDescriptor.str_manufacturer.offset
    offset_of!(uefi_raw::protocol::usb::DeviceDescriptor, str_product) as u64, // DeviceDescriptor.str_product.offset
    offset_of!(uefi_raw::protocol::usb::DeviceDescriptor, str_serial_number) as u64, // DeviceDescriptor.str_serial_number.offset
    offset_of!(uefi_raw::protocol::usb::DeviceDescriptor, num_configurations) as u64, // DeviceDescriptor.num_configurations.offset
    size_of::<uefi_raw::protocol::usb::ConfigDescriptor>() as u64, // ConfigDescriptor.size
    align_of::<uefi_raw::protocol::usb::ConfigDescriptor>() as u64, // ConfigDescriptor.alignment
    offset_of!(uefi_raw::protocol::usb::ConfigDescriptor, length) as u64, // ConfigDescriptor.length.offset
    offset_of!(uefi_raw::protocol::usb::ConfigDescriptor, descriptor_type) as u64, // ConfigDescriptor.descriptor_type.offset
    offset_of!(uefi_raw::protocol::usb::ConfigDescriptor, total_length) as u64, // ConfigDescriptor.total_length.offset
    offset_of!(uefi_raw::protocol::usb::ConfigDescriptor, num_interfaces) as u64, // ConfigDescriptor.num_interfaces.offset
    offset_of!(uefi_raw::protocol::usb::ConfigDescriptor, configuration_value) as u64, // ConfigDescriptor.configuration_value.offset
    offset_of!(uefi_raw::protocol::usb::ConfigDescriptor, configuration) as u64, // ConfigDescriptor.configuration.offset
    offset_of!(uefi_raw::protocol::usb::ConfigDescriptor, attributes) as u64, // ConfigDescriptor.attributes.offset
    offset_of!(uefi_raw::protocol::usb::ConfigDescriptor, max_power) as u64, // ConfigDescriptor.max_power.offset
    size_of::<uefi_raw::protocol::usb::InterfaceDescriptor>() as u64, // InterfaceDescriptor.size
    align_of::<uefi_raw::protocol::usb::InterfaceDescriptor>() as u64, // InterfaceDescriptor.alignment
    offset_of!(uefi_raw::protocol::usb::InterfaceDescriptor, length) as u64, // InterfaceDescriptor.length.offset
    offset_of!(uefi_raw::protocol::usb::InterfaceDescriptor, descriptor_type) as u64, // InterfaceDescriptor.descriptor_type.offset
    offset_of!(uefi_raw::protocol::usb::InterfaceDescriptor, interface_number) as u64, // InterfaceDescriptor.interface_number.offset
    offset_of!(uefi_raw::protocol::usb::InterfaceDescriptor, alternate_setting) as u64, // InterfaceDescriptor.alternate_setting.offset
    offset_of!(uefi_raw::protocol::usb::InterfaceDescriptor, num_endpoints) as u64, // InterfaceDescriptor.num_endpoints.offset
    offset_of!(uefi_raw::protocol::usb::InterfaceDescriptor, interface_class) as u64, // InterfaceDescriptor.interface_class.offset
    offset_of!(uefi_raw::protocol::usb::InterfaceDescriptor, interface_subclass) as u64, // InterfaceDescriptor.interface_subclass.offset
    offset_of!(uefi_raw::protocol::usb::InterfaceDescriptor, interface_protocol) as u64, // InterfaceDescriptor.interface_protocol.offset
    offset_of!(uefi_raw::protocol::usb::InterfaceDescriptor, interface) as u64, // InterfaceDescriptor.interface.offset
    size_of::<uefi_raw::protocol::usb::EndpointDescriptor>() as u64, // EndpointDescriptor.size
    align_of::<uefi_raw::protocol::usb::EndpointDescriptor>() as u64, // EndpointDescriptor.alignment
    offset_of!(uefi_raw::protocol::usb::EndpointDescriptor, length) as u64, // EndpointDescriptor.length.offset
    offset_of!(uefi_raw::protocol::usb::EndpointDescriptor, descriptor_type) as u64, // EndpointDescriptor.descriptor_type.offset
    offset_of!(uefi_raw::protocol::usb::EndpointDescriptor, endpoint_address) as u64, // EndpointDescriptor.endpoint_address.offset
    offset_of!(uefi_raw::protocol::usb::EndpointDescriptor, attributes) as u64, // EndpointDescriptor.attributes.offset
    offset_of!(uefi_raw::protocol::usb::EndpointDescriptor, max_packet_size) as u64, // EndpointDescriptor.max_packet_size.offset
    offset_of!(uefi_raw::protocol::usb::EndpointDescriptor, interval) as u64, // EndpointDescriptor.interval.offset
    size_of::<uefi_raw::protocol::usb::AsyncUsbTransferCallback>() as u64, // AsyncUsbTransferCallback.size
    align_of::<uefi_raw::protocol::usb::AsyncUsbTransferCallback>() as u64, // AsyncUsbTransferCallback.alignment
    size_of::<uefi_raw::protocol::usb::io::UsbIoProtocol>() as u64, // UsbIoProtocol.size
    align_of::<uefi_raw::protocol::usb::io::UsbIoProtocol>() as u64, // UsbIoProtocol.alignment
    offset_of!(uefi_raw::protocol::usb::io::UsbIoProtocol, control_transfer) as u64, // UsbIoProtocol.control_transfer.offset
    offset_of!(uefi_raw::protocol::usb::io::UsbIoProtocol, bulk_transfer) as u64, // UsbIoProtocol.bulk_transfer.offset
    offset_of!(uefi_raw::protocol::usb::io::UsbIoProtocol, async_interrupt_transfer) as u64, // UsbIoProtocol.async_interrupt_transfer.offset
    offset_of!(uefi_raw::protocol::usb::io::UsbIoProtocol, sync_interrupt_transfer) as u64, // UsbIoProtocol.sync_interrupt_transfer.offset
    offset_of!(uefi_raw::protocol::usb::io::UsbIoProtocol, isochronous_transfer) as u64, // UsbIoProtocol.isochronous_transfer.offset
    offset_of!(uefi_raw::protocol::usb::io::UsbIoProtocol, async_isochronous_transfer) as u64, // UsbIoProtocol.async_isochronous_transfer.offset
    offset_of!(uefi_raw::protocol::usb::io::UsbIoProtocol, get_device_descriptor) as u64, // UsbIoProtocol.get_device_descriptor.offset
    offset_of!(uefi_raw::protocol::usb::io::UsbIoProtocol, get_config_descriptor) as u64, // UsbIoProtocol.get_config_descriptor.offset
    offset_of!(uefi_raw::protocol::usb::io::UsbIoProtocol, get_interface_descriptor) as u64, // UsbIoProtocol.get_interface_descriptor.offset
    offset_of!(uefi_raw::protocol::usb::io::UsbIoProtocol, get_endpoint_descriptor) as u64, // UsbIoProtocol.get_endpoint_descriptor.offset
    offset_of!(uefi_raw::protocol::usb::io::UsbIoProtocol, get_string_descriptor) as u64, // UsbIoProtocol.get_string_descriptor.offset
    offset_of!(uefi_raw::protocol::usb::io::UsbIoProtocol, get_supported_languages) as u64, // UsbIoProtocol.get_supported_languages.offset
    offset_of!(uefi_raw::protocol::usb::io::UsbIoProtocol, port_reset) as u64, // UsbIoProtocol.port_reset.offset
    size_of::<uefi_raw::protocol::usb::host_controller::Speed>() as u64, // Speed.size
    align_of::<uefi_raw::protocol::usb::host_controller::Speed>() as u64, // Speed.alignment
    size_of::<uefi_raw::protocol::usb::host_controller::ResetAttributes>() as u64, // ResetAttributes.size
    align_of::<uefi_raw::protocol::usb::host_controller::ResetAttributes>() as u64, // ResetAttributes.alignment
    size_of::<uefi_raw::protocol::usb::host_controller::HostControllerState>() as u64, // HostControllerState.size
    align_of::<uefi_raw::protocol::usb::host_controller::HostControllerState>() as u64, // HostControllerState.alignment
    size_of::<uefi_raw::protocol::usb::host_controller::TransactionTranslator>() as u64, // TransactionTranslator.size
    align_of::<uefi_raw::protocol::usb::host_controller::TransactionTranslator>() as u64, // TransactionTranslator.alignment
    offset_of!(uefi_raw::protocol::usb::host_controller::TransactionTranslator, hub_address) as u64, // TransactionTranslator.hub_address.offset
    offset_of!(uefi_raw::protocol::usb::host_controller::TransactionTranslator, port_number) as u64, // TransactionTranslator.port_number.offset
    size_of::<uefi_raw::protocol::usb::host_controller::UsbPortStatus>() as u64, // UsbPortStatus.size
    align_of::<uefi_raw::protocol::usb::host_controller::UsbPortStatus>() as u64, // UsbPortStatus.alignment
    offset_of!(uefi_raw::protocol::usb::host_controller::UsbPortStatus, port_status) as u64, // UsbPortStatus.port_status.offset
    offset_of!(uefi_raw::protocol::usb::host_controller::UsbPortStatus, port_change_status) as u64, // UsbPortStatus.port_change_status.offset
    size_of::<uefi_raw::protocol::usb::host_controller::PortStatus>() as u64, // PortStatus.size
    align_of::<uefi_raw::protocol::usb::host_controller::PortStatus>() as u64, // PortStatus.alignment
    size_of::<uefi_raw::protocol::usb::host_controller::PortChangeStatus>() as u64, // PortChangeStatus.size
    align_of::<uefi_raw::protocol::usb::host_controller::PortChangeStatus>() as u64, // PortChangeStatus.alignment
    size_of::<uefi_raw::protocol::usb::host_controller::PortFeature>() as u64, // PortFeature.size
    align_of::<uefi_raw::protocol::usb::host_controller::PortFeature>() as u64, // PortFeature.alignment
    size_of::<uefi_raw::protocol::usb::host_controller::Usb2HostControllerProtocol>() as u64, // Usb2HostControllerProtocol.size
    align_of::<uefi_raw::protocol::usb::host_controller::Usb2HostControllerProtocol>() as u64, // Usb2HostControllerProtocol.alignment
    offset_of!(uefi_raw::protocol::usb::host_controller::Usb2HostControllerProtocol, get_capability) as u64, // Usb2HostControllerProtocol.get_capability.offset
    offset_of!(uefi_raw::protocol::usb::host_controller::Usb2HostControllerProtocol, reset) as u64, // Usb2HostControllerProtocol.reset.offset
    offset_of!(uefi_raw::protocol::usb::host_controller::Usb2HostControllerProtocol, get_state) as u64, // Usb2HostControllerProtocol.get_state.offset
    offset_of!(uefi_raw::protocol::usb::host_controller::Usb2HostControllerProtocol, set_state) as u64, // Usb2HostControllerProtocol.set_state.offset
    offset_of!(uefi_raw::protocol::usb::host_controller::Usb2HostControllerProtocol, control_transfer) as u64, // Usb2HostControllerProtocol.control_transfer.offset
    offset_of!(uefi_raw::protocol::usb::host_controller::Usb2HostControllerProtocol, bulk_transfer) as u64, // Usb2HostControllerProtocol.bulk_transfer.offset
    offset_of!(uefi_raw::protocol::usb::host_controller::Usb2HostControllerProtocol, async_interrupt_transfer) as u64, // Usb2HostControllerProtocol.async_interrupt_transfer.offset
    offset_of!(uefi_raw::protocol::usb::host_controller::Usb2HostControllerProtocol, sync_interrupt_transfer) as u64, // Usb2HostControllerProtocol.sync_interrupt_transfer.offset
    offset_of!(uefi_raw::protocol::usb::host_controller::Usb2HostControllerProtocol, isochronous_transfer) as u64, // Usb2HostControllerProtocol.isochronous_transfer.offset
    offset_of!(uefi_raw::protocol::usb::host_controller::Usb2HostControllerProtocol, async_isochronous_transfer) as u64, // Usb2HostControllerProtocol.async_isochronous_transfer.offset
    offset_of!(uefi_raw::protocol::usb::host_controller::Usb2HostControllerProtocol, get_root_hub_port_status) as u64, // Usb2HostControllerProtocol.get_root_hub_port_status.offset
    offset_of!(uefi_raw::protocol::usb::host_controller::Usb2HostControllerProtocol, set_root_hub_port_feature) as u64, // Usb2HostControllerProtocol.set_root_hub_port_feature.offset
    offset_of!(uefi_raw::protocol::usb::host_controller::Usb2HostControllerProtocol, clear_root_hub_port_feature) as u64, // Usb2HostControllerProtocol.clear_root_hub_port_feature.offset
    offset_of!(uefi_raw::protocol::usb::host_controller::Usb2HostControllerProtocol, major_revision) as u64, // Usb2HostControllerProtocol.major_revision.offset
    offset_of!(uefi_raw::protocol::usb::host_controller::Usb2HostControllerProtocol, minor_revision) as u64, // Usb2HostControllerProtocol.minor_revision.offset
    size_of::<uefi_raw::protocol::iommu::EdkiiIommuProtocol>() as u64, // EdkiiIommuProtocol.size
    align_of::<uefi_raw::protocol::iommu::EdkiiIommuProtocol>() as u64, // EdkiiIommuProtocol.alignment
    offset_of!(uefi_raw::protocol::iommu::EdkiiIommuProtocol, revision) as u64, // EdkiiIommuProtocol.revision.offset
    offset_of!(uefi_raw::protocol::iommu::EdkiiIommuProtocol, set_attribute) as u64, // EdkiiIommuProtocol.set_attribute.offset
    offset_of!(uefi_raw::protocol::iommu::EdkiiIommuProtocol, map) as u64, // EdkiiIommuProtocol.map.offset
    offset_of!(uefi_raw::protocol::iommu::EdkiiIommuProtocol, unmap) as u64, // EdkiiIommuProtocol.unmap.offset
    offset_of!(uefi_raw::protocol::iommu::EdkiiIommuProtocol, allocate_buffer) as u64, // EdkiiIommuProtocol.allocate_buffer.offset
    offset_of!(uefi_raw::protocol::iommu::EdkiiIommuProtocol, free_buffer) as u64, // EdkiiIommuProtocol.free_buffer.offset
    size_of::<uefi_raw::protocol::iommu::EdkiiIommuOperation>() as u64, // EdkiiIommuOperation.size
    align_of::<uefi_raw::protocol::iommu::EdkiiIommuOperation>() as u64, // EdkiiIommuOperation.alignment
    size_of::<uefi_raw::protocol::iommu::EdkiiIommuAttribute>() as u64, // EdkiiIommuAttribute.size
    align_of::<uefi_raw::protocol::iommu::EdkiiIommuAttribute>() as u64, // EdkiiIommuAttribute.alignment
    size_of::<uefi_raw::protocol::iommu::EdkiiIommuAccess>() as u64, // EdkiiIommuAccess.size
    align_of::<uefi_raw::protocol::iommu::EdkiiIommuAccess>() as u64, // EdkiiIommuAccess.alignment
    size_of::<uefi_raw::protocol::rng::RngAlgorithmType>() as u64, // RngAlgorithmType.size
    align_of::<uefi_raw::protocol::rng::RngAlgorithmType>() as u64, // RngAlgorithmType.alignment
    size_of::<uefi_raw::protocol::rng::RngProtocol>() as u64, // RngProtocol.size
    align_of::<uefi_raw::protocol::rng::RngProtocol>() as u64, // RngProtocol.alignment
    offset_of!(uefi_raw::protocol::rng::RngProtocol, get_info) as u64, // RngProtocol.get_info.offset
    offset_of!(uefi_raw::protocol::rng::RngProtocol, get_rng) as u64, // RngProtocol.get_rng.offset
    size_of::<uefi_raw::protocol::acpi::AcpiTableProtocol>() as u64, // AcpiTableProtocol.size
    align_of::<uefi_raw::protocol::acpi::AcpiTableProtocol>() as u64, // AcpiTableProtocol.alignment
    offset_of!(uefi_raw::protocol::acpi::AcpiTableProtocol, install_acpi_table) as u64, // AcpiTableProtocol.install_acpi_table.offset
    offset_of!(uefi_raw::protocol::acpi::AcpiTableProtocol, uninstall_acpi_table) as u64, // AcpiTableProtocol.uninstall_acpi_table.offset
    size_of::<uefi_raw::protocol::memory_protection::MemoryAttributeProtocol>() as u64, // MemoryAttributeProtocol.size
    align_of::<uefi_raw::protocol::memory_protection::MemoryAttributeProtocol>() as u64, // MemoryAttributeProtocol.alignment
    offset_of!(uefi_raw::protocol::memory_protection::MemoryAttributeProtocol, get_memory_attributes) as u64, // MemoryAttributeProtocol.get_memory_attributes.offset
    offset_of!(uefi_raw::protocol::memory_protection::MemoryAttributeProtocol, set_memory_attributes) as u64, // MemoryAttributeProtocol.set_memory_attributes.offset
    offset_of!(uefi_raw::protocol::memory_protection::MemoryAttributeProtocol, clear_memory_attributes) as u64, // MemoryAttributeProtocol.clear_memory_attributes.offset
    size_of::<uefi_raw::protocol::misc::TimestampProtocol>() as u64, // TimestampProtocol.size
    align_of::<uefi_raw::protocol::misc::TimestampProtocol>() as u64, // TimestampProtocol.alignment
    offset_of!(uefi_raw::protocol::misc::TimestampProtocol, get_timestamp) as u64, // TimestampProtocol.get_timestamp.offset
    offset_of!(uefi_raw::protocol::misc::TimestampProtocol, get_properties) as u64, // TimestampProtocol.get_properties.offset
    size_of::<uefi_raw::protocol::misc::TimestampProperties>() as u64, // TimestampProperties.size
    align_of::<uefi_raw::protocol::misc::TimestampProperties>() as u64, // TimestampProperties.alignment
    offset_of!(uefi_raw::protocol::misc::TimestampProperties, frequency) as u64, // TimestampProperties.frequency.offset
    offset_of!(uefi_raw::protocol::misc::TimestampProperties, end_value) as u64, // TimestampProperties.end_value.offset
    size_of::<uefi_raw::protocol::misc::ResetNotificationProtocol>() as u64, // ResetNotificationProtocol.size
    align_of::<uefi_raw::protocol::misc::ResetNotificationProtocol>() as u64, // ResetNotificationProtocol.alignment
    offset_of!(uefi_raw::protocol::misc::ResetNotificationProtocol, register_reset_notify) as u64, // ResetNotificationProtocol.register_reset_notify.offset
    offset_of!(uefi_raw::protocol::misc::ResetNotificationProtocol, unregister_reset_notify) as u64, // ResetNotificationProtocol.unregister_reset_notify.offset
    size_of::<uefi_raw::protocol::misc::ResetSystemFn>() as u64, // ResetSystemFn.size
    align_of::<uefi_raw::protocol::misc::ResetSystemFn>() as u64, // ResetSystemFn.alignment
    size_of::<uefi_raw::protocol::driver::DriverBindingProtocol>() as u64, // DriverBindingProtocol.size
    align_of::<uefi_raw::protocol::driver::DriverBindingProtocol>() as u64, // DriverBindingProtocol.alignment
    offset_of!(uefi_raw::protocol::driver::DriverBindingProtocol, supported) as u64, // DriverBindingProtocol.supported.offset
    offset_of!(uefi_raw::protocol::driver::DriverBindingProtocol, start) as u64, // DriverBindingProtocol.start.offset
    offset_of!(uefi_raw::protocol::driver::DriverBindingProtocol, stop) as u64, // DriverBindingProtocol.stop.offset
    offset_of!(uefi_raw::protocol::driver::DriverBindingProtocol, version) as u64, // DriverBindingProtocol.version.offset
    offset_of!(uefi_raw::protocol::driver::DriverBindingProtocol, image_handle) as u64, // DriverBindingProtocol.image_handle.offset
    offset_of!(uefi_raw::protocol::driver::DriverBindingProtocol, driver_binding_handle) as u64, // DriverBindingProtocol.driver_binding_handle.offset
    size_of::<uefi_raw::protocol::driver::ComponentName2Protocol>() as u64, // ComponentName2Protocol.size
    align_of::<uefi_raw::protocol::driver::ComponentName2Protocol>() as u64, // ComponentName2Protocol.alignment
    offset_of!(uefi_raw::protocol::driver::ComponentName2Protocol, get_driver_name) as u64, // ComponentName2Protocol.get_driver_name.offset
    offset_of!(uefi_raw::protocol::driver::ComponentName2Protocol, get_controller_name) as u64, // ComponentName2Protocol.get_controller_name.offset
    offset_of!(uefi_raw::protocol::driver::ComponentName2Protocol, supported_languages) as u64, // ComponentName2Protocol.supported_languages.offset
    size_of::<uefi_raw::protocol::driver::ServiceBindingProtocol>() as u64, // ServiceBindingProtocol.size
    align_of::<uefi_raw::protocol::driver::ServiceBindingProtocol>() as u64, // ServiceBindingProtocol.alignment
    offset_of!(uefi_raw::protocol::driver::ServiceBindingProtocol, create_child) as u64, // ServiceBindingProtocol.create_child.offset
    offset_of!(uefi_raw::protocol::driver::ServiceBindingProtocol, destroy_child) as u64, // ServiceBindingProtocol.destroy_child.offset
    size_of::<uefi_raw::protocol::string::UnicodeCollationProtocol>() as u64, // UnicodeCollationProtocol.size
    align_of::<uefi_raw::protocol::string::UnicodeCollationProtocol>() as u64, // UnicodeCollationProtocol.alignment
    offset_of!(uefi_raw::protocol::string::UnicodeCollationProtocol, stri_coll) as u64, // UnicodeCollationProtocol.stri_coll.offset
    offset_of!(uefi_raw::protocol::string::UnicodeCollationProtocol, metai_match) as u64, // UnicodeCollationProtocol.metai_match.offset
    offset_of!(uefi_raw::protocol::string::UnicodeCollationProtocol, str_lwr) as u64, // UnicodeCollationProtocol.str_lwr.offset
    offset_of!(uefi_raw::protocol::string::UnicodeCollationProtocol, str_upr) as u64, // UnicodeCollationProtocol.str_upr.offset
    offset_of!(uefi_raw::protocol::string::UnicodeCollationProtocol, fat_to_str) as u64, // UnicodeCollationProtocol.fat_to_str.offset
    offset_of!(uefi_raw::protocol::string::UnicodeCollationProtocol, str_to_fat) as u64, // UnicodeCollationProtocol.str_to_fat.offset
    offset_of!(uefi_raw::protocol::string::UnicodeCollationProtocol, supported_languages) as u64, // UnicodeCollationProtocol.supported_languages.offset
    uefi_raw::protocol::pci::root_bridge::PciRootBridgeIoProtocolWidth::UINT8.0 as u64, // PCI_ROOT_BRIDGE_IO_PROTOCOL_WIDTH_UINT8
    uefi_raw::protocol::pci::root_bridge::PciRootBridgeIoProtocolWidth::UINT16.0 as u64, // PCI_ROOT_BRIDGE_IO_PROTOCOL_WIDTH_UINT16
    uefi_raw::protocol::pci::root_bridge::PciRootBridgeIoProtocolWidth::UINT32.0 as u64, // PCI_ROOT_BRIDGE_IO_PROTOCOL_WIDTH_UINT32
    uefi_raw::protocol::pci::root_bridge::PciRootBridgeIoProtocolWidth::UINT64.0 as u64, // PCI_ROOT_BRIDGE_IO_PROTOCOL_WIDTH_UINT64
    uefi_raw::protocol::pci::root_bridge::PciRootBridgeIoProtocolWidth::FIFO_UINT8.0 as u64, // PCI_ROOT_BRIDGE_IO_PROTOCOL_WIDTH_FIFO_UINT8
    uefi_raw::protocol::pci::root_bridge::PciRootBridgeIoProtocolWidth::FIFO_UINT16.0 as u64, // PCI_ROOT_BRIDGE_IO_PROTOCOL_WIDTH_FIFO_UINT16
    uefi_raw::protocol::pci::root_bridge::PciRootBridgeIoProtocolWidth::FIFO_UINT32.0 as u64, // PCI_ROOT_BRIDGE_IO_PROTOCOL_WIDTH_FIFO_UINT32
    uefi_raw::protocol::pci::root_bridge::PciRootBridgeIoProtocolWidth::FIFO_UINT64.0 as u64, // PCI_ROOT_BRIDGE_IO_PROTOCOL_WIDTH_FIFO_UINT64
    uefi_raw::protocol::pci::root_bridge::PciRootBridgeIoProtocolWidth::FILL_UINT8.0 as u64, // PCI_ROOT_BRIDGE_IO_PROTOCOL_WIDTH_FILL_UINT8
    uefi_raw::protocol::pci::root_bridge::PciRootBridgeIoProtocolWidth::FILL_UINT16.0 as u64, // PCI_ROOT_BRIDGE_IO_PROTOCOL_WIDTH_FILL_UINT16
    uefi_raw::protocol::pci::root_bridge::PciRootBridgeIoProtocolWidth::FILL_UINT32.0 as u64, // PCI_ROOT_BRIDGE_IO_PROTOCOL_WIDTH_FILL_UINT32
    uefi_raw::protocol::pci::root_bridge::PciRootBridgeIoProtocolWidth::FILL_UINT64.0 as u64, // PCI_ROOT_BRIDGE_IO_PROTOCOL_WIDTH_FILL_UINT64
    uefi_raw::protocol::pci::root_bridge::PciRootBridgeIoProtocolWidth::MAXIMUM.0 as u64, // PCI_ROOT_BRIDGE_IO_PROTOCOL_WIDTH_MAXIMUM
    uefi_raw::protocol::pci::root_bridge::PciRootBridgeIoProtocolOperation::BUS_MASTER_READ.0 as u64, // PCI_ROOT_BRIDGE_IO_PROTOCOL_OPERATION_BUS_MASTER_READ
    uefi_raw::protocol::pci::root_bridge::PciRootBridgeIoProtocolOperation::BUS_MASTER_WRITE.0 as u64, // PCI_ROOT_BRIDGE_IO_PROTOCOL_OPERATION_BUS_MASTER_WRITE
    uefi_raw::protocol::pci::root_bridge::PciRootBridgeIoProtocolOperation::BUS_MASTER_COMMON_BUFFER.0 as u64, // PCI_ROOT_BRIDGE_IO_PROTOCOL_OPERATION_BUS_MASTER_COMMON_BUFFER
    uefi_raw::protocol::pci::root_bridge::PciRootBridgeIoProtocolOperation::BUS_MASTER_READ64.0 as u64, // PCI_ROOT_BRIDGE_IO_PROTOCOL_OPERATION_BUS_MASTER_READ64
    uefi_raw::protocol::pci::root_bridge::PciRootBridgeIoProtocolOperation::BUS_MASTER_WRITE64.0 as u64, // PCI_ROOT_BRIDGE_IO_PROTOCOL_OPERATION_BUS_MASTER_WRITE64
    uefi_raw::protocol::pci::root_bridge::PciRootBridgeIoProtocolOperation::BUS_MASTER_COMMON_BUFFER64.0 as u64, // PCI_ROOT_BRIDGE_IO_PROTOCOL_OPERATION_BUS_MASTER_COMMON_BUFFER64
    uefi_raw::protocol::pci::root_bridge::PciRootBridgeIoProtocolOperation::MAXIMUM.0 as u64, // PCI_ROOT_BRIDGE_IO_PROTOCOL_OPERATION_MAXIMUM
    uefi_raw::protocol::pci::root_bridge::PciRootBridgeIoProtocolAttributes::EFI_PCI_ATTRIBUTE_ISA_MOTHERBOARD_IO.bits() as u64, // PCI_ROOT_BRIDGE_IO_PROTOCOL_ATTRIBUTES_EFI_PCI_ATTRIBUTE_ISA_MOTHERBOARD_IO
    uefi_raw::protocol::pci::root_bridge::PciRootBridgeIoProtocolAttributes::EFI_PCI_ATTRIBUTE_ISA_IO.bits() as u64, // PCI_ROOT_BRIDGE_IO_PROTOCOL_ATTRIBUTES_EFI_PCI_ATTRIBUTE_ISA_IO
    uefi_raw::protocol::pci::root_bridge::PciRootBridgeIoProtocolAttributes::EFI_PCI_ATTRIBUTE_VGA_PALETTE_IO.bits() as u64, // PCI_ROOT_BRIDGE_IO_PROTOCOL_ATTRIBUTES_EFI_PCI_ATTRIBUTE_VGA_PALETTE_IO
    uefi_raw::protocol::pci::root_bridge::PciRootBridgeIoProtocolAttributes::EFI_PCI_ATTRIBUTE_VGA_MEMORY.bits() as u64, // PCI_ROOT_BRIDGE_IO_PROTOCOL_ATTRIBUTES_EFI_PCI_ATTRIBUTE_VGA_MEMORY
    uefi_raw::protocol::pci::root_bridge::PciRootBridgeIoProtocolAttributes::EFI_PCI_ATTRIBUTE_VGA_IO.bits() as u64, // PCI_ROOT_BRIDGE_IO_PROTOCOL_ATTRIBUTES_EFI_PCI_ATTRIBUTE_VGA_IO
    uefi_raw::protocol::pci::root_bridge::PciRootBridgeIoProtocolAttributes::EFI_PCI_ATTRIBUTE_IDE_PRIMARY_IO.bits() as u64, // PCI_ROOT_BRIDGE_IO_PROTOCOL_ATTRIBUTES_EFI_PCI_ATTRIBUTE_IDE_PRIMARY_IO
    uefi_raw::protocol::pci::root_bridge::PciRootBridgeIoProtocolAttributes::EFI_PCI_ATTRIBUTE_IDE_SECONDARY_IO.bits() as u64, // PCI_ROOT_BRIDGE_IO_PROTOCOL_ATTRIBUTES_EFI_PCI_ATTRIBUTE_IDE_SECONDARY_IO
    uefi_raw::protocol::pci::root_bridge::PciRootBridgeIoProtocolAttributes::EFI_PCI_ATTRIBUTE_MEMORY_WRITE_COMBINE.bits() as u64, // PCI_ROOT_BRIDGE_IO_PROTOCOL_ATTRIBUTES_EFI_PCI_ATTRIBUTE_MEMORY_WRITE_COMBINE
    uefi_raw::protocol::pci::root_bridge::PciRootBridgeIoProtocolAttributes::EFI_PCI_ATTRIBUTE_MEMORY_CACHED.bits() as u64, // PCI_ROOT_BRIDGE_IO_PROTOCOL_ATTRIBUTES_EFI_PCI_ATTRIBUTE_MEMORY_CACHED
    uefi_raw::protocol::pci::root_bridge::PciRootBridgeIoProtocolAttributes::EFI_PCI_ATTRIBUTE_MEMORY_DISABLE.bits() as u64, // PCI_ROOT_BRIDGE_IO_PROTOCOL_ATTRIBUTES_EFI_PCI_ATTRIBUTE_MEMORY_DISABLE
    uefi_raw::protocol::pci::root_bridge::PciRootBridgeIoProtocolAttributes::EFI_PCI_ATTRIBUTE_DUAL_ADDRESS_CYCLE.bits() as u64, // PCI_ROOT_BRIDGE_IO_PROTOCOL_ATTRIBUTES_EFI_PCI_ATTRIBUTE_DUAL_ADDRESS_CYCLE
    uefi_raw::protocol::pci::root_bridge::PciRootBridgeIoProtocolAttributes::EFI_PCI_ATTRIBUTE_ISA_IO_16.bits() as u64, // PCI_ROOT_BRIDGE_IO_PROTOCOL_ATTRIBUTES_EFI_PCI_ATTRIBUTE_ISA_IO_16
    uefi_raw::protocol::pci::root_bridge::PciRootBridgeIoProtocolAttributes::EFI_PCI_ATTRIBUTE_VGA_PALETTE_IO_16.bits() as u64, // PCI_ROOT_BRIDGE_IO_PROTOCOL_ATTRIBUTES_EFI_PCI_ATTRIBUTE_VGA_PALETTE_IO_16
    uefi_raw::protocol::pci::root_bridge::PciRootBridgeIoProtocolAttributes::EFI_PCI_ATTRIBUTE_VGA_IO_16.bits() as u64, // PCI_ROOT_BRIDGE_IO_PROTOCOL_ATTRIBUTES_EFI_PCI_ATTRIBUTE_VGA_IO_16
    uefi_raw::protocol::usb::DataDirection::DATA_IN.0 as u64, // DATA_DIRECTION_DATA_IN
    uefi_raw::protocol::usb::DataDirection::DATA_OUT.0 as u64, // DATA_DIRECTION_DATA_OUT
    uefi_raw::protocol::usb::DataDirection::NO_DATA.0 as u64, // DATA_DIRECTION_NO_DATA
    uefi_raw::protocol::usb::UsbTransferStatus::NOT_EXECUTE.bits() as u64, // USB_TRANSFER_STATUS_NOT_EXECUTE
    uefi_raw::protocol::usb::UsbTransferStatus::STALL.bits() as u64, // USB_TRANSFER_STATUS_STALL
    uefi_raw::protocol::usb::UsbTransferStatus::BUFFER.bits() as u64, // USB_TRANSFER_STATUS_BUFFER
    uefi_raw::protocol::usb::UsbTransferStatus::BABBLE.bits() as u64, // USB_TRANSFER_STATUS_BABBLE
    uefi_raw::protocol::usb::UsbTransferStatus::NAK.bits() as u64, // USB_TRANSFER_STATUS_NAK
    uefi_raw::protocol::usb::UsbTransferStatus::CRC.bits() as u64, // USB_TRANSFER_STATUS_CRC
    uefi_raw::protocol::usb::UsbTransferStatus::TIMEOUT.bits() as u64, // USB_TRANSFER_STATUS_TIMEOUT
    uefi_raw::protocol::usb::UsbTransferStatus::BIT_STUFF.bits() as u64, // USB_TRANSFER_STATUS_BIT_STUFF
    uefi_raw::protocol::usb::UsbTransferStatus::SYSTEM.bits() as u64, // USB_TRANSFER_STATUS_SYSTEM
    uefi_raw::protocol::usb::UsbTransferStatus::SUCCESS.bits() as u64, // USB_TRANSFER_STATUS_SUCCESS
    uefi_raw::protocol::usb::host_controller::Speed::FULL.0 as u64, // SPEED_FULL
    uefi_raw::protocol::usb::host_controller::Speed::LOW.0 as u64, // SPEED_LOW
    uefi_raw::protocol::usb::host_controller::Speed::HIGH.0 as u64, // SPEED_HIGH
    uefi_raw::protocol::usb::host_controller::Speed::SUPER.0 as u64, // SPEED_SUPER
    uefi_raw::protocol::usb::host_controller::ResetAttributes::RESET_GLOBAL.bits() as u64, // RESET_ATTRIBUTES_RESET_GLOBAL
    uefi_raw::protocol::usb::host_controller::ResetAttributes::RESET_HOST.bits() as u64, // RESET_ATTRIBUTES_RESET_HOST
    uefi_raw::protocol::usb::host_controller::ResetAttributes::RESET_GLOBAL_WITH_DEBUG.bits() as u64, // RESET_ATTRIBUTES_RESET_GLOBAL_WITH_DEBUG
    uefi_raw::protocol::usb::host_controller::ResetAttributes::RESET_HOST_WITH_DEBUG.bits() as u64, // RESET_ATTRIBUTES_RESET_HOST_WITH_DEBUG
    uefi_raw::protocol::usb::host_controller::HostControllerState::HALT.0 as u64, // HOST_CONTROLLER_STATE_HALT
    uefi_raw::protocol::usb::host_controller::HostControllerState::OPERATIONAL.0 as u64, // HOST_CONTROLLER_STATE_OPERATIONAL
    uefi_raw::protocol::usb::host_controller::HostControllerState::SUSPEND.0 as u64, // HOST_CONTROLLER_STATE_SUSPEND
    uefi_raw::protocol::usb::host_controller::PortStatus::CONNECTION.bits() as u64, // PORT_STATUS_CONNECTION
    uefi_raw::protocol::usb::host_controller::PortStatus::ENABLE.bits() as u64, // PORT_STATUS_ENABLE
    uefi_raw::protocol::usb::host_controller::PortStatus::SUSPEND.bits() as u64, // PORT_STATUS_SUSPEND
    uefi_raw::protocol::usb::host_controller::PortStatus::OVER_CURRENT.bits() as u64, // PORT_STATUS_OVER_CURRENT
    uefi_raw::protocol::usb::host_controller::PortStatus::RESET.bits() as u64, // PORT_STATUS_RESET
    uefi_raw::protocol::usb::host_controller::PortStatus::POWER.bits() as u64, // PORT_STATUS_POWER
    uefi_raw::protocol::usb::host_controller::PortStatus::LOW_SPEED.bits() as u64, // PORT_STATUS_LOW_SPEED
    uefi_raw::protocol::usb::host_controller::PortStatus::HIGH_SPEED.bits() as u64, // PORT_STATUS_HIGH_SPEED
    uefi_raw::protocol::usb::host_controller::PortStatus::SUPER_SPEED.bits() as u64, // PORT_STATUS_SUPER_SPEED
    uefi_raw::protocol::usb::host_controller::PortStatus::OWNER.bits() as u64, // PORT_STATUS_OWNER
    uefi_raw::protocol::usb::host_controller::PortChangeStatus::CONNECTION.bits() as u64, // PORT_CHANGE_STATUS_CONNECTION
    uefi_raw::protocol::usb::host_controller::PortChangeStatus::ENABLE.bits() as u64, // PORT_CHANGE_STATUS_ENABLE
    uefi_raw::protocol::usb::host_controller::PortChangeStatus::SUSPEND.bits() as u64, // PORT_CHANGE_STATUS_SUSPEND
    uefi_raw::protocol::usb::host_controller::PortChangeStatus::OVER_CURRENT.bits() as u64, // PORT_CHANGE_STATUS_OVER_CURRENT
    uefi_raw::protocol::usb::host_controller::PortChangeStatus::RESET.bits() as u64, // PORT_CHANGE_STATUS_RESET
    uefi_raw::protocol::usb::host_controller::PortFeature::ENABLE.0 as u64, // PORT_FEATURE_ENABLE
    uefi_raw::protocol::usb::host_controller::PortFeature::SUSPEND.0 as u64, // PORT_FEATURE_SUSPEND
    uefi_raw::protocol::usb::host_controller::PortFeature::RESET.0 as u64, // PORT_FEATURE_RESET
    uefi_raw::protocol::usb::host_controller::PortFeature::POWER.0 as u64, // PORT_FEATURE_POWER
    uefi_raw::protocol::usb::host_controller::PortFeature::OWNER.0 as u64, // PORT_FEATURE_OWNER
    uefi_raw::protocol::usb::host_controller::PortFeature::CONNECT_CHANGE.0 as u64, // PORT_FEATURE_CONNECT_CHANGE
    uefi_raw::protocol::usb::host_controller::PortFeature::ENABLE_CHANGE.0 as u64, // PORT_FEATURE_ENABLE_CHANGE
    uefi_raw::protocol::usb::host_controller::PortFeature::SUSPEND_CHANGE.0 as u64, // PORT_FEATURE_SUSPEND_CHANGE
    uefi_raw::protocol::usb::host_controller::PortFeature::OVER_CURRENT_CHARGE.0 as u64, // PORT_FEATURE_OVER_CURRENT_CHARGE
    uefi_raw::protocol::usb::host_controller::PortFeature::RESET_CHANGE.0 as u64, // PORT_FEATURE_RESET_CHANGE
    uefi_raw::protocol::iommu::EdkiiIommuOperation::BUS_MASTER_READ.0 as u64, // EDKII_IOMMU_OPERATION_BUS_MASTER_READ
    uefi_raw::protocol::iommu::EdkiiIommuOperation::BUS_MASTER_WRITE.0 as u64, // EDKII_IOMMU_OPERATION_BUS_MASTER_WRITE
    uefi_raw::protocol::iommu::EdkiiIommuOperation::BUS_MASTER_COMMON_BUFFER.0 as u64, // EDKII_IOMMU_OPERATION_BUS_MASTER_COMMON_BUFFER
    uefi_raw::protocol::iommu::EdkiiIommuOperation::BUS_MASTER_READ64.0 as u64, // EDKII_IOMMU_OPERATION_BUS_MASTER_READ64
    uefi_raw::protocol::iommu::EdkiiIommuOperation::BUS_MASTER_WRITE64.0 as u64, // EDKII_IOMMU_OPERATION_BUS_MASTER_WRITE64
    uefi_raw::protocol::iommu::EdkiiIommuOperation::BUS_MASTER_COMMON_BUFFER64.0 as u64, // EDKII_IOMMU_OPERATION_BUS_MASTER_COMMON_BUFFER64
    uefi_raw::protocol::iommu::EdkiiIommuOperation::MAXIMUM.0 as u64, // EDKII_IOMMU_OPERATION_MAXIMUM
    uefi_raw::protocol::iommu::EdkiiIommuAttribute::MEMORY_WRITE_COMBINE.bits() as u64, // EDKII_IOMMU_ATTRIBUTE_MEMORY_WRITE_COMBINE
    uefi_raw::protocol::iommu::EdkiiIommuAttribute::MEMORY_CACHED.bits() as u64, // EDKII_IOMMU_ATTRIBUTE_MEMORY_CACHED
    uefi_raw::protocol::iommu::EdkiiIommuAttribute::DUAL_ADDRESS_CYCLE.bits() as u64, // EDKII_IOMMU_ATTRIBUTE_DUAL_ADDRESS_CYCLE
    uefi_raw::protocol::iommu::EdkiiIommuAccess::READ.bits() as u64, // EDKII_IOMMU_ACCESS_READ
    uefi_raw::protocol::iommu::EdkiiIommuAccess::WRITE.bits() as u64, // EDKII_IOMMU_ACCESS_WRITE
    uefi_raw::protocol::iommu::EDKII_IOMMU_PROTOCOL_REVISION as u64, // EDKII_IOMMU_PROTOCOL_REVISION
    uefi_raw::protocol::iommu::EdkiiIommuAttribute::VALID_FOR_ALLOCATE_BUFFER.bits() as u64, // EDKII_IOMMU_ATTRIBUTE_VALID_FOR_ALLOCATE_BUFFER
    uefi_raw::protocol::iommu::EdkiiIommuAttribute::INVALID_FOR_ALLOCATE_BUFFER.bits() as u64, // EDKII_IOMMU_ATTRIBUTE_INVALID_FOR_ALLOCATE_BUFFER
];
const _: () = { let actual = uefi_raw::protocol::pci::root_bridge::PciRootBridgeIoProtocol::GUID.to_bytes(); let expected: [u8; 16] = [187, 126, 112, 47, 26, 74, 212, 17, 154, 56, 0, 144, 39, 63, 193, 77]; let mut i = 0; while i < 16 { assert!(actual[i] == expected[i]); i += 1; } };
const _: () = { let actual = uefi_raw::protocol::usb::io::UsbIoProtocol::GUID.to_bytes(); let expected: [u8; 16] = [214, 104, 47, 43, 210, 12, 207, 68, 142, 139, 187, 162, 11, 27, 91, 117]; let mut i = 0; while i < 16 { assert!(actual[i] == expected[i]); i += 1; } };
const _: () = { let actual = uefi_raw::protocol::usb::host_controller::Usb2HostControllerProtocol::GUID.to_bytes(); let expected: [u8; 16] = [38, 82, 116, 62, 24, 152, 182, 69, 162, 172, 215, 205, 14, 139, 162, 188]; let mut i = 0; while i < 16 { assert!(actual[i] == expected[i]); i += 1; } };
const _: () = { let actual = uefi_raw::protocol::iommu::EdkiiIommuProtocol::GUID.to_bytes(); let expected: [u8; 16] = [233, 157, 147, 78, 72, 217, 15, 75, 136, 237, 230, 225, 206, 81, 124, 30]; let mut i = 0; while i < 16 { assert!(actual[i] == expected[i]); i += 1; } };
const _: () = { let actual = uefi_raw::protocol::rng::RngAlgorithmType::EMPTY_ALGORITHM.0.to_bytes(); let expected: [u8; 16] = [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]; let mut i = 0; while i < 16 { assert!(actual[i] == expected[i]); i += 1; } };
const _: () = { let actual = uefi_raw::protocol::rng::RngAlgorithmType::ALGORITHM_RAW.0.to_bytes(); let expected: [u8; 16] = [215, 118, 49, 228, 232, 182, 39, 72, 183, 132, 127, 253, 196, 182, 133, 97]; let mut i = 0; while i < 16 { assert!(actual[i] == expected[i]); i += 1; } };
const _: () = { let actual = uefi_raw::protocol::rng::RngAlgorithmType::ALGORITHM_SP800_90_HASH_256.0.to_bytes(); let expected: [u8; 16] = [203, 103, 175, 167, 59, 96, 66, 77, 186, 33, 112, 191, 182, 41, 63, 150]; let mut i = 0; while i < 16 { assert!(actual[i] == expected[i]); i += 1; } };
const _: () = { let actual = uefi_raw::protocol::rng::RngAlgorithmType::ALGORITHM_SP800_90_HMAC_256.0.to_bytes(); let expected: [u8; 16] = [67, 155, 20, 197, 133, 174, 83, 79, 153, 130, 185, 67, 53, 211, 169, 231]; let mut i = 0; while i < 16 { assert!(actual[i] == expected[i]); i += 1; } };
const _: () = { let actual = uefi_raw::protocol::rng::RngAlgorithmType::ALGORITHM_SP800_90_CTR_256.0.to_bytes(); let expected: [u8; 16] = [110, 222, 240, 68, 140, 77, 69, 64, 168, 199, 77, 209, 104, 133, 107, 158]; let mut i = 0; while i < 16 { assert!(actual[i] == expected[i]); i += 1; } };
const _: () = { let actual = uefi_raw::protocol::rng::RngAlgorithmType::ALGORITHM_X9_31_3DES.0.to_bytes(); let expected: [u8; 16] = [90, 120, 196, 99, 52, 202, 18, 64, 163, 200, 11, 106, 50, 79, 85, 70]; let mut i = 0; while i < 16 { assert!(actual[i] == expected[i]); i += 1; } };
const _: () = { let actual = uefi_raw::protocol::rng::RngAlgorithmType::ALGORITHM_X9_31_AES.0.to_bytes(); let expected: [u8; 16] = [33, 51, 208, 172, 126, 119, 61, 77, 177, 200, 32, 207, 216, 136, 32, 201]; let mut i = 0; while i < 16 { assert!(actual[i] == expected[i]); i += 1; } };
const _: () = { let actual = uefi_raw::protocol::rng::RngProtocol::GUID.to_bytes(); let expected: [u8; 16] = [165, 188, 82, 49, 222, 234, 61, 67, 134, 46, 192, 28, 220, 41, 31, 68]; let mut i = 0; while i < 16 { assert!(actual[i] == expected[i]); i += 1; } };
const _: () = { let actual = uefi_raw::protocol::acpi::AcpiTableProtocol::GUID.to_bytes(); let expected: [u8; 16] = [221, 107, 224, 255, 7, 97, 166, 70, 123, 178, 90, 156, 126, 197, 39, 92]; let mut i = 0; while i < 16 { assert!(actual[i] == expected[i]); i += 1; } };
const _: () = { let actual = uefi_raw::protocol::memory_protection::MemoryAttributeProtocol::GUID.to_bytes(); let expected: [u8; 16] = [246, 12, 86, 244, 236, 64, 74, 75, 161, 146, 191, 29, 87, 208, 177, 137]; let mut i = 0; while i < 16 { assert!(actual[i] == expected[i]); i += 1; } };
const _: () = { let actual = uefi_raw::protocol::misc::TimestampProtocol::GUID.to_bytes(); let expected: [u8; 16] = [65, 222, 191, 175, 110, 46, 98, 66, 186, 101, 98, 185, 35, 110, 84, 149]; let mut i = 0; while i < 16 { assert!(actual[i] == expected[i]); i += 1; } };
const _: () = { let actual = uefi_raw::protocol::misc::ResetNotificationProtocol::GUID.to_bytes(); let expected: [u8; 16] = [224, 74, 163, 157, 249, 234, 191, 75, 142, 195, 253, 96, 34, 108, 68, 190]; let mut i = 0; while i < 16 { assert!(actual[i] == expected[i]); i += 1; } };
const _: () = { let actual = uefi_raw::protocol::driver::DriverBindingProtocol::GUID.to_bytes(); let expected: [u8; 16] = [171, 49, 160, 24, 67, 180, 26, 77, 165, 192, 12, 9, 38, 30, 159, 113]; let mut i = 0; while i < 16 { assert!(actual[i] == expected[i]); i += 1; } };
const _: () = { let actual = uefi_raw::protocol::driver::ComponentName2Protocol::GUID.to_bytes(); let expected: [u8; 16] = [255, 92, 122, 106, 217, 232, 112, 79, 186, 218, 117, 171, 48, 37, 206, 20]; let mut i = 0; while i < 16 { assert!(actual[i] == expected[i]); i += 1; } };
const _: () = { let actual = uefi_raw::protocol::driver::ComponentName2Protocol::DEPRECATED_COMPONENT_NAME_GUID.to_bytes(); let expected: [u8; 16] = [44, 119, 122, 16, 225, 213, 212, 17, 154, 70, 0, 144, 39, 63, 193, 77]; let mut i = 0; while i < 16 { assert!(actual[i] == expected[i]); i += 1; } };
const _: () = { let actual = uefi_raw::protocol::string::UnicodeCollationProtocol::GUID.to_bytes(); let expected: [u8; 16] = [252, 81, 199, 164, 174, 35, 62, 76, 146, 233, 73, 100, 207, 99, 243, 73]; let mut i = 0; while i < 16 { assert!(actual[i] == expected[i]); i += 1; } };
