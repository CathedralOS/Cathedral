// SPDX-License-Identifier: MIT OR Apache-2.0
#![no_std]
use core::mem::{size_of, align_of, offset_of};
#[unsafe(no_mangle)]
pub static CATHEDRAL_SHELL_LAYOUT: [u64; 64] = [
    size_of::<uefi_raw::protocol::shell::ListEntry>() as u64, // ListEntry.size
    align_of::<uefi_raw::protocol::shell::ListEntry>() as u64, // ListEntry.alignment
    offset_of!(uefi_raw::protocol::shell::ListEntry, f_link) as u64, // ListEntry.f_link.offset
    offset_of!(uefi_raw::protocol::shell::ListEntry, b_link) as u64, // ListEntry.b_link.offset
    size_of::<uefi_raw::protocol::shell::ShellFileInfo>() as u64, // ShellFileInfo.size
    align_of::<uefi_raw::protocol::shell::ShellFileInfo>() as u64, // ShellFileInfo.alignment
    offset_of!(uefi_raw::protocol::shell::ShellFileInfo, link) as u64, // ShellFileInfo.link.offset
    offset_of!(uefi_raw::protocol::shell::ShellFileInfo, status) as u64, // ShellFileInfo.status.offset
    offset_of!(uefi_raw::protocol::shell::ShellFileInfo, full_name) as u64, // ShellFileInfo.full_name.offset
    offset_of!(uefi_raw::protocol::shell::ShellFileInfo, file_name) as u64, // ShellFileInfo.file_name.offset
    offset_of!(uefi_raw::protocol::shell::ShellFileInfo, handle) as u64, // ShellFileInfo.handle.offset
    offset_of!(uefi_raw::protocol::shell::ShellFileInfo, info) as u64, // ShellFileInfo.info.offset
    size_of::<uefi_raw::protocol::shell::ShellDeviceNameFlags>() as u64, // ShellDeviceNameFlags.size
    align_of::<uefi_raw::protocol::shell::ShellDeviceNameFlags>() as u64, // ShellDeviceNameFlags.alignment
    size_of::<uefi_raw::protocol::shell::ShellProtocol>() as u64, // ShellProtocol.size
    align_of::<uefi_raw::protocol::shell::ShellProtocol>() as u64, // ShellProtocol.alignment
    offset_of!(uefi_raw::protocol::shell::ShellProtocol, execute) as u64, // ShellProtocol.execute.offset
    offset_of!(uefi_raw::protocol::shell::ShellProtocol, get_env) as u64, // ShellProtocol.get_env.offset
    offset_of!(uefi_raw::protocol::shell::ShellProtocol, set_env) as u64, // ShellProtocol.set_env.offset
    offset_of!(uefi_raw::protocol::shell::ShellProtocol, get_alias) as u64, // ShellProtocol.get_alias.offset
    offset_of!(uefi_raw::protocol::shell::ShellProtocol, set_alias) as u64, // ShellProtocol.set_alias.offset
    offset_of!(uefi_raw::protocol::shell::ShellProtocol, get_help_text) as u64, // ShellProtocol.get_help_text.offset
    offset_of!(uefi_raw::protocol::shell::ShellProtocol, get_device_path_from_map) as u64, // ShellProtocol.get_device_path_from_map.offset
    offset_of!(uefi_raw::protocol::shell::ShellProtocol, get_map_from_device_path) as u64, // ShellProtocol.get_map_from_device_path.offset
    offset_of!(uefi_raw::protocol::shell::ShellProtocol, get_device_path_from_file_path) as u64, // ShellProtocol.get_device_path_from_file_path.offset
    offset_of!(uefi_raw::protocol::shell::ShellProtocol, get_file_path_from_device_path) as u64, // ShellProtocol.get_file_path_from_device_path.offset
    offset_of!(uefi_raw::protocol::shell::ShellProtocol, set_map) as u64, // ShellProtocol.set_map.offset
    offset_of!(uefi_raw::protocol::shell::ShellProtocol, get_cur_dir) as u64, // ShellProtocol.get_cur_dir.offset
    offset_of!(uefi_raw::protocol::shell::ShellProtocol, set_cur_dir) as u64, // ShellProtocol.set_cur_dir.offset
    offset_of!(uefi_raw::protocol::shell::ShellProtocol, open_file_list) as u64, // ShellProtocol.open_file_list.offset
    offset_of!(uefi_raw::protocol::shell::ShellProtocol, free_file_list) as u64, // ShellProtocol.free_file_list.offset
    offset_of!(uefi_raw::protocol::shell::ShellProtocol, remove_dup_in_file_list) as u64, // ShellProtocol.remove_dup_in_file_list.offset
    offset_of!(uefi_raw::protocol::shell::ShellProtocol, batch_is_active) as u64, // ShellProtocol.batch_is_active.offset
    offset_of!(uefi_raw::protocol::shell::ShellProtocol, is_root_shell) as u64, // ShellProtocol.is_root_shell.offset
    offset_of!(uefi_raw::protocol::shell::ShellProtocol, enable_page_break) as u64, // ShellProtocol.enable_page_break.offset
    offset_of!(uefi_raw::protocol::shell::ShellProtocol, disable_page_break) as u64, // ShellProtocol.disable_page_break.offset
    offset_of!(uefi_raw::protocol::shell::ShellProtocol, get_page_break) as u64, // ShellProtocol.get_page_break.offset
    offset_of!(uefi_raw::protocol::shell::ShellProtocol, get_device_name) as u64, // ShellProtocol.get_device_name.offset
    offset_of!(uefi_raw::protocol::shell::ShellProtocol, get_file_info) as u64, // ShellProtocol.get_file_info.offset
    offset_of!(uefi_raw::protocol::shell::ShellProtocol, set_file_info) as u64, // ShellProtocol.set_file_info.offset
    offset_of!(uefi_raw::protocol::shell::ShellProtocol, open_file_by_name) as u64, // ShellProtocol.open_file_by_name.offset
    offset_of!(uefi_raw::protocol::shell::ShellProtocol, close_file) as u64, // ShellProtocol.close_file.offset
    offset_of!(uefi_raw::protocol::shell::ShellProtocol, create_file) as u64, // ShellProtocol.create_file.offset
    offset_of!(uefi_raw::protocol::shell::ShellProtocol, read_file) as u64, // ShellProtocol.read_file.offset
    offset_of!(uefi_raw::protocol::shell::ShellProtocol, write_file) as u64, // ShellProtocol.write_file.offset
    offset_of!(uefi_raw::protocol::shell::ShellProtocol, delete_file) as u64, // ShellProtocol.delete_file.offset
    offset_of!(uefi_raw::protocol::shell::ShellProtocol, delete_file_by_name) as u64, // ShellProtocol.delete_file_by_name.offset
    offset_of!(uefi_raw::protocol::shell::ShellProtocol, get_file_position) as u64, // ShellProtocol.get_file_position.offset
    offset_of!(uefi_raw::protocol::shell::ShellProtocol, set_file_position) as u64, // ShellProtocol.set_file_position.offset
    offset_of!(uefi_raw::protocol::shell::ShellProtocol, flush_file) as u64, // ShellProtocol.flush_file.offset
    offset_of!(uefi_raw::protocol::shell::ShellProtocol, find_files) as u64, // ShellProtocol.find_files.offset
    offset_of!(uefi_raw::protocol::shell::ShellProtocol, find_files_in_dir) as u64, // ShellProtocol.find_files_in_dir.offset
    offset_of!(uefi_raw::protocol::shell::ShellProtocol, get_file_size) as u64, // ShellProtocol.get_file_size.offset
    offset_of!(uefi_raw::protocol::shell::ShellProtocol, open_root) as u64, // ShellProtocol.open_root.offset
    offset_of!(uefi_raw::protocol::shell::ShellProtocol, open_root_by_handle) as u64, // ShellProtocol.open_root_by_handle.offset
    offset_of!(uefi_raw::protocol::shell::ShellProtocol, execution_break) as u64, // ShellProtocol.execution_break.offset
    offset_of!(uefi_raw::protocol::shell::ShellProtocol, major_version) as u64, // ShellProtocol.major_version.offset
    offset_of!(uefi_raw::protocol::shell::ShellProtocol, minor_version) as u64, // ShellProtocol.minor_version.offset
    offset_of!(uefi_raw::protocol::shell::ShellProtocol, register_guid_name) as u64, // ShellProtocol.register_guid_name.offset
    offset_of!(uefi_raw::protocol::shell::ShellProtocol, get_guid_name) as u64, // ShellProtocol.get_guid_name.offset
    offset_of!(uefi_raw::protocol::shell::ShellProtocol, get_guid_from_name) as u64, // ShellProtocol.get_guid_from_name.offset
    offset_of!(uefi_raw::protocol::shell::ShellProtocol, get_env_ex) as u64, // ShellProtocol.get_env_ex.offset
    uefi_raw::protocol::shell::ShellDeviceNameFlags::USE_COMPONENT_NAME.bits() as u64, // SHELL_DEVICE_NAME_FLAGS_USE_COMPONENT_NAME
    uefi_raw::protocol::shell::ShellDeviceNameFlags::USE_DEVICE_PATH.bits() as u64, // SHELL_DEVICE_NAME_FLAGS_USE_DEVICE_PATH
];
const _: () = { let actual = uefi_raw::protocol::shell::ShellProtocol::GUID.to_bytes(); let expected: [u8; 16] = [8, 208, 2, 99, 155, 127, 48, 79, 135, 172, 96, 201, 254, 245, 218, 78]; let mut i = 0; while i < 16 { assert!(actual[i] == expected[i]); i += 1; } };
