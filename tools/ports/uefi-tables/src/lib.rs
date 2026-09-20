#![no_std]
// Host tooling only: compile inert constants for x86_64-unknown-uefi.
use core::mem::{align_of, offset_of, size_of};
use uefi_raw::table::{Header,system::SystemTable,configuration::ConfigurationTable};
use uefi_raw::table::boot::{BootServices,OpenProtocolInformationEntry};
use uefi_raw::table::runtime::{RuntimeServices,TimeCapabilities};

#[used]
#[unsafe(no_mangle)]
pub static CATHEDRAL_TABLE_LAYOUT: [u64; 101] = [
    size_of::<Header>() as u64, // Header.size
    align_of::<Header>() as u64, // Header.alignment
    offset_of!(Header, signature) as u64, // Header.signature.offset
    offset_of!(Header, revision) as u64, // Header.revision.offset
    offset_of!(Header, size) as u64, // Header.size.offset
    offset_of!(Header, crc) as u64, // Header.crc.offset
    offset_of!(Header, reserved) as u64, // Header.reserved.offset
    size_of::<SystemTable>() as u64, // SystemTable.size
    align_of::<SystemTable>() as u64, // SystemTable.alignment
    offset_of!(SystemTable, header) as u64, // SystemTable.header.offset
    offset_of!(SystemTable, firmware_vendor) as u64, // SystemTable.firmware_vendor.offset
    offset_of!(SystemTable, firmware_revision) as u64, // SystemTable.firmware_revision.offset
    offset_of!(SystemTable, stdin_handle) as u64, // SystemTable.stdin_handle.offset
    offset_of!(SystemTable, stdin) as u64, // SystemTable.stdin.offset
    offset_of!(SystemTable, stdout_handle) as u64, // SystemTable.stdout_handle.offset
    offset_of!(SystemTable, stdout) as u64, // SystemTable.stdout.offset
    offset_of!(SystemTable, stderr_handle) as u64, // SystemTable.stderr_handle.offset
    offset_of!(SystemTable, stderr) as u64, // SystemTable.stderr.offset
    offset_of!(SystemTable, runtime_services) as u64, // SystemTable.runtime_services.offset
    offset_of!(SystemTable, boot_services) as u64, // SystemTable.boot_services.offset
    offset_of!(SystemTable, number_of_configuration_table_entries) as u64, // SystemTable.number_of_configuration_table_entries.offset
    offset_of!(SystemTable, configuration_table) as u64, // SystemTable.configuration_table.offset
    size_of::<ConfigurationTable>() as u64, // ConfigurationTable.size
    align_of::<ConfigurationTable>() as u64, // ConfigurationTable.alignment
    offset_of!(ConfigurationTable, vendor_guid) as u64, // ConfigurationTable.vendor_guid.offset
    offset_of!(ConfigurationTable, vendor_table) as u64, // ConfigurationTable.vendor_table.offset
    size_of::<BootServices>() as u64, // BootServices.size
    align_of::<BootServices>() as u64, // BootServices.alignment
    offset_of!(BootServices, header) as u64, // BootServices.header.offset
    offset_of!(BootServices, raise_tpl) as u64, // BootServices.raise_tpl.offset
    offset_of!(BootServices, restore_tpl) as u64, // BootServices.restore_tpl.offset
    offset_of!(BootServices, allocate_pages) as u64, // BootServices.allocate_pages.offset
    offset_of!(BootServices, free_pages) as u64, // BootServices.free_pages.offset
    offset_of!(BootServices, get_memory_map) as u64, // BootServices.get_memory_map.offset
    offset_of!(BootServices, allocate_pool) as u64, // BootServices.allocate_pool.offset
    offset_of!(BootServices, free_pool) as u64, // BootServices.free_pool.offset
    offset_of!(BootServices, create_event) as u64, // BootServices.create_event.offset
    offset_of!(BootServices, set_timer) as u64, // BootServices.set_timer.offset
    offset_of!(BootServices, wait_for_event) as u64, // BootServices.wait_for_event.offset
    offset_of!(BootServices, signal_event) as u64, // BootServices.signal_event.offset
    offset_of!(BootServices, close_event) as u64, // BootServices.close_event.offset
    offset_of!(BootServices, check_event) as u64, // BootServices.check_event.offset
    offset_of!(BootServices, install_protocol_interface) as u64, // BootServices.install_protocol_interface.offset
    offset_of!(BootServices, reinstall_protocol_interface) as u64, // BootServices.reinstall_protocol_interface.offset
    offset_of!(BootServices, uninstall_protocol_interface) as u64, // BootServices.uninstall_protocol_interface.offset
    offset_of!(BootServices, handle_protocol) as u64, // BootServices.handle_protocol.offset
    offset_of!(BootServices, reserved) as u64, // BootServices.reserved.offset
    offset_of!(BootServices, register_protocol_notify) as u64, // BootServices.register_protocol_notify.offset
    offset_of!(BootServices, locate_handle) as u64, // BootServices.locate_handle.offset
    offset_of!(BootServices, locate_device_path) as u64, // BootServices.locate_device_path.offset
    offset_of!(BootServices, install_configuration_table) as u64, // BootServices.install_configuration_table.offset
    offset_of!(BootServices, load_image) as u64, // BootServices.load_image.offset
    offset_of!(BootServices, start_image) as u64, // BootServices.start_image.offset
    offset_of!(BootServices, exit) as u64, // BootServices.exit.offset
    offset_of!(BootServices, unload_image) as u64, // BootServices.unload_image.offset
    offset_of!(BootServices, exit_boot_services) as u64, // BootServices.exit_boot_services.offset
    offset_of!(BootServices, get_next_monotonic_count) as u64, // BootServices.get_next_monotonic_count.offset
    offset_of!(BootServices, stall) as u64, // BootServices.stall.offset
    offset_of!(BootServices, set_watchdog_timer) as u64, // BootServices.set_watchdog_timer.offset
    offset_of!(BootServices, connect_controller) as u64, // BootServices.connect_controller.offset
    offset_of!(BootServices, disconnect_controller) as u64, // BootServices.disconnect_controller.offset
    offset_of!(BootServices, open_protocol) as u64, // BootServices.open_protocol.offset
    offset_of!(BootServices, close_protocol) as u64, // BootServices.close_protocol.offset
    offset_of!(BootServices, open_protocol_information) as u64, // BootServices.open_protocol_information.offset
    offset_of!(BootServices, protocols_per_handle) as u64, // BootServices.protocols_per_handle.offset
    offset_of!(BootServices, locate_handle_buffer) as u64, // BootServices.locate_handle_buffer.offset
    offset_of!(BootServices, locate_protocol) as u64, // BootServices.locate_protocol.offset
    offset_of!(BootServices, install_multiple_protocol_interfaces) as u64, // BootServices.install_multiple_protocol_interfaces.offset
    offset_of!(BootServices, uninstall_multiple_protocol_interfaces) as u64, // BootServices.uninstall_multiple_protocol_interfaces.offset
    offset_of!(BootServices, calculate_crc32) as u64, // BootServices.calculate_crc32.offset
    offset_of!(BootServices, copy_mem) as u64, // BootServices.copy_mem.offset
    offset_of!(BootServices, set_mem) as u64, // BootServices.set_mem.offset
    offset_of!(BootServices, create_event_ex) as u64, // BootServices.create_event_ex.offset
    size_of::<RuntimeServices>() as u64, // RuntimeServices.size
    align_of::<RuntimeServices>() as u64, // RuntimeServices.alignment
    offset_of!(RuntimeServices, header) as u64, // RuntimeServices.header.offset
    offset_of!(RuntimeServices, get_time) as u64, // RuntimeServices.get_time.offset
    offset_of!(RuntimeServices, set_time) as u64, // RuntimeServices.set_time.offset
    offset_of!(RuntimeServices, get_wakeup_time) as u64, // RuntimeServices.get_wakeup_time.offset
    offset_of!(RuntimeServices, set_wakeup_time) as u64, // RuntimeServices.set_wakeup_time.offset
    offset_of!(RuntimeServices, set_virtual_address_map) as u64, // RuntimeServices.set_virtual_address_map.offset
    offset_of!(RuntimeServices, convert_pointer) as u64, // RuntimeServices.convert_pointer.offset
    offset_of!(RuntimeServices, get_variable) as u64, // RuntimeServices.get_variable.offset
    offset_of!(RuntimeServices, get_next_variable_name) as u64, // RuntimeServices.get_next_variable_name.offset
    offset_of!(RuntimeServices, set_variable) as u64, // RuntimeServices.set_variable.offset
    offset_of!(RuntimeServices, get_next_high_monotonic_count) as u64, // RuntimeServices.get_next_high_monotonic_count.offset
    offset_of!(RuntimeServices, reset_system) as u64, // RuntimeServices.reset_system.offset
    offset_of!(RuntimeServices, update_capsule) as u64, // RuntimeServices.update_capsule.offset
    offset_of!(RuntimeServices, query_capsule_capabilities) as u64, // RuntimeServices.query_capsule_capabilities.offset
    offset_of!(RuntimeServices, query_variable_info) as u64, // RuntimeServices.query_variable_info.offset
    size_of::<OpenProtocolInformationEntry>() as u64, // OpenProtocolInformationEntry.size
    align_of::<OpenProtocolInformationEntry>() as u64, // OpenProtocolInformationEntry.alignment
    offset_of!(OpenProtocolInformationEntry, agent_handle) as u64, // OpenProtocolInformationEntry.agent_handle.offset
    offset_of!(OpenProtocolInformationEntry, controller_handle) as u64, // OpenProtocolInformationEntry.controller_handle.offset
    offset_of!(OpenProtocolInformationEntry, attributes) as u64, // OpenProtocolInformationEntry.attributes.offset
    offset_of!(OpenProtocolInformationEntry, open_count) as u64, // OpenProtocolInformationEntry.open_count.offset
    size_of::<TimeCapabilities>() as u64, // TimeCapabilities.size
    align_of::<TimeCapabilities>() as u64, // TimeCapabilities.alignment
    offset_of!(TimeCapabilities, resolution) as u64, // TimeCapabilities.resolution.offset
    offset_of!(TimeCapabilities, accuracy) as u64, // TimeCapabilities.accuracy.offset
    offset_of!(TimeCapabilities, sets_to_zero) as u64, // TimeCapabilities.sets_to_zero.offset
];

// Companion ABI audit requested by the scalar slice; printed separately and
// deliberately not presented as independently observed Omega geometry.
#[used]
#[unsafe(no_mangle)]
pub static CATHEDRAL_SCALAR_LAYOUT: [u64; 8] = [
    size_of::<uefi_raw::Guid>() as u64,
    align_of::<uefi_raw::Guid>() as u64,
    size_of::<uefi_raw::capsule::CapsuleHeader>() as u64,
    align_of::<uefi_raw::capsule::CapsuleHeader>() as u64,
    size_of::<uefi_raw::time::Time>() as u64,
    align_of::<uefi_raw::time::Time>() as u64,
    size_of::<uefi_raw::table::boot::MemoryDescriptor>() as u64,
    align_of::<uefi_raw::table::boot::MemoryDescriptor>() as u64,
];
