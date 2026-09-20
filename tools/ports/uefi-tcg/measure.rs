// Host conformance tooling, never shipped. No TPM operations execute.
#![no_std]
use core::mem::{size_of, align_of, offset_of};
use uefi_raw::protocol::tcg::{v1::*, v2::*, AlgorithmId, EventType};
#[unsafe(no_mangle)]
pub static CATHEDRAL_TCG_LAYOUT: [u64; 123] = [
    AlgorithmId::AES.0 as u64, // ALGORITHM_AES
    AlgorithmId::ERROR.0 as u64, // ALGORITHM_ERROR
    AlgorithmId::HMAC.0 as u64, // ALGORITHM_HMAC
    AlgorithmId::KEYED_HASH.0 as u64, // ALGORITHM_KEYED_HASH
    AlgorithmId::MGF1.0 as u64, // ALGORITHM_MGF1
    AlgorithmId::NULL.0 as u64, // ALGORITHM_NULL
    AlgorithmId::RSA.0 as u64, // ALGORITHM_RSA
    AlgorithmId::SHA1.0 as u64, // ALGORITHM_SHA1
    AlgorithmId::SHA256.0 as u64, // ALGORITHM_SHA256
    AlgorithmId::SHA384.0 as u64, // ALGORITHM_SHA384
    AlgorithmId::SHA512.0 as u64, // ALGORITHM_SHA512
    AlgorithmId::SM3_256.0 as u64, // ALGORITHM_SM3_256
    AlgorithmId::SM4.0 as u64, // ALGORITHM_SM4
    AlgorithmId::TDES.0 as u64, // ALGORITHM_TDES
    AlgorithmId::XOR.0 as u64, // ALGORITHM_XOR
    align_of::<AlgorithmId>() as u64, // AlgorithmId.alignment
    size_of::<AlgorithmId>() as u64, // AlgorithmId.size
    EventType::ACTION.0 as u64, // EVENT_ACTION
    EventType::COMPACT_HASH.0 as u64, // EVENT_COMPACT_HASH
    EventType::CPU_MICROCODE.0 as u64, // EVENT_CPU_MICROCODE
    EventType::CRTM_CONTENTS.0 as u64, // EVENT_CRTM_CONTENTS
    EventType::CRTM_VERSION.0 as u64, // EVENT_CRTM_VERSION
    EventType::EFI_ACTION.0 as u64, // EVENT_EFI_ACTION
    EventType::EFI_BOOT_SERVICES_APPLICATION.0 as u64, // EVENT_EFI_BOOT_SERVICES_APPLICATION
    EventType::EFI_BOOT_SERVICES_DRIVER.0 as u64, // EVENT_EFI_BOOT_SERVICES_DRIVER
    EventType::EFI_EVENT_BASE.0 as u64, // EVENT_EFI_EVENT_BASE
    EventType::EFI_GPT_EVENT.0 as u64, // EVENT_EFI_GPT_EVENT
    EventType::EFI_HANDOFF_TABLES.0 as u64, // EVENT_EFI_HANDOFF_TABLES
    EventType::EFI_HANDOFF_TABLES2.0 as u64, // EVENT_EFI_HANDOFF_TABLES2
    EventType::EFI_HCRTM_EVENT.0 as u64, // EVENT_EFI_HCRTM_EVENT
    EventType::EFI_PLATFORM_FIRMWARE_BLOB.0 as u64, // EVENT_EFI_PLATFORM_FIRMWARE_BLOB
    EventType::EFI_PLATFORM_FIRMWARE_BLOB2.0 as u64, // EVENT_EFI_PLATFORM_FIRMWARE_BLOB2
    EventType::EFI_RUNTIME_SERVICES_DRIVER.0 as u64, // EVENT_EFI_RUNTIME_SERVICES_DRIVER
    EventType::EFI_SPDM_FIRMWARE_BLOB.0 as u64, // EVENT_EFI_SPDM_FIRMWARE_BLOB
    EventType::EFI_SPDM_FIRMWARE_CONFIG.0 as u64, // EVENT_EFI_SPDM_FIRMWARE_CONFIG
    EventType::EFI_VARIABLE_AUTHORITY.0 as u64, // EVENT_EFI_VARIABLE_AUTHORITY
    EventType::EFI_VARIABLE_BOOT.0 as u64, // EVENT_EFI_VARIABLE_BOOT
    EventType::EFI_VARIABLE_BOOT2.0 as u64, // EVENT_EFI_VARIABLE_BOOT2
    EventType::EFI_VARIABLE_DRIVER_CONFIG.0 as u64, // EVENT_EFI_VARIABLE_DRIVER_CONFIG
    EventType::EVENT_TAG.0 as u64, // EVENT_EVENT_TAG
    EventType::IPL.0 as u64, // EVENT_IPL
    EventType::IPL_PARTITION_DATA.0 as u64, // EVENT_IPL_PARTITION_DATA
    Tcg2EventLogBitmap::TCG_1_2.bits() as u64, // EVENT_LOG_TCG_1_2
    Tcg2EventLogBitmap::TCG_2.bits() as u64, // EVENT_LOG_TCG_2
    EventType::NONHOST_CODE.0 as u64, // EVENT_NONHOST_CODE
    EventType::NONHOST_CONFIG.0 as u64, // EVENT_NONHOST_CONFIG
    EventType::NONHOST_INFO.0 as u64, // EVENT_NONHOST_INFO
    EventType::NO_ACTION.0 as u64, // EVENT_NO_ACTION
    EventType::OMIT_BOOT_DEVICE_EVENTS.0 as u64, // EVENT_OMIT_BOOT_DEVICE_EVENTS
    EventType::PLATFORM_CONFIG_FLAGS.0 as u64, // EVENT_PLATFORM_CONFIG_FLAGS
    EventType::POST_CODE.0 as u64, // EVENT_POST_CODE
    EventType::PREBOOT_CERT.0 as u64, // EVENT_PREBOOT_CERT
    EventType::SEPARATOR.0 as u64, // EVENT_SEPARATOR
    EventType::TABLE_OF_DEVICES.0 as u64, // EVENT_TABLE_OF_DEVICES
    EventType::UNUSED.0 as u64, // EVENT_UNUSED
    align_of::<EventType>() as u64, // EventType.alignment
    size_of::<EventType>() as u64, // EventType.size
    Tcg2HashLogExtendEventFlags::EFI_TCG2_EXTEND_ONLY.bits() as u64, // HASH_LOG_EXTEND_ONLY
    Tcg2HashLogExtendEventFlags::PE_COFF_IMAGE.bits() as u64, // HASH_LOG_PE_COFF_IMAGE
    Tcg2HashAlgorithmBitmap::SHA1.bits() as u64, // HASH_SHA1
    Tcg2HashAlgorithmBitmap::SHA256.bits() as u64, // HASH_SHA256
    Tcg2HashAlgorithmBitmap::SHA384.bits() as u64, // HASH_SHA384
    Tcg2HashAlgorithmBitmap::SHA512.bits() as u64, // HASH_SHA512
    Tcg2HashAlgorithmBitmap::SM3_256.bits() as u64, // HASH_SM3_256
    offset_of!(Tcg2BootServiceCapability, active_pcr_banks) as u64, // Tcg2BootServiceCapability.active_pcr_banks.offset
    align_of::<Tcg2BootServiceCapability>() as u64, // Tcg2BootServiceCapability.alignment
    offset_of!(Tcg2BootServiceCapability, hash_algorithm_bitmap) as u64, // Tcg2BootServiceCapability.hash_algorithm_bitmap.offset
    offset_of!(Tcg2BootServiceCapability, manufacturer_id) as u64, // Tcg2BootServiceCapability.manufacturer_id.offset
    offset_of!(Tcg2BootServiceCapability, max_command_size) as u64, // Tcg2BootServiceCapability.max_command_size.offset
    offset_of!(Tcg2BootServiceCapability, max_response_size) as u64, // Tcg2BootServiceCapability.max_response_size.offset
    offset_of!(Tcg2BootServiceCapability, number_of_pcr_banks) as u64, // Tcg2BootServiceCapability.number_of_pcr_banks.offset
    offset_of!(Tcg2BootServiceCapability, protocol_version) as u64, // Tcg2BootServiceCapability.protocol_version.offset
    size_of::<Tcg2BootServiceCapability>() as u64, // Tcg2BootServiceCapability.size
    offset_of!(Tcg2BootServiceCapability, size) as u64, // Tcg2BootServiceCapability.size.offset
    offset_of!(Tcg2BootServiceCapability, structure_version) as u64, // Tcg2BootServiceCapability.structure_version.offset
    offset_of!(Tcg2BootServiceCapability, supported_event_logs) as u64, // Tcg2BootServiceCapability.supported_event_logs.offset
    offset_of!(Tcg2BootServiceCapability, tpm_present_flag) as u64, // Tcg2BootServiceCapability.tpm_present_flag.offset
    align_of::<Tcg2EventHeader>() as u64, // Tcg2EventHeader.alignment
    offset_of!(Tcg2EventHeader, event_type) as u64, // Tcg2EventHeader.event_type.offset
    offset_of!(Tcg2EventHeader, header_size) as u64, // Tcg2EventHeader.header_size.offset
    offset_of!(Tcg2EventHeader, header_version) as u64, // Tcg2EventHeader.header_version.offset
    offset_of!(Tcg2EventHeader, pcr_index) as u64, // Tcg2EventHeader.pcr_index.offset
    size_of::<Tcg2EventHeader>() as u64, // Tcg2EventHeader.size
    align_of::<Tcg2EventLogBitmap>() as u64, // Tcg2EventLogBitmap.alignment
    size_of::<Tcg2EventLogBitmap>() as u64, // Tcg2EventLogBitmap.size
    align_of::<Tcg2HashAlgorithmBitmap>() as u64, // Tcg2HashAlgorithmBitmap.alignment
    size_of::<Tcg2HashAlgorithmBitmap>() as u64, // Tcg2HashAlgorithmBitmap.size
    align_of::<Tcg2HashLogExtendEventFlags>() as u64, // Tcg2HashLogExtendEventFlags.alignment
    size_of::<Tcg2HashLogExtendEventFlags>() as u64, // Tcg2HashLogExtendEventFlags.size
    align_of::<Tcg2Protocol>() as u64, // Tcg2Protocol.alignment
    offset_of!(Tcg2Protocol, get_active_pcr_banks) as u64, // Tcg2Protocol.get_active_pcr_banks.offset
    offset_of!(Tcg2Protocol, get_capability) as u64, // Tcg2Protocol.get_capability.offset
    offset_of!(Tcg2Protocol, get_event_log) as u64, // Tcg2Protocol.get_event_log.offset
    offset_of!(Tcg2Protocol, get_result_of_set_active_pcr_banks) as u64, // Tcg2Protocol.get_result_of_set_active_pcr_banks.offset
    offset_of!(Tcg2Protocol, hash_log_extend_event) as u64, // Tcg2Protocol.hash_log_extend_event.offset
    offset_of!(Tcg2Protocol, set_active_pcr_banks) as u64, // Tcg2Protocol.set_active_pcr_banks.offset
    size_of::<Tcg2Protocol>() as u64, // Tcg2Protocol.size
    offset_of!(Tcg2Protocol, submit_command) as u64, // Tcg2Protocol.submit_command.offset
    align_of::<Tcg2Version>() as u64, // Tcg2Version.alignment
    offset_of!(Tcg2Version, major) as u64, // Tcg2Version.major.offset
    offset_of!(Tcg2Version, minor) as u64, // Tcg2Version.minor.offset
    size_of::<Tcg2Version>() as u64, // Tcg2Version.size
    align_of::<TcgBootServiceCapability>() as u64, // TcgBootServiceCapability.alignment
    offset_of!(TcgBootServiceCapability, hash_algorithm_bitmap) as u64, // TcgBootServiceCapability.hash_algorithm_bitmap.offset
    offset_of!(TcgBootServiceCapability, protocol_spec_version) as u64, // TcgBootServiceCapability.protocol_spec_version.offset
    size_of::<TcgBootServiceCapability>() as u64, // TcgBootServiceCapability.size
    offset_of!(TcgBootServiceCapability, size) as u64, // TcgBootServiceCapability.size.offset
    offset_of!(TcgBootServiceCapability, structure_version) as u64, // TcgBootServiceCapability.structure_version.offset
    offset_of!(TcgBootServiceCapability, tpm_deactivated_flag) as u64, // TcgBootServiceCapability.tpm_deactivated_flag.offset
    offset_of!(TcgBootServiceCapability, tpm_present_flag) as u64, // TcgBootServiceCapability.tpm_present_flag.offset
    align_of::<TcgProtocol>() as u64, // TcgProtocol.alignment
    offset_of!(TcgProtocol, hash_all) as u64, // TcgProtocol.hash_all.offset
    offset_of!(TcgProtocol, hash_log_extend_event) as u64, // TcgProtocol.hash_log_extend_event.offset
    offset_of!(TcgProtocol, log_event) as u64, // TcgProtocol.log_event.offset
    offset_of!(TcgProtocol, pass_through_to_tpm) as u64, // TcgProtocol.pass_through_to_tpm.offset
    size_of::<TcgProtocol>() as u64, // TcgProtocol.size
    offset_of!(TcgProtocol, status_check) as u64, // TcgProtocol.status_check.offset
    align_of::<TcgVersion>() as u64, // TcgVersion.alignment
    offset_of!(TcgVersion, major) as u64, // TcgVersion.major.offset
    offset_of!(TcgVersion, minor) as u64, // TcgVersion.minor.offset
    offset_of!(TcgVersion, rev_major) as u64, // TcgVersion.rev_major.offset
    offset_of!(TcgVersion, rev_minor) as u64, // TcgVersion.rev_minor.offset
    size_of::<TcgVersion>() as u64, // TcgVersion.size
];
