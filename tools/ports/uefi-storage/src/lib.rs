#![no_std]
// Host-only cross-target evidence; no firmware calls or copied Rust vendor source.
use core::mem::{size_of, align_of, offset_of};
#[used]
#[unsafe(no_mangle)]
pub static CATHEDRAL_STORAGE_LAYOUT: [u64; 657] = [
    size_of::<uefi_raw::protocol::block::Lba>() as u64, // Lba.size
    align_of::<uefi_raw::protocol::block::Lba>() as u64, // Lba.alignment
    size_of::<uefi_raw::protocol::file_system::FileProtocolRevision>() as u64, // FileProtocolRevision.size
    align_of::<uefi_raw::protocol::file_system::FileProtocolRevision>() as u64, // FileProtocolRevision.alignment
    size_of::<uefi_raw::protocol::file_system::FileAttribute>() as u64, // FileAttribute.size
    align_of::<uefi_raw::protocol::file_system::FileAttribute>() as u64, // FileAttribute.alignment
    size_of::<uefi_raw::protocol::file_system::FileMode>() as u64, // FileMode.size
    align_of::<uefi_raw::protocol::file_system::FileMode>() as u64, // FileMode.alignment
    size_of::<uefi_raw::protocol::ata::AtaPassThruAttributes>() as u64, // AtaPassThruAttributes.size
    align_of::<uefi_raw::protocol::ata::AtaPassThruAttributes>() as u64, // AtaPassThruAttributes.alignment
    size_of::<uefi_raw::protocol::ata::AtaPassThruCommandProtocol>() as u64, // AtaPassThruCommandProtocol.size
    align_of::<uefi_raw::protocol::ata::AtaPassThruCommandProtocol>() as u64, // AtaPassThruCommandProtocol.alignment
    size_of::<uefi_raw::protocol::ata::AtaPassThruLength>() as u64, // AtaPassThruLength.size
    align_of::<uefi_raw::protocol::ata::AtaPassThruLength>() as u64, // AtaPassThruLength.alignment
    size_of::<uefi_raw::protocol::scsi::ScsiIoType>() as u64, // ScsiIoType.size
    align_of::<uefi_raw::protocol::scsi::ScsiIoType>() as u64, // ScsiIoType.alignment
    size_of::<uefi_raw::protocol::scsi::ScsiIoDataDirection>() as u64, // ScsiIoDataDirection.size
    align_of::<uefi_raw::protocol::scsi::ScsiIoDataDirection>() as u64, // ScsiIoDataDirection.alignment
    size_of::<uefi_raw::protocol::scsi::ScsiIoHostAdapterStatus>() as u64, // ScsiIoHostAdapterStatus.size
    align_of::<uefi_raw::protocol::scsi::ScsiIoHostAdapterStatus>() as u64, // ScsiIoHostAdapterStatus.alignment
    size_of::<uefi_raw::protocol::scsi::ScsiIoTargetStatus>() as u64, // ScsiIoTargetStatus.size
    align_of::<uefi_raw::protocol::scsi::ScsiIoTargetStatus>() as u64, // ScsiIoTargetStatus.alignment
    size_of::<uefi_raw::protocol::nvme::NvmExpressCommandCdwValidity>() as u64, // NvmExpressCommandCdwValidity.size
    align_of::<uefi_raw::protocol::nvme::NvmExpressCommandCdwValidity>() as u64, // NvmExpressCommandCdwValidity.alignment
    size_of::<uefi_raw::protocol::nvme::NvmExpressPassThruAttributes>() as u64, // NvmExpressPassThruAttributes.size
    align_of::<uefi_raw::protocol::nvme::NvmExpressPassThruAttributes>() as u64, // NvmExpressPassThruAttributes.alignment
    size_of::<uefi_raw::protocol::nvme::NvmExpressQueueType>() as u64, // NvmExpressQueueType.size
    align_of::<uefi_raw::protocol::nvme::NvmExpressQueueType>() as u64, // NvmExpressQueueType.alignment
    size_of::<uefi_raw::protocol::firmware_volume::FvAttributes>() as u64, // FvAttributes.size
    align_of::<uefi_raw::protocol::firmware_volume::FvAttributes>() as u64, // FvAttributes.alignment
    size_of::<uefi_raw::protocol::firmware_volume::FvFileAttributes>() as u64, // FvFileAttributes.size
    align_of::<uefi_raw::protocol::firmware_volume::FvFileAttributes>() as u64, // FvFileAttributes.alignment
    size_of::<uefi_raw::protocol::firmware_volume::FvWritePolicy>() as u64, // FvWritePolicy.size
    align_of::<uefi_raw::protocol::firmware_volume::FvWritePolicy>() as u64, // FvWritePolicy.alignment
    size_of::<uefi_raw::protocol::firmware_volume::FvFiletype>() as u64, // FvFiletype.size
    align_of::<uefi_raw::protocol::firmware_volume::FvFiletype>() as u64, // FvFiletype.alignment
    size_of::<uefi_raw::protocol::firmware_volume::SectionType>() as u64, // SectionType.size
    align_of::<uefi_raw::protocol::firmware_volume::SectionType>() as u64, // SectionType.alignment
    size_of::<uefi_raw::protocol::firmware_management::CapsuleSupport>() as u64, // CapsuleSupport.size
    align_of::<uefi_raw::protocol::firmware_management::CapsuleSupport>() as u64, // CapsuleSupport.alignment
    size_of::<uefi_raw::protocol::firmware_management::FmpDep>() as u64, // FmpDep.size
    align_of::<uefi_raw::protocol::firmware_management::FmpDep>() as u64, // FmpDep.alignment
    size_of::<uefi_raw::protocol::firmware_management::ImageAttributes>() as u64, // ImageAttributes.size
    align_of::<uefi_raw::protocol::firmware_management::ImageAttributes>() as u64, // ImageAttributes.alignment
    size_of::<uefi_raw::protocol::firmware_management::ImageCompatibilities>() as u64, // ImageCompatibilities.size
    align_of::<uefi_raw::protocol::firmware_management::ImageCompatibilities>() as u64, // ImageCompatibilities.alignment
    size_of::<uefi_raw::protocol::firmware_management::ImageUpdatable>() as u64, // ImageUpdatable.size
    align_of::<uefi_raw::protocol::firmware_management::ImageUpdatable>() as u64, // ImageUpdatable.alignment
    size_of::<uefi_raw::protocol::firmware_management::PackageAttributes>() as u64, // PackageAttributes.size
    align_of::<uefi_raw::protocol::firmware_management::PackageAttributes>() as u64, // PackageAttributes.alignment
    size_of::<uefi_raw::firmware_storage::FirmwareVolumeAttributes>() as u64, // FirmwareVolumeAttributes.size
    align_of::<uefi_raw::firmware_storage::FirmwareVolumeAttributes>() as u64, // FirmwareVolumeAttributes.alignment
    uefi_raw::protocol::file_system::FileProtocolRevision::REVISION_1.0 as u64, // FILE_PROTOCOL_REVISION_REVISION_1
    uefi_raw::protocol::file_system::FileProtocolRevision::REVISION_2.0 as u64, // FILE_PROTOCOL_REVISION_REVISION_2
    uefi_raw::protocol::file_system::FileAttribute::READ_ONLY.bits() as u64, // FILE_ATTRIBUTE_READ_ONLY
    uefi_raw::protocol::file_system::FileAttribute::HIDDEN.bits() as u64, // FILE_ATTRIBUTE_HIDDEN
    uefi_raw::protocol::file_system::FileAttribute::SYSTEM.bits() as u64, // FILE_ATTRIBUTE_SYSTEM
    uefi_raw::protocol::file_system::FileAttribute::DIRECTORY.bits() as u64, // FILE_ATTRIBUTE_DIRECTORY
    uefi_raw::protocol::file_system::FileAttribute::ARCHIVE.bits() as u64, // FILE_ATTRIBUTE_ARCHIVE
    uefi_raw::protocol::file_system::FileAttribute::VALID_ATTR.bits() as u64, // FILE_ATTRIBUTE_VALID_ATTR
    uefi_raw::protocol::file_system::FileMode::READ.bits() as u64, // FILE_MODE_READ
    uefi_raw::protocol::file_system::FileMode::WRITE.bits() as u64, // FILE_MODE_WRITE
    uefi_raw::protocol::file_system::FileMode::CREATE.bits() as u64, // FILE_MODE_CREATE
    uefi_raw::protocol::block::BlockIoProtocol::REVISION as u64, // BLOCK_IO_PROTOCOL_REVISION
    uefi_raw::protocol::block::BlockIoProtocol::REVISION_2 as u64, // BLOCK_IO_PROTOCOL_REVISION_2
    uefi_raw::protocol::block::BlockIoProtocol::REVISION_3 as u64, // BLOCK_IO_PROTOCOL_REVISION_3
    uefi_raw::protocol::disk::DiskIoProtocol::REVISION as u64, // DISK_IO_PROTOCOL_REVISION
    uefi_raw::protocol::disk::DiskIo2Protocol::REVISION as u64, // DISK_IO2_PROTOCOL_REVISION
    uefi_raw::protocol::ata::AtaPassThruAttributes::PHYSICAL.bits() as u64, // ATA_PASS_THRU_ATTRIBUTES_PHYSICAL
    uefi_raw::protocol::ata::AtaPassThruAttributes::LOGICAL.bits() as u64, // ATA_PASS_THRU_ATTRIBUTES_LOGICAL
    uefi_raw::protocol::ata::AtaPassThruAttributes::NONBLOCKIO.bits() as u64, // ATA_PASS_THRU_ATTRIBUTES_NONBLOCKIO
    uefi_raw::protocol::ata::AtaPassThruCommandProtocol::ATA_HARDWARE_RESET.0 as u64, // ATA_PASS_THRU_COMMAND_PROTOCOL_ATA_HARDWARE_RESET
    uefi_raw::protocol::ata::AtaPassThruCommandProtocol::ATA_SOFTWARE_RESET.0 as u64, // ATA_PASS_THRU_COMMAND_PROTOCOL_ATA_SOFTWARE_RESET
    uefi_raw::protocol::ata::AtaPassThruCommandProtocol::ATA_NON_DATA.0 as u64, // ATA_PASS_THRU_COMMAND_PROTOCOL_ATA_NON_DATA
    uefi_raw::protocol::ata::AtaPassThruCommandProtocol::PIO_DATA_IN.0 as u64, // ATA_PASS_THRU_COMMAND_PROTOCOL_PIO_DATA_IN
    uefi_raw::protocol::ata::AtaPassThruCommandProtocol::PIO_DATA_OUT.0 as u64, // ATA_PASS_THRU_COMMAND_PROTOCOL_PIO_DATA_OUT
    uefi_raw::protocol::ata::AtaPassThruCommandProtocol::DMA.0 as u64, // ATA_PASS_THRU_COMMAND_PROTOCOL_DMA
    uefi_raw::protocol::ata::AtaPassThruCommandProtocol::DMA_QUEUED.0 as u64, // ATA_PASS_THRU_COMMAND_PROTOCOL_DMA_QUEUED
    uefi_raw::protocol::ata::AtaPassThruCommandProtocol::DEVICE_DIAGNOSTIC.0 as u64, // ATA_PASS_THRU_COMMAND_PROTOCOL_DEVICE_DIAGNOSTIC
    uefi_raw::protocol::ata::AtaPassThruCommandProtocol::DEVICE_RESET.0 as u64, // ATA_PASS_THRU_COMMAND_PROTOCOL_DEVICE_RESET
    uefi_raw::protocol::ata::AtaPassThruCommandProtocol::UDMA_DATA_IN.0 as u64, // ATA_PASS_THRU_COMMAND_PROTOCOL_UDMA_DATA_IN
    uefi_raw::protocol::ata::AtaPassThruCommandProtocol::UDMA_DATA_OUT.0 as u64, // ATA_PASS_THRU_COMMAND_PROTOCOL_UDMA_DATA_OUT
    uefi_raw::protocol::ata::AtaPassThruCommandProtocol::FPDMA.0 as u64, // ATA_PASS_THRU_COMMAND_PROTOCOL_FPDMA
    uefi_raw::protocol::ata::AtaPassThruCommandProtocol::RETURN_RESPONSE.0 as u64, // ATA_PASS_THRU_COMMAND_PROTOCOL_RETURN_RESPONSE
    uefi_raw::protocol::ata::AtaPassThruLength::BYTES.0 as u64, // ATA_PASS_THRU_LENGTH_BYTES
    uefi_raw::protocol::ata::AtaPassThruLength::MASK.0 as u64, // ATA_PASS_THRU_LENGTH_MASK
    uefi_raw::protocol::ata::AtaPassThruLength::NO_DATA_TRANSFER.0 as u64, // ATA_PASS_THRU_LENGTH_NO_DATA_TRANSFER
    uefi_raw::protocol::ata::AtaPassThruLength::FEATURES.0 as u64, // ATA_PASS_THRU_LENGTH_FEATURES
    uefi_raw::protocol::ata::AtaPassThruLength::SECTOR_COUNT.0 as u64, // ATA_PASS_THRU_LENGTH_SECTOR_COUNT
    uefi_raw::protocol::ata::AtaPassThruLength::TPSIU.0 as u64, // ATA_PASS_THRU_LENGTH_TPSIU
    uefi_raw::protocol::ata::AtaPassThruLength::COUNT.0 as u64, // ATA_PASS_THRU_LENGTH_COUNT
    uefi_raw::protocol::scsi::ScsiIoType::DISK.0 as u64, // SCSI_IO_TYPE_DISK
    uefi_raw::protocol::scsi::ScsiIoType::TAPE.0 as u64, // SCSI_IO_TYPE_TAPE
    uefi_raw::protocol::scsi::ScsiIoType::PRINTER.0 as u64, // SCSI_IO_TYPE_PRINTER
    uefi_raw::protocol::scsi::ScsiIoType::PROCESSOR.0 as u64, // SCSI_IO_TYPE_PROCESSOR
    uefi_raw::protocol::scsi::ScsiIoType::WRITE_ONCE_READ_MULTIPLE.0 as u64, // SCSI_IO_TYPE_WRITE_ONCE_READ_MULTIPLE
    uefi_raw::protocol::scsi::ScsiIoType::CDROM.0 as u64, // SCSI_IO_TYPE_CDROM
    uefi_raw::protocol::scsi::ScsiIoType::SCANNER.0 as u64, // SCSI_IO_TYPE_SCANNER
    uefi_raw::protocol::scsi::ScsiIoType::OPTICAL.0 as u64, // SCSI_IO_TYPE_OPTICAL
    uefi_raw::protocol::scsi::ScsiIoType::MEDIUM_CHANGER.0 as u64, // SCSI_IO_TYPE_MEDIUM_CHANGER
    uefi_raw::protocol::scsi::ScsiIoType::COMMUNICATION.0 as u64, // SCSI_IO_TYPE_COMMUNICATION
    uefi_raw::protocol::scsi::ScsiIoType::RAID.0 as u64, // SCSI_IO_TYPE_RAID
    uefi_raw::protocol::scsi::ScsiIoType::ENCLOSURE_SERVICES.0 as u64, // SCSI_IO_TYPE_ENCLOSURE_SERVICES
    uefi_raw::protocol::scsi::ScsiIoType::REDUCED_BLOCK_COMMANDS.0 as u64, // SCSI_IO_TYPE_REDUCED_BLOCK_COMMANDS
    uefi_raw::protocol::scsi::ScsiIoType::OPTICAL_CARD_READER_WRITER.0 as u64, // SCSI_IO_TYPE_OPTICAL_CARD_READER_WRITER
    uefi_raw::protocol::scsi::ScsiIoType::BRIDGE_CONTROLLER.0 as u64, // SCSI_IO_TYPE_BRIDGE_CONTROLLER
    uefi_raw::protocol::scsi::ScsiIoType::OBJECT_BASED_STORAGE.0 as u64, // SCSI_IO_TYPE_OBJECT_BASED_STORAGE
    uefi_raw::protocol::scsi::ScsiIoType::RESERVED_LOW.0 as u64, // SCSI_IO_TYPE_RESERVED_LOW
    uefi_raw::protocol::scsi::ScsiIoType::RESERVED_HIGH.0 as u64, // SCSI_IO_TYPE_RESERVED_HIGH
    uefi_raw::protocol::scsi::ScsiIoType::UNKNOWN.0 as u64, // SCSI_IO_TYPE_UNKNOWN
    uefi_raw::protocol::scsi::ScsiIoDataDirection::READ.0 as u64, // SCSI_IO_DATA_DIRECTION_READ
    uefi_raw::protocol::scsi::ScsiIoDataDirection::WRITE.0 as u64, // SCSI_IO_DATA_DIRECTION_WRITE
    uefi_raw::protocol::scsi::ScsiIoDataDirection::BIDIRECTIONAL.0 as u64, // SCSI_IO_DATA_DIRECTION_BIDIRECTIONAL
    uefi_raw::protocol::scsi::ScsiIoHostAdapterStatus::OK.0 as u64, // SCSI_IO_HOST_ADAPTER_STATUS_OK
    uefi_raw::protocol::scsi::ScsiIoHostAdapterStatus::TIMEOUT_COMMAND.0 as u64, // SCSI_IO_HOST_ADAPTER_STATUS_TIMEOUT_COMMAND
    uefi_raw::protocol::scsi::ScsiIoHostAdapterStatus::TIMEOUT.0 as u64, // SCSI_IO_HOST_ADAPTER_STATUS_TIMEOUT
    uefi_raw::protocol::scsi::ScsiIoHostAdapterStatus::MESSAGE_REJECT.0 as u64, // SCSI_IO_HOST_ADAPTER_STATUS_MESSAGE_REJECT
    uefi_raw::protocol::scsi::ScsiIoHostAdapterStatus::BUS_RESET.0 as u64, // SCSI_IO_HOST_ADAPTER_STATUS_BUS_RESET
    uefi_raw::protocol::scsi::ScsiIoHostAdapterStatus::PARITY_ERROR.0 as u64, // SCSI_IO_HOST_ADAPTER_STATUS_PARITY_ERROR
    uefi_raw::protocol::scsi::ScsiIoHostAdapterStatus::REQUEST_SENSE_FAILED.0 as u64, // SCSI_IO_HOST_ADAPTER_STATUS_REQUEST_SENSE_FAILED
    uefi_raw::protocol::scsi::ScsiIoHostAdapterStatus::SELECTION_TIMEOUT.0 as u64, // SCSI_IO_HOST_ADAPTER_STATUS_SELECTION_TIMEOUT
    uefi_raw::protocol::scsi::ScsiIoHostAdapterStatus::DATA_OVERRUN_UNDERRUN.0 as u64, // SCSI_IO_HOST_ADAPTER_STATUS_DATA_OVERRUN_UNDERRUN
    uefi_raw::protocol::scsi::ScsiIoHostAdapterStatus::BUS_FREE.0 as u64, // SCSI_IO_HOST_ADAPTER_STATUS_BUS_FREE
    uefi_raw::protocol::scsi::ScsiIoHostAdapterStatus::PHASE_ERROR.0 as u64, // SCSI_IO_HOST_ADAPTER_STATUS_PHASE_ERROR
    uefi_raw::protocol::scsi::ScsiIoHostAdapterStatus::OTHER.0 as u64, // SCSI_IO_HOST_ADAPTER_STATUS_OTHER
    uefi_raw::protocol::scsi::ScsiIoTargetStatus::GOOD.0 as u64, // SCSI_IO_TARGET_STATUS_GOOD
    uefi_raw::protocol::scsi::ScsiIoTargetStatus::CHECK_CONDITION.0 as u64, // SCSI_IO_TARGET_STATUS_CHECK_CONDITION
    uefi_raw::protocol::scsi::ScsiIoTargetStatus::CONDITION_MET.0 as u64, // SCSI_IO_TARGET_STATUS_CONDITION_MET
    uefi_raw::protocol::scsi::ScsiIoTargetStatus::BUSY.0 as u64, // SCSI_IO_TARGET_STATUS_BUSY
    uefi_raw::protocol::scsi::ScsiIoTargetStatus::INTERMEDIATE.0 as u64, // SCSI_IO_TARGET_STATUS_INTERMEDIATE
    uefi_raw::protocol::scsi::ScsiIoTargetStatus::INTERMEDIATE_CONDITION_MET.0 as u64, // SCSI_IO_TARGET_STATUS_INTERMEDIATE_CONDITION_MET
    uefi_raw::protocol::scsi::ScsiIoTargetStatus::RESERVATION_CONFLICT.0 as u64, // SCSI_IO_TARGET_STATUS_RESERVATION_CONFLICT
    uefi_raw::protocol::scsi::ScsiIoTargetStatus::COMMAND_TERMINATED.0 as u64, // SCSI_IO_TARGET_STATUS_COMMAND_TERMINATED
    uefi_raw::protocol::scsi::ScsiIoTargetStatus::QUEUE_FULL.0 as u64, // SCSI_IO_TARGET_STATUS_QUEUE_FULL
    uefi_raw::protocol::nvme::NvmExpressCommandCdwValidity::CDW_2.bits() as u64, // NVM_EXPRESS_COMMAND_CDW_VALIDITY_CDW_2
    uefi_raw::protocol::nvme::NvmExpressCommandCdwValidity::CDW_3.bits() as u64, // NVM_EXPRESS_COMMAND_CDW_VALIDITY_CDW_3
    uefi_raw::protocol::nvme::NvmExpressCommandCdwValidity::CDW_10.bits() as u64, // NVM_EXPRESS_COMMAND_CDW_VALIDITY_CDW_10
    uefi_raw::protocol::nvme::NvmExpressCommandCdwValidity::CDW_11.bits() as u64, // NVM_EXPRESS_COMMAND_CDW_VALIDITY_CDW_11
    uefi_raw::protocol::nvme::NvmExpressCommandCdwValidity::CDW_12.bits() as u64, // NVM_EXPRESS_COMMAND_CDW_VALIDITY_CDW_12
    uefi_raw::protocol::nvme::NvmExpressCommandCdwValidity::CDW_13.bits() as u64, // NVM_EXPRESS_COMMAND_CDW_VALIDITY_CDW_13
    uefi_raw::protocol::nvme::NvmExpressCommandCdwValidity::CDW_14.bits() as u64, // NVM_EXPRESS_COMMAND_CDW_VALIDITY_CDW_14
    uefi_raw::protocol::nvme::NvmExpressCommandCdwValidity::CDW_15.bits() as u64, // NVM_EXPRESS_COMMAND_CDW_VALIDITY_CDW_15
    uefi_raw::protocol::nvme::NvmExpressPassThruAttributes::PHYSICAL.bits() as u64, // NVM_EXPRESS_PASS_THRU_ATTRIBUTES_PHYSICAL
    uefi_raw::protocol::nvme::NvmExpressPassThruAttributes::LOGICAL.bits() as u64, // NVM_EXPRESS_PASS_THRU_ATTRIBUTES_LOGICAL
    uefi_raw::protocol::nvme::NvmExpressPassThruAttributes::NONBLOCKIO.bits() as u64, // NVM_EXPRESS_PASS_THRU_ATTRIBUTES_NONBLOCKIO
    uefi_raw::protocol::nvme::NvmExpressPassThruAttributes::CMD_SET_NVM.bits() as u64, // NVM_EXPRESS_PASS_THRU_ATTRIBUTES_CMD_SET_NVM
    uefi_raw::protocol::nvme::NvmExpressQueueType::ADMIN.0 as u64, // NVM_EXPRESS_QUEUE_TYPE_ADMIN
    uefi_raw::protocol::nvme::NvmExpressQueueType::IO.0 as u64, // NVM_EXPRESS_QUEUE_TYPE_IO
    uefi_raw::protocol::firmware_volume::FvAttributes::READ_DISABLE_CAP.bits() as u64, // FV_ATTRIBUTES_READ_DISABLE_CAP
    uefi_raw::protocol::firmware_volume::FvAttributes::READ_ENABLE_CAP.bits() as u64, // FV_ATTRIBUTES_READ_ENABLE_CAP
    uefi_raw::protocol::firmware_volume::FvAttributes::READ_STATUS.bits() as u64, // FV_ATTRIBUTES_READ_STATUS
    uefi_raw::protocol::firmware_volume::FvAttributes::WRITE_DISABLE_CAP.bits() as u64, // FV_ATTRIBUTES_WRITE_DISABLE_CAP
    uefi_raw::protocol::firmware_volume::FvAttributes::WRITE_ENABLE_CAP.bits() as u64, // FV_ATTRIBUTES_WRITE_ENABLE_CAP
    uefi_raw::protocol::firmware_volume::FvAttributes::WRITE_STATUS.bits() as u64, // FV_ATTRIBUTES_WRITE_STATUS
    uefi_raw::protocol::firmware_volume::FvAttributes::LOCK_CAP.bits() as u64, // FV_ATTRIBUTES_LOCK_CAP
    uefi_raw::protocol::firmware_volume::FvAttributes::LOCK_STATUS.bits() as u64, // FV_ATTRIBUTES_LOCK_STATUS
    uefi_raw::protocol::firmware_volume::FvAttributes::WRITE_POLICY_RELIABLE.bits() as u64, // FV_ATTRIBUTES_WRITE_POLICY_RELIABLE
    uefi_raw::protocol::firmware_volume::FvAttributes::READ_LOCK_CAP.bits() as u64, // FV_ATTRIBUTES_READ_LOCK_CAP
    uefi_raw::protocol::firmware_volume::FvAttributes::READ_LOCK_STATUS.bits() as u64, // FV_ATTRIBUTES_READ_LOCK_STATUS
    uefi_raw::protocol::firmware_volume::FvAttributes::WRITE_LOCK_CAP.bits() as u64, // FV_ATTRIBUTES_WRITE_LOCK_CAP
    uefi_raw::protocol::firmware_volume::FvAttributes::WRITE_LOCK_STATUS.bits() as u64, // FV_ATTRIBUTES_WRITE_LOCK_STATUS
    uefi_raw::protocol::firmware_volume::FvAttributes::ALIGNMENT.bits() as u64, // FV_ATTRIBUTES_ALIGNMENT
    uefi_raw::protocol::firmware_volume::FvAttributes::ALIGNMENT_1.bits() as u64, // FV_ATTRIBUTES_ALIGNMENT_1
    uefi_raw::protocol::firmware_volume::FvAttributes::ALIGNMENT_2.bits() as u64, // FV_ATTRIBUTES_ALIGNMENT_2
    uefi_raw::protocol::firmware_volume::FvAttributes::ALIGNMENT_4.bits() as u64, // FV_ATTRIBUTES_ALIGNMENT_4
    uefi_raw::protocol::firmware_volume::FvAttributes::ALIGNMENT_8.bits() as u64, // FV_ATTRIBUTES_ALIGNMENT_8
    uefi_raw::protocol::firmware_volume::FvAttributes::ALIGNMENT_16.bits() as u64, // FV_ATTRIBUTES_ALIGNMENT_16
    uefi_raw::protocol::firmware_volume::FvAttributes::ALIGNMENT_32.bits() as u64, // FV_ATTRIBUTES_ALIGNMENT_32
    uefi_raw::protocol::firmware_volume::FvAttributes::ALIGNMENT_64.bits() as u64, // FV_ATTRIBUTES_ALIGNMENT_64
    uefi_raw::protocol::firmware_volume::FvAttributes::ALIGNMENT_128.bits() as u64, // FV_ATTRIBUTES_ALIGNMENT_128
    uefi_raw::protocol::firmware_volume::FvAttributes::ALIGNMENT_256.bits() as u64, // FV_ATTRIBUTES_ALIGNMENT_256
    uefi_raw::protocol::firmware_volume::FvAttributes::ALIGNMENT_512.bits() as u64, // FV_ATTRIBUTES_ALIGNMENT_512
    uefi_raw::protocol::firmware_volume::FvAttributes::ALIGNMENT_1K.bits() as u64, // FV_ATTRIBUTES_ALIGNMENT_1K
    uefi_raw::protocol::firmware_volume::FvAttributes::ALIGNMENT_2K.bits() as u64, // FV_ATTRIBUTES_ALIGNMENT_2K
    uefi_raw::protocol::firmware_volume::FvAttributes::ALIGNMENT_4K.bits() as u64, // FV_ATTRIBUTES_ALIGNMENT_4K
    uefi_raw::protocol::firmware_volume::FvAttributes::ALIGNMENT_8K.bits() as u64, // FV_ATTRIBUTES_ALIGNMENT_8K
    uefi_raw::protocol::firmware_volume::FvAttributes::ALIGNMENT_16K.bits() as u64, // FV_ATTRIBUTES_ALIGNMENT_16K
    uefi_raw::protocol::firmware_volume::FvAttributes::ALIGNMENT_32K.bits() as u64, // FV_ATTRIBUTES_ALIGNMENT_32K
    uefi_raw::protocol::firmware_volume::FvAttributes::ALIGNMENT_64K.bits() as u64, // FV_ATTRIBUTES_ALIGNMENT_64K
    uefi_raw::protocol::firmware_volume::FvAttributes::ALIGNMENT_128K.bits() as u64, // FV_ATTRIBUTES_ALIGNMENT_128K
    uefi_raw::protocol::firmware_volume::FvAttributes::ALIGNMENT_256K.bits() as u64, // FV_ATTRIBUTES_ALIGNMENT_256K
    uefi_raw::protocol::firmware_volume::FvAttributes::ALIGNMENT_512K.bits() as u64, // FV_ATTRIBUTES_ALIGNMENT_512K
    uefi_raw::protocol::firmware_volume::FvAttributes::ALIGNMENT_1M.bits() as u64, // FV_ATTRIBUTES_ALIGNMENT_1M
    uefi_raw::protocol::firmware_volume::FvAttributes::ALIGNMENT_2M.bits() as u64, // FV_ATTRIBUTES_ALIGNMENT_2M
    uefi_raw::protocol::firmware_volume::FvAttributes::ALIGNMENT_4M.bits() as u64, // FV_ATTRIBUTES_ALIGNMENT_4M
    uefi_raw::protocol::firmware_volume::FvAttributes::ALIGNMENT_8M.bits() as u64, // FV_ATTRIBUTES_ALIGNMENT_8M
    uefi_raw::protocol::firmware_volume::FvAttributes::ALIGNMENT_16M.bits() as u64, // FV_ATTRIBUTES_ALIGNMENT_16M
    uefi_raw::protocol::firmware_volume::FvAttributes::ALIGNMENT_32M.bits() as u64, // FV_ATTRIBUTES_ALIGNMENT_32M
    uefi_raw::protocol::firmware_volume::FvAttributes::ALIGNMENT_64M.bits() as u64, // FV_ATTRIBUTES_ALIGNMENT_64M
    uefi_raw::protocol::firmware_volume::FvAttributes::ALIGNMENT_128M.bits() as u64, // FV_ATTRIBUTES_ALIGNMENT_128M
    uefi_raw::protocol::firmware_volume::FvAttributes::ALIGNMENT_256M.bits() as u64, // FV_ATTRIBUTES_ALIGNMENT_256M
    uefi_raw::protocol::firmware_volume::FvAttributes::ALIGNMENT_512M.bits() as u64, // FV_ATTRIBUTES_ALIGNMENT_512M
    uefi_raw::protocol::firmware_volume::FvAttributes::ALIGNMENT_1G.bits() as u64, // FV_ATTRIBUTES_ALIGNMENT_1G
    uefi_raw::protocol::firmware_volume::FvAttributes::ALIGNMENT_2G.bits() as u64, // FV_ATTRIBUTES_ALIGNMENT_2G
    uefi_raw::protocol::firmware_volume::FvFileAttributes::ALIGNMENT.bits() as u64, // FV_FILE_ATTRIBUTES_ALIGNMENT
    uefi_raw::protocol::firmware_volume::FvFileAttributes::FIXED.bits() as u64, // FV_FILE_ATTRIBUTES_FIXED
    uefi_raw::protocol::firmware_volume::FvFileAttributes::MEMORY_MAPPED.bits() as u64, // FV_FILE_ATTRIBUTES_MEMORY_MAPPED
    uefi_raw::protocol::firmware_volume::FvWritePolicy::EFI_FV_UNRELIABLE_WRITE.0 as u64, // FV_WRITE_POLICY_EFI_FV_UNRELIABLE_WRITE
    uefi_raw::protocol::firmware_volume::FvWritePolicy::EFI_FV_RELIABLE_WRITE.0 as u64, // FV_WRITE_POLICY_EFI_FV_RELIABLE_WRITE
    uefi_raw::protocol::firmware_volume::FvFiletype::ALL.0 as u64, // FV_FILETYPE_ALL
    uefi_raw::protocol::firmware_volume::FvFiletype::RAW.0 as u64, // FV_FILETYPE_RAW
    uefi_raw::protocol::firmware_volume::FvFiletype::FREEFORM.0 as u64, // FV_FILETYPE_FREEFORM
    uefi_raw::protocol::firmware_volume::FvFiletype::SECURITY_CORE.0 as u64, // FV_FILETYPE_SECURITY_CORE
    uefi_raw::protocol::firmware_volume::FvFiletype::PEI_CORE.0 as u64, // FV_FILETYPE_PEI_CORE
    uefi_raw::protocol::firmware_volume::FvFiletype::DXE_CORE.0 as u64, // FV_FILETYPE_DXE_CORE
    uefi_raw::protocol::firmware_volume::FvFiletype::PEIM.0 as u64, // FV_FILETYPE_PEIM
    uefi_raw::protocol::firmware_volume::FvFiletype::DRIVER.0 as u64, // FV_FILETYPE_DRIVER
    uefi_raw::protocol::firmware_volume::FvFiletype::COMBINED_PEIM_DRIVER.0 as u64, // FV_FILETYPE_COMBINED_PEIM_DRIVER
    uefi_raw::protocol::firmware_volume::FvFiletype::APPLICATION.0 as u64, // FV_FILETYPE_APPLICATION
    uefi_raw::protocol::firmware_volume::FvFiletype::MM.0 as u64, // FV_FILETYPE_MM
    uefi_raw::protocol::firmware_volume::FvFiletype::FIRMWARE_VOLUME_IMAGE.0 as u64, // FV_FILETYPE_FIRMWARE_VOLUME_IMAGE
    uefi_raw::protocol::firmware_volume::FvFiletype::COMBINED_MM_DXE.0 as u64, // FV_FILETYPE_COMBINED_MM_DXE
    uefi_raw::protocol::firmware_volume::FvFiletype::MM_CORE.0 as u64, // FV_FILETYPE_MM_CORE
    uefi_raw::protocol::firmware_volume::FvFiletype::MM_STANDALONE.0 as u64, // FV_FILETYPE_MM_STANDALONE
    uefi_raw::protocol::firmware_volume::FvFiletype::MM_CORE_STANDALONE.0 as u64, // FV_FILETYPE_MM_CORE_STANDALONE
    uefi_raw::protocol::firmware_volume::FvFiletype::FFS_PAD.0 as u64, // FV_FILETYPE_FFS_PAD
    uefi_raw::protocol::firmware_volume::SectionType::ALL.0 as u64, // SECTION_TYPE_ALL
    uefi_raw::protocol::firmware_volume::SectionType::COMPRESSION.0 as u64, // SECTION_TYPE_COMPRESSION
    uefi_raw::protocol::firmware_volume::SectionType::GUID_DEFINED.0 as u64, // SECTION_TYPE_GUID_DEFINED
    uefi_raw::protocol::firmware_volume::SectionType::DISPOSABLE.0 as u64, // SECTION_TYPE_DISPOSABLE
    uefi_raw::protocol::firmware_volume::SectionType::PE32.0 as u64, // SECTION_TYPE_PE32
    uefi_raw::protocol::firmware_volume::SectionType::PIC.0 as u64, // SECTION_TYPE_PIC
    uefi_raw::protocol::firmware_volume::SectionType::TE.0 as u64, // SECTION_TYPE_TE
    uefi_raw::protocol::firmware_volume::SectionType::DXE_DEPEX.0 as u64, // SECTION_TYPE_DXE_DEPEX
    uefi_raw::protocol::firmware_volume::SectionType::VERSION.0 as u64, // SECTION_TYPE_VERSION
    uefi_raw::protocol::firmware_volume::SectionType::USER_INTERFACE.0 as u64, // SECTION_TYPE_USER_INTERFACE
    uefi_raw::protocol::firmware_volume::SectionType::COMPATIBILITY16.0 as u64, // SECTION_TYPE_COMPATIBILITY16
    uefi_raw::protocol::firmware_volume::SectionType::FIRMWARE_VOLUME_IMAGE.0 as u64, // SECTION_TYPE_FIRMWARE_VOLUME_IMAGE
    uefi_raw::protocol::firmware_volume::SectionType::FREEFORM_SUBTYPE_GUID.0 as u64, // SECTION_TYPE_FREEFORM_SUBTYPE_GUID
    uefi_raw::protocol::firmware_volume::SectionType::RAW.0 as u64, // SECTION_TYPE_RAW
    uefi_raw::protocol::firmware_volume::SectionType::PEI_DEPEX.0 as u64, // SECTION_TYPE_PEI_DEPEX
    uefi_raw::protocol::firmware_volume::SectionType::MM_DEPEX.0 as u64, // SECTION_TYPE_MM_DEPEX
    *uefi_raw::protocol::firmware_volume::FvFiletype::OEM_RANGE.start() as u64, // FV_FILETYPE_OEM_RANGE_MIN
    *uefi_raw::protocol::firmware_volume::FvFiletype::OEM_RANGE.end() as u64, // FV_FILETYPE_OEM_RANGE_MAX
    *uefi_raw::protocol::firmware_volume::FvFiletype::DEBUG_RANGE.start() as u64, // FV_FILETYPE_DEBUG_RANGE_MIN
    *uefi_raw::protocol::firmware_volume::FvFiletype::DEBUG_RANGE.end() as u64, // FV_FILETYPE_DEBUG_RANGE_MAX
    *uefi_raw::protocol::firmware_volume::FvFiletype::FFS_RANGE.start() as u64, // FV_FILETYPE_FFS_RANGE_MIN
    *uefi_raw::protocol::firmware_volume::FvFiletype::FFS_RANGE.end() as u64, // FV_FILETYPE_FFS_RANGE_MAX
    uefi_raw::protocol::firmware_volume::FirmwareVolumeBlock2Protocol::LBA_LIST_TERMINATOR as u64, // FIRMWARE_VOLUME_BLOCK2_PROTOCOL_LBA_LIST_TERMINATOR
    uefi_raw::protocol::firmware_management::CapsuleSupport::AUTHENTICATION.bits() as u64, // CAPSULE_SUPPORT_AUTHENTICATION
    uefi_raw::protocol::firmware_management::CapsuleSupport::DEPENDENCY.bits() as u64, // CAPSULE_SUPPORT_DEPENDENCY
    uefi_raw::protocol::firmware_management::FmpDep::PUSH_GUID.0 as u64, // FMP_DEP_PUSH_GUID
    uefi_raw::protocol::firmware_management::FmpDep::PUSH_VERSION.0 as u64, // FMP_DEP_PUSH_VERSION
    uefi_raw::protocol::firmware_management::FmpDep::VERSION_STR.0 as u64, // FMP_DEP_VERSION_STR
    uefi_raw::protocol::firmware_management::FmpDep::AND.0 as u64, // FMP_DEP_AND
    uefi_raw::protocol::firmware_management::FmpDep::OR.0 as u64, // FMP_DEP_OR
    uefi_raw::protocol::firmware_management::FmpDep::NOT.0 as u64, // FMP_DEP_NOT
    uefi_raw::protocol::firmware_management::FmpDep::TRUE.0 as u64, // FMP_DEP_TRUE
    uefi_raw::protocol::firmware_management::FmpDep::FALSE.0 as u64, // FMP_DEP_FALSE
    uefi_raw::protocol::firmware_management::FmpDep::EQ.0 as u64, // FMP_DEP_EQ
    uefi_raw::protocol::firmware_management::FmpDep::GT.0 as u64, // FMP_DEP_GT
    uefi_raw::protocol::firmware_management::FmpDep::GTE.0 as u64, // FMP_DEP_GTE
    uefi_raw::protocol::firmware_management::FmpDep::LT.0 as u64, // FMP_DEP_LT
    uefi_raw::protocol::firmware_management::FmpDep::LTE.0 as u64, // FMP_DEP_LTE
    uefi_raw::protocol::firmware_management::FmpDep::END.0 as u64, // FMP_DEP_END
    uefi_raw::protocol::firmware_management::FmpDep::DECLARE_LENGTH.0 as u64, // FMP_DEP_DECLARE_LENGTH
    uefi_raw::protocol::firmware_management::ImageAttributes::IMAGE_UPDATABLE.bits() as u64, // IMAGE_ATTRIBUTES_IMAGE_UPDATABLE
    uefi_raw::protocol::firmware_management::ImageAttributes::RESET_REQUIRED.bits() as u64, // IMAGE_ATTRIBUTES_RESET_REQUIRED
    uefi_raw::protocol::firmware_management::ImageAttributes::AUTHENTICATION_REQUIRED.bits() as u64, // IMAGE_ATTRIBUTES_AUTHENTICATION_REQUIRED
    uefi_raw::protocol::firmware_management::ImageAttributes::IN_USE.bits() as u64, // IMAGE_ATTRIBUTES_IN_USE
    uefi_raw::protocol::firmware_management::ImageAttributes::UEFI_IMAGE.bits() as u64, // IMAGE_ATTRIBUTES_UEFI_IMAGE
    uefi_raw::protocol::firmware_management::ImageAttributes::DEPENDENCY.bits() as u64, // IMAGE_ATTRIBUTES_DEPENDENCY
    uefi_raw::protocol::firmware_management::ImageCompatibilities::CHECK_SUPPORTED.bits() as u64, // IMAGE_COMPATIBILITIES_CHECK_SUPPORTED
    uefi_raw::protocol::firmware_management::ImageCompatibilities::all().bits() as u64, // IMAGE_COMPATIBILITIES_ALL_BITS
    uefi_raw::protocol::firmware_management::ImageUpdatable::VALID.bits() as u64, // IMAGE_UPDATABLE_VALID
    uefi_raw::protocol::firmware_management::ImageUpdatable::INVALID.bits() as u64, // IMAGE_UPDATABLE_INVALID
    uefi_raw::protocol::firmware_management::ImageUpdatable::INVALID_TYPE.bits() as u64, // IMAGE_UPDATABLE_INVALID_TYPE
    uefi_raw::protocol::firmware_management::ImageUpdatable::INVALID_OLD.bits() as u64, // IMAGE_UPDATABLE_INVALID_OLD
    uefi_raw::protocol::firmware_management::ImageUpdatable::VALID_WITH_VENDOR_CODE.bits() as u64, // IMAGE_UPDATABLE_VALID_WITH_VENDOR_CODE
    uefi_raw::protocol::firmware_management::PackageAttributes::UPDATABLE.bits() as u64, // PACKAGE_ATTRIBUTES_UPDATABLE
    uefi_raw::protocol::firmware_management::PackageAttributes::RESET_REQUIRED.bits() as u64, // PACKAGE_ATTRIBUTES_RESET_REQUIRED
    uefi_raw::protocol::firmware_management::PackageAttributes::AUTHENTICATION_REQUIRED.bits() as u64, // PACKAGE_ATTRIBUTES_AUTHENTICATION_REQUIRED
    uefi_raw::protocol::firmware_management::FirmwareManagementCapsuleHeader::INIT_VERSION as u64, // FIRMWARE_MANAGEMENT_CAPSULE_HEADER_INIT_VERSION
    uefi_raw::protocol::firmware_management::FirmwareManagementCapsuleImageHeader::INIT_VERSION as u64, // FIRMWARE_MANAGEMENT_CAPSULE_IMAGE_HEADER_INIT_VERSION
    uefi_raw::protocol::firmware_management::FirmwareImageDescriptor::VERSION as u64, // FIRMWARE_IMAGE_DESCRIPTOR_VERSION
    uefi_raw::firmware_storage::FirmwareVolumeAttributes::READ_DISABLED_CAP.bits() as u64, // FIRMWARE_VOLUME_ATTRIBUTES_READ_DISABLED_CAP
    uefi_raw::firmware_storage::FirmwareVolumeAttributes::READ_ENABLED_CAP.bits() as u64, // FIRMWARE_VOLUME_ATTRIBUTES_READ_ENABLED_CAP
    uefi_raw::firmware_storage::FirmwareVolumeAttributes::READ_STATUS.bits() as u64, // FIRMWARE_VOLUME_ATTRIBUTES_READ_STATUS
    uefi_raw::firmware_storage::FirmwareVolumeAttributes::WRITE_DISABLED_CAP.bits() as u64, // FIRMWARE_VOLUME_ATTRIBUTES_WRITE_DISABLED_CAP
    uefi_raw::firmware_storage::FirmwareVolumeAttributes::WRITE_ENABLED_CAP.bits() as u64, // FIRMWARE_VOLUME_ATTRIBUTES_WRITE_ENABLED_CAP
    uefi_raw::firmware_storage::FirmwareVolumeAttributes::WRITE_STATUS.bits() as u64, // FIRMWARE_VOLUME_ATTRIBUTES_WRITE_STATUS
    uefi_raw::firmware_storage::FirmwareVolumeAttributes::LOCK_CAP.bits() as u64, // FIRMWARE_VOLUME_ATTRIBUTES_LOCK_CAP
    uefi_raw::firmware_storage::FirmwareVolumeAttributes::LOCK_STATUS.bits() as u64, // FIRMWARE_VOLUME_ATTRIBUTES_LOCK_STATUS
    uefi_raw::firmware_storage::FirmwareVolumeAttributes::STICKY_WRITE.bits() as u64, // FIRMWARE_VOLUME_ATTRIBUTES_STICKY_WRITE
    uefi_raw::firmware_storage::FirmwareVolumeAttributes::MEMORY_MAPPED.bits() as u64, // FIRMWARE_VOLUME_ATTRIBUTES_MEMORY_MAPPED
    uefi_raw::firmware_storage::FirmwareVolumeAttributes::ERASE_POLARITY.bits() as u64, // FIRMWARE_VOLUME_ATTRIBUTES_ERASE_POLARITY
    uefi_raw::firmware_storage::FirmwareVolumeAttributes::READ_LOCK_CAP.bits() as u64, // FIRMWARE_VOLUME_ATTRIBUTES_READ_LOCK_CAP
    uefi_raw::firmware_storage::FirmwareVolumeAttributes::READ_LOCK_STATUS.bits() as u64, // FIRMWARE_VOLUME_ATTRIBUTES_READ_LOCK_STATUS
    uefi_raw::firmware_storage::FirmwareVolumeAttributes::WRITE_LOCK_CAP.bits() as u64, // FIRMWARE_VOLUME_ATTRIBUTES_WRITE_LOCK_CAP
    uefi_raw::firmware_storage::FirmwareVolumeAttributes::WRITE_LOCK_STATUS.bits() as u64, // FIRMWARE_VOLUME_ATTRIBUTES_WRITE_LOCK_STATUS
    uefi_raw::firmware_storage::FirmwareVolumeAttributes::ALIGNMENT.bits() as u64, // FIRMWARE_VOLUME_ATTRIBUTES_ALIGNMENT
    uefi_raw::firmware_storage::FirmwareVolumeAttributes::WEAK_ALIGNMENT.bits() as u64, // FIRMWARE_VOLUME_ATTRIBUTES_WEAK_ALIGNMENT
    uefi_raw::firmware_storage::FirmwareVolumeAttributes::ALIGNMENT_1.bits() as u64, // FIRMWARE_VOLUME_ATTRIBUTES_ALIGNMENT_1
    uefi_raw::firmware_storage::FirmwareVolumeAttributes::ALIGNMENT_2.bits() as u64, // FIRMWARE_VOLUME_ATTRIBUTES_ALIGNMENT_2
    uefi_raw::firmware_storage::FirmwareVolumeAttributes::ALIGNMENT_4.bits() as u64, // FIRMWARE_VOLUME_ATTRIBUTES_ALIGNMENT_4
    uefi_raw::firmware_storage::FirmwareVolumeAttributes::ALIGNMENT_8.bits() as u64, // FIRMWARE_VOLUME_ATTRIBUTES_ALIGNMENT_8
    uefi_raw::firmware_storage::FirmwareVolumeAttributes::ALIGNMENT_16.bits() as u64, // FIRMWARE_VOLUME_ATTRIBUTES_ALIGNMENT_16
    uefi_raw::firmware_storage::FirmwareVolumeAttributes::ALIGNMENT_32.bits() as u64, // FIRMWARE_VOLUME_ATTRIBUTES_ALIGNMENT_32
    uefi_raw::firmware_storage::FirmwareVolumeAttributes::ALIGNMENT_64.bits() as u64, // FIRMWARE_VOLUME_ATTRIBUTES_ALIGNMENT_64
    uefi_raw::firmware_storage::FirmwareVolumeAttributes::ALIGNMENT_128.bits() as u64, // FIRMWARE_VOLUME_ATTRIBUTES_ALIGNMENT_128
    uefi_raw::firmware_storage::FirmwareVolumeAttributes::ALIGNMENT_256.bits() as u64, // FIRMWARE_VOLUME_ATTRIBUTES_ALIGNMENT_256
    uefi_raw::firmware_storage::FirmwareVolumeAttributes::ALIGNMENT_512.bits() as u64, // FIRMWARE_VOLUME_ATTRIBUTES_ALIGNMENT_512
    uefi_raw::firmware_storage::FirmwareVolumeAttributes::ALIGNMENT_1K.bits() as u64, // FIRMWARE_VOLUME_ATTRIBUTES_ALIGNMENT_1K
    uefi_raw::firmware_storage::FirmwareVolumeAttributes::ALIGNMENT_2K.bits() as u64, // FIRMWARE_VOLUME_ATTRIBUTES_ALIGNMENT_2K
    uefi_raw::firmware_storage::FirmwareVolumeAttributes::ALIGNMENT_4K.bits() as u64, // FIRMWARE_VOLUME_ATTRIBUTES_ALIGNMENT_4K
    uefi_raw::firmware_storage::FirmwareVolumeAttributes::ALIGNMENT_8K.bits() as u64, // FIRMWARE_VOLUME_ATTRIBUTES_ALIGNMENT_8K
    uefi_raw::firmware_storage::FirmwareVolumeAttributes::ALIGNMENT_16K.bits() as u64, // FIRMWARE_VOLUME_ATTRIBUTES_ALIGNMENT_16K
    uefi_raw::firmware_storage::FirmwareVolumeAttributes::ALIGNMENT_32K.bits() as u64, // FIRMWARE_VOLUME_ATTRIBUTES_ALIGNMENT_32K
    uefi_raw::firmware_storage::FirmwareVolumeAttributes::ALIGNMENT_64K.bits() as u64, // FIRMWARE_VOLUME_ATTRIBUTES_ALIGNMENT_64K
    uefi_raw::firmware_storage::FirmwareVolumeAttributes::ALIGNMENT_128K.bits() as u64, // FIRMWARE_VOLUME_ATTRIBUTES_ALIGNMENT_128K
    uefi_raw::firmware_storage::FirmwareVolumeAttributes::ALIGNMENT_256K.bits() as u64, // FIRMWARE_VOLUME_ATTRIBUTES_ALIGNMENT_256K
    uefi_raw::firmware_storage::FirmwareVolumeAttributes::ALIGNMENT_512K.bits() as u64, // FIRMWARE_VOLUME_ATTRIBUTES_ALIGNMENT_512K
    uefi_raw::firmware_storage::FirmwareVolumeAttributes::ALIGNMENT_1M.bits() as u64, // FIRMWARE_VOLUME_ATTRIBUTES_ALIGNMENT_1M
    uefi_raw::firmware_storage::FirmwareVolumeAttributes::ALIGNMENT_2M.bits() as u64, // FIRMWARE_VOLUME_ATTRIBUTES_ALIGNMENT_2M
    uefi_raw::firmware_storage::FirmwareVolumeAttributes::ALIGNMENT_4M.bits() as u64, // FIRMWARE_VOLUME_ATTRIBUTES_ALIGNMENT_4M
    uefi_raw::firmware_storage::FirmwareVolumeAttributes::ALIGNMENT_8M.bits() as u64, // FIRMWARE_VOLUME_ATTRIBUTES_ALIGNMENT_8M
    uefi_raw::firmware_storage::FirmwareVolumeAttributes::ALIGNMENT_16M.bits() as u64, // FIRMWARE_VOLUME_ATTRIBUTES_ALIGNMENT_16M
    uefi_raw::firmware_storage::FirmwareVolumeAttributes::ALIGNMENT_32M.bits() as u64, // FIRMWARE_VOLUME_ATTRIBUTES_ALIGNMENT_32M
    uefi_raw::firmware_storage::FirmwareVolumeAttributes::ALIGNMENT_64M.bits() as u64, // FIRMWARE_VOLUME_ATTRIBUTES_ALIGNMENT_64M
    uefi_raw::firmware_storage::FirmwareVolumeAttributes::ALIGNMENT_128M.bits() as u64, // FIRMWARE_VOLUME_ATTRIBUTES_ALIGNMENT_128M
    uefi_raw::firmware_storage::FirmwareVolumeAttributes::ALIGNMENT_256M.bits() as u64, // FIRMWARE_VOLUME_ATTRIBUTES_ALIGNMENT_256M
    uefi_raw::firmware_storage::FirmwareVolumeAttributes::ALIGNMENT_512M.bits() as u64, // FIRMWARE_VOLUME_ATTRIBUTES_ALIGNMENT_512M
    uefi_raw::firmware_storage::FirmwareVolumeAttributes::ALIGNMENT_1G.bits() as u64, // FIRMWARE_VOLUME_ATTRIBUTES_ALIGNMENT_1G
    uefi_raw::firmware_storage::FirmwareVolumeAttributes::ALIGNMENT_2G.bits() as u64, // FIRMWARE_VOLUME_ATTRIBUTES_ALIGNMENT_2G
    uefi_raw::protocol::scsi::SCSI_TARGET_MAX_BYTES as u64, // SCSI_TARGET_MAX_BYTES
    size_of::<uefi_raw::protocol::file_system::SimpleFileSystemProtocol>() as u64, // SimpleFileSystemProtocol.size
    align_of::<uefi_raw::protocol::file_system::SimpleFileSystemProtocol>() as u64, // SimpleFileSystemProtocol.alignment
    offset_of!(uefi_raw::protocol::file_system::SimpleFileSystemProtocol, revision) as u64, // SimpleFileSystemProtocol.revision.offset
    offset_of!(uefi_raw::protocol::file_system::SimpleFileSystemProtocol, open_volume) as u64, // SimpleFileSystemProtocol.open_volume.offset
    size_of::<uefi_raw::protocol::file_system::FileProtocolV1>() as u64, // FileProtocolV1.size
    align_of::<uefi_raw::protocol::file_system::FileProtocolV1>() as u64, // FileProtocolV1.alignment
    offset_of!(uefi_raw::protocol::file_system::FileProtocolV1, revision) as u64, // FileProtocolV1.revision.offset
    offset_of!(uefi_raw::protocol::file_system::FileProtocolV1, open) as u64, // FileProtocolV1.open.offset
    offset_of!(uefi_raw::protocol::file_system::FileProtocolV1, close) as u64, // FileProtocolV1.close.offset
    offset_of!(uefi_raw::protocol::file_system::FileProtocolV1, delete) as u64, // FileProtocolV1.delete.offset
    offset_of!(uefi_raw::protocol::file_system::FileProtocolV1, read) as u64, // FileProtocolV1.read.offset
    offset_of!(uefi_raw::protocol::file_system::FileProtocolV1, write) as u64, // FileProtocolV1.write.offset
    offset_of!(uefi_raw::protocol::file_system::FileProtocolV1, get_position) as u64, // FileProtocolV1.get_position.offset
    offset_of!(uefi_raw::protocol::file_system::FileProtocolV1, set_position) as u64, // FileProtocolV1.set_position.offset
    offset_of!(uefi_raw::protocol::file_system::FileProtocolV1, get_info) as u64, // FileProtocolV1.get_info.offset
    offset_of!(uefi_raw::protocol::file_system::FileProtocolV1, set_info) as u64, // FileProtocolV1.set_info.offset
    offset_of!(uefi_raw::protocol::file_system::FileProtocolV1, flush) as u64, // FileProtocolV1.flush.offset
    size_of::<uefi_raw::protocol::file_system::FileProtocolV2>() as u64, // FileProtocolV2.size
    align_of::<uefi_raw::protocol::file_system::FileProtocolV2>() as u64, // FileProtocolV2.alignment
    offset_of!(uefi_raw::protocol::file_system::FileProtocolV2, v1) as u64, // FileProtocolV2.v1.offset
    offset_of!(uefi_raw::protocol::file_system::FileProtocolV2, open_ex) as u64, // FileProtocolV2.open_ex.offset
    offset_of!(uefi_raw::protocol::file_system::FileProtocolV2, read_ex) as u64, // FileProtocolV2.read_ex.offset
    offset_of!(uefi_raw::protocol::file_system::FileProtocolV2, write_ex) as u64, // FileProtocolV2.write_ex.offset
    offset_of!(uefi_raw::protocol::file_system::FileProtocolV2, flush_ex) as u64, // FileProtocolV2.flush_ex.offset
    size_of::<uefi_raw::protocol::file_system::FileIoToken>() as u64, // FileIoToken.size
    align_of::<uefi_raw::protocol::file_system::FileIoToken>() as u64, // FileIoToken.alignment
    offset_of!(uefi_raw::protocol::file_system::FileIoToken, event) as u64, // FileIoToken.event.offset
    offset_of!(uefi_raw::protocol::file_system::FileIoToken, status) as u64, // FileIoToken.status.offset
    offset_of!(uefi_raw::protocol::file_system::FileIoToken, buffer_size) as u64, // FileIoToken.buffer_size.offset
    offset_of!(uefi_raw::protocol::file_system::FileIoToken, buffer) as u64, // FileIoToken.buffer.offset
    size_of::<uefi_raw::protocol::file_system::FileInfo>() as u64, // FileInfo.size
    align_of::<uefi_raw::protocol::file_system::FileInfo>() as u64, // FileInfo.alignment
    offset_of!(uefi_raw::protocol::file_system::FileInfo, size) as u64, // FileInfo.size.offset
    offset_of!(uefi_raw::protocol::file_system::FileInfo, file_size) as u64, // FileInfo.file_size.offset
    offset_of!(uefi_raw::protocol::file_system::FileInfo, physical_size) as u64, // FileInfo.physical_size.offset
    offset_of!(uefi_raw::protocol::file_system::FileInfo, create_time) as u64, // FileInfo.create_time.offset
    offset_of!(uefi_raw::protocol::file_system::FileInfo, last_access_time) as u64, // FileInfo.last_access_time.offset
    offset_of!(uefi_raw::protocol::file_system::FileInfo, modification_time) as u64, // FileInfo.modification_time.offset
    offset_of!(uefi_raw::protocol::file_system::FileInfo, attribute) as u64, // FileInfo.attribute.offset
    offset_of!(uefi_raw::protocol::file_system::FileInfo, file_name) as u64, // FileInfo.file_name.offset
    size_of::<uefi_raw::protocol::file_system::FileSystemInfo>() as u64, // FileSystemInfo.size
    align_of::<uefi_raw::protocol::file_system::FileSystemInfo>() as u64, // FileSystemInfo.alignment
    offset_of!(uefi_raw::protocol::file_system::FileSystemInfo, size) as u64, // FileSystemInfo.size.offset
    offset_of!(uefi_raw::protocol::file_system::FileSystemInfo, read_only) as u64, // FileSystemInfo.read_only.offset
    offset_of!(uefi_raw::protocol::file_system::FileSystemInfo, volume_size) as u64, // FileSystemInfo.volume_size.offset
    offset_of!(uefi_raw::protocol::file_system::FileSystemInfo, free_space) as u64, // FileSystemInfo.free_space.offset
    offset_of!(uefi_raw::protocol::file_system::FileSystemInfo, block_size) as u64, // FileSystemInfo.block_size.offset
    offset_of!(uefi_raw::protocol::file_system::FileSystemInfo, volume_label) as u64, // FileSystemInfo.volume_label.offset
    size_of::<uefi_raw::protocol::file_system::FileSystemVolumeLabel>() as u64, // FileSystemVolumeLabel.size
    align_of::<uefi_raw::protocol::file_system::FileSystemVolumeLabel>() as u64, // FileSystemVolumeLabel.alignment
    offset_of!(uefi_raw::protocol::file_system::FileSystemVolumeLabel, volume_label) as u64, // FileSystemVolumeLabel.volume_label.offset
    size_of::<uefi_raw::protocol::block::BlockIoMedia>() as u64, // BlockIoMedia.size
    align_of::<uefi_raw::protocol::block::BlockIoMedia>() as u64, // BlockIoMedia.alignment
    offset_of!(uefi_raw::protocol::block::BlockIoMedia, media_id) as u64, // BlockIoMedia.media_id.offset
    offset_of!(uefi_raw::protocol::block::BlockIoMedia, removable_media) as u64, // BlockIoMedia.removable_media.offset
    offset_of!(uefi_raw::protocol::block::BlockIoMedia, media_present) as u64, // BlockIoMedia.media_present.offset
    offset_of!(uefi_raw::protocol::block::BlockIoMedia, logical_partition) as u64, // BlockIoMedia.logical_partition.offset
    offset_of!(uefi_raw::protocol::block::BlockIoMedia, read_only) as u64, // BlockIoMedia.read_only.offset
    offset_of!(uefi_raw::protocol::block::BlockIoMedia, write_caching) as u64, // BlockIoMedia.write_caching.offset
    offset_of!(uefi_raw::protocol::block::BlockIoMedia, block_size) as u64, // BlockIoMedia.block_size.offset
    offset_of!(uefi_raw::protocol::block::BlockIoMedia, io_align) as u64, // BlockIoMedia.io_align.offset
    offset_of!(uefi_raw::protocol::block::BlockIoMedia, last_block) as u64, // BlockIoMedia.last_block.offset
    offset_of!(uefi_raw::protocol::block::BlockIoMedia, lowest_aligned_lba) as u64, // BlockIoMedia.lowest_aligned_lba.offset
    offset_of!(uefi_raw::protocol::block::BlockIoMedia, logical_blocks_per_physical_block) as u64, // BlockIoMedia.logical_blocks_per_physical_block.offset
    offset_of!(uefi_raw::protocol::block::BlockIoMedia, optimal_transfer_length_granularity) as u64, // BlockIoMedia.optimal_transfer_length_granularity.offset
    size_of::<uefi_raw::protocol::block::BlockIoProtocol>() as u64, // BlockIoProtocol.size
    align_of::<uefi_raw::protocol::block::BlockIoProtocol>() as u64, // BlockIoProtocol.alignment
    offset_of!(uefi_raw::protocol::block::BlockIoProtocol, revision) as u64, // BlockIoProtocol.revision.offset
    offset_of!(uefi_raw::protocol::block::BlockIoProtocol, media) as u64, // BlockIoProtocol.media.offset
    offset_of!(uefi_raw::protocol::block::BlockIoProtocol, reset) as u64, // BlockIoProtocol.reset.offset
    offset_of!(uefi_raw::protocol::block::BlockIoProtocol, read_blocks) as u64, // BlockIoProtocol.read_blocks.offset
    offset_of!(uefi_raw::protocol::block::BlockIoProtocol, write_blocks) as u64, // BlockIoProtocol.write_blocks.offset
    offset_of!(uefi_raw::protocol::block::BlockIoProtocol, flush_blocks) as u64, // BlockIoProtocol.flush_blocks.offset
    size_of::<uefi_raw::protocol::block::BlockIo2Token>() as u64, // BlockIo2Token.size
    align_of::<uefi_raw::protocol::block::BlockIo2Token>() as u64, // BlockIo2Token.alignment
    offset_of!(uefi_raw::protocol::block::BlockIo2Token, event) as u64, // BlockIo2Token.event.offset
    offset_of!(uefi_raw::protocol::block::BlockIo2Token, transaction_status) as u64, // BlockIo2Token.transaction_status.offset
    size_of::<uefi_raw::protocol::block::BlockIo2Protocol>() as u64, // BlockIo2Protocol.size
    align_of::<uefi_raw::protocol::block::BlockIo2Protocol>() as u64, // BlockIo2Protocol.alignment
    offset_of!(uefi_raw::protocol::block::BlockIo2Protocol, media) as u64, // BlockIo2Protocol.media.offset
    offset_of!(uefi_raw::protocol::block::BlockIo2Protocol, reset) as u64, // BlockIo2Protocol.reset.offset
    offset_of!(uefi_raw::protocol::block::BlockIo2Protocol, read_blocks_ex) as u64, // BlockIo2Protocol.read_blocks_ex.offset
    offset_of!(uefi_raw::protocol::block::BlockIo2Protocol, write_blocks_ex) as u64, // BlockIo2Protocol.write_blocks_ex.offset
    offset_of!(uefi_raw::protocol::block::BlockIo2Protocol, flush_blocks_ex) as u64, // BlockIo2Protocol.flush_blocks_ex.offset
    size_of::<uefi_raw::protocol::disk::DiskIoProtocol>() as u64, // DiskIoProtocol.size
    align_of::<uefi_raw::protocol::disk::DiskIoProtocol>() as u64, // DiskIoProtocol.alignment
    offset_of!(uefi_raw::protocol::disk::DiskIoProtocol, revision) as u64, // DiskIoProtocol.revision.offset
    offset_of!(uefi_raw::protocol::disk::DiskIoProtocol, read_disk) as u64, // DiskIoProtocol.read_disk.offset
    offset_of!(uefi_raw::protocol::disk::DiskIoProtocol, write_disk) as u64, // DiskIoProtocol.write_disk.offset
    size_of::<uefi_raw::protocol::disk::DiskIo2Token>() as u64, // DiskIo2Token.size
    align_of::<uefi_raw::protocol::disk::DiskIo2Token>() as u64, // DiskIo2Token.alignment
    offset_of!(uefi_raw::protocol::disk::DiskIo2Token, event) as u64, // DiskIo2Token.event.offset
    offset_of!(uefi_raw::protocol::disk::DiskIo2Token, transaction_status) as u64, // DiskIo2Token.transaction_status.offset
    size_of::<uefi_raw::protocol::disk::DiskIo2Protocol>() as u64, // DiskIo2Protocol.size
    align_of::<uefi_raw::protocol::disk::DiskIo2Protocol>() as u64, // DiskIo2Protocol.alignment
    offset_of!(uefi_raw::protocol::disk::DiskIo2Protocol, revision) as u64, // DiskIo2Protocol.revision.offset
    offset_of!(uefi_raw::protocol::disk::DiskIo2Protocol, cancel) as u64, // DiskIo2Protocol.cancel.offset
    offset_of!(uefi_raw::protocol::disk::DiskIo2Protocol, read_disk_ex) as u64, // DiskIo2Protocol.read_disk_ex.offset
    offset_of!(uefi_raw::protocol::disk::DiskIo2Protocol, write_disk_ex) as u64, // DiskIo2Protocol.write_disk_ex.offset
    offset_of!(uefi_raw::protocol::disk::DiskIo2Protocol, flush_disk_ex) as u64, // DiskIo2Protocol.flush_disk_ex.offset
    size_of::<uefi_raw::protocol::disk::DiskInfoProtocol>() as u64, // DiskInfoProtocol.size
    align_of::<uefi_raw::protocol::disk::DiskInfoProtocol>() as u64, // DiskInfoProtocol.alignment
    offset_of!(uefi_raw::protocol::disk::DiskInfoProtocol, interface) as u64, // DiskInfoProtocol.interface.offset
    offset_of!(uefi_raw::protocol::disk::DiskInfoProtocol, inquiry) as u64, // DiskInfoProtocol.inquiry.offset
    offset_of!(uefi_raw::protocol::disk::DiskInfoProtocol, identify) as u64, // DiskInfoProtocol.identify.offset
    offset_of!(uefi_raw::protocol::disk::DiskInfoProtocol, sense_data) as u64, // DiskInfoProtocol.sense_data.offset
    offset_of!(uefi_raw::protocol::disk::DiskInfoProtocol, which_ide) as u64, // DiskInfoProtocol.which_ide.offset
    size_of::<uefi_raw::protocol::ata::AtaPassThruMode>() as u64, // AtaPassThruMode.size
    align_of::<uefi_raw::protocol::ata::AtaPassThruMode>() as u64, // AtaPassThruMode.alignment
    offset_of!(uefi_raw::protocol::ata::AtaPassThruMode, attributes) as u64, // AtaPassThruMode.attributes.offset
    offset_of!(uefi_raw::protocol::ata::AtaPassThruMode, io_align) as u64, // AtaPassThruMode.io_align.offset
    size_of::<uefi_raw::protocol::ata::AtaStatusBlock>() as u64, // AtaStatusBlock.size
    align_of::<uefi_raw::protocol::ata::AtaStatusBlock>() as u64, // AtaStatusBlock.alignment
    offset_of!(uefi_raw::protocol::ata::AtaStatusBlock, reserved1) as u64, // AtaStatusBlock.reserved1.offset
    offset_of!(uefi_raw::protocol::ata::AtaStatusBlock, status) as u64, // AtaStatusBlock.status.offset
    offset_of!(uefi_raw::protocol::ata::AtaStatusBlock, error) as u64, // AtaStatusBlock.error.offset
    offset_of!(uefi_raw::protocol::ata::AtaStatusBlock, sector_number) as u64, // AtaStatusBlock.sector_number.offset
    offset_of!(uefi_raw::protocol::ata::AtaStatusBlock, cylinder_low) as u64, // AtaStatusBlock.cylinder_low.offset
    offset_of!(uefi_raw::protocol::ata::AtaStatusBlock, cylinder_high) as u64, // AtaStatusBlock.cylinder_high.offset
    offset_of!(uefi_raw::protocol::ata::AtaStatusBlock, device_head) as u64, // AtaStatusBlock.device_head.offset
    offset_of!(uefi_raw::protocol::ata::AtaStatusBlock, sector_number_exp) as u64, // AtaStatusBlock.sector_number_exp.offset
    offset_of!(uefi_raw::protocol::ata::AtaStatusBlock, cylinder_low_exp) as u64, // AtaStatusBlock.cylinder_low_exp.offset
    offset_of!(uefi_raw::protocol::ata::AtaStatusBlock, cylinder_high_exp) as u64, // AtaStatusBlock.cylinder_high_exp.offset
    offset_of!(uefi_raw::protocol::ata::AtaStatusBlock, reserved2) as u64, // AtaStatusBlock.reserved2.offset
    offset_of!(uefi_raw::protocol::ata::AtaStatusBlock, sector_count) as u64, // AtaStatusBlock.sector_count.offset
    offset_of!(uefi_raw::protocol::ata::AtaStatusBlock, sector_count_exp) as u64, // AtaStatusBlock.sector_count_exp.offset
    offset_of!(uefi_raw::protocol::ata::AtaStatusBlock, reserved3) as u64, // AtaStatusBlock.reserved3.offset
    size_of::<uefi_raw::protocol::ata::AtaCommandBlock>() as u64, // AtaCommandBlock.size
    align_of::<uefi_raw::protocol::ata::AtaCommandBlock>() as u64, // AtaCommandBlock.alignment
    offset_of!(uefi_raw::protocol::ata::AtaCommandBlock, reserved1) as u64, // AtaCommandBlock.reserved1.offset
    offset_of!(uefi_raw::protocol::ata::AtaCommandBlock, command) as u64, // AtaCommandBlock.command.offset
    offset_of!(uefi_raw::protocol::ata::AtaCommandBlock, features) as u64, // AtaCommandBlock.features.offset
    offset_of!(uefi_raw::protocol::ata::AtaCommandBlock, sector_number) as u64, // AtaCommandBlock.sector_number.offset
    offset_of!(uefi_raw::protocol::ata::AtaCommandBlock, cylinder_low) as u64, // AtaCommandBlock.cylinder_low.offset
    offset_of!(uefi_raw::protocol::ata::AtaCommandBlock, cylinder_high) as u64, // AtaCommandBlock.cylinder_high.offset
    offset_of!(uefi_raw::protocol::ata::AtaCommandBlock, device_head) as u64, // AtaCommandBlock.device_head.offset
    offset_of!(uefi_raw::protocol::ata::AtaCommandBlock, sector_number_exp) as u64, // AtaCommandBlock.sector_number_exp.offset
    offset_of!(uefi_raw::protocol::ata::AtaCommandBlock, cylinder_low_exp) as u64, // AtaCommandBlock.cylinder_low_exp.offset
    offset_of!(uefi_raw::protocol::ata::AtaCommandBlock, cylinder_high_exp) as u64, // AtaCommandBlock.cylinder_high_exp.offset
    offset_of!(uefi_raw::protocol::ata::AtaCommandBlock, features_exp) as u64, // AtaCommandBlock.features_exp.offset
    offset_of!(uefi_raw::protocol::ata::AtaCommandBlock, sector_count) as u64, // AtaCommandBlock.sector_count.offset
    offset_of!(uefi_raw::protocol::ata::AtaCommandBlock, sector_count_exp) as u64, // AtaCommandBlock.sector_count_exp.offset
    offset_of!(uefi_raw::protocol::ata::AtaCommandBlock, reserved2) as u64, // AtaCommandBlock.reserved2.offset
    size_of::<uefi_raw::protocol::ata::AtaPassThruCommandPacket>() as u64, // AtaPassThruCommandPacket.size
    align_of::<uefi_raw::protocol::ata::AtaPassThruCommandPacket>() as u64, // AtaPassThruCommandPacket.alignment
    offset_of!(uefi_raw::protocol::ata::AtaPassThruCommandPacket, asb) as u64, // AtaPassThruCommandPacket.asb.offset
    offset_of!(uefi_raw::protocol::ata::AtaPassThruCommandPacket, acb) as u64, // AtaPassThruCommandPacket.acb.offset
    offset_of!(uefi_raw::protocol::ata::AtaPassThruCommandPacket, timeout) as u64, // AtaPassThruCommandPacket.timeout.offset
    offset_of!(uefi_raw::protocol::ata::AtaPassThruCommandPacket, in_data_buffer) as u64, // AtaPassThruCommandPacket.in_data_buffer.offset
    offset_of!(uefi_raw::protocol::ata::AtaPassThruCommandPacket, out_data_buffer) as u64, // AtaPassThruCommandPacket.out_data_buffer.offset
    offset_of!(uefi_raw::protocol::ata::AtaPassThruCommandPacket, in_transfer_length) as u64, // AtaPassThruCommandPacket.in_transfer_length.offset
    offset_of!(uefi_raw::protocol::ata::AtaPassThruCommandPacket, out_transfer_length) as u64, // AtaPassThruCommandPacket.out_transfer_length.offset
    offset_of!(uefi_raw::protocol::ata::AtaPassThruCommandPacket, protocol) as u64, // AtaPassThruCommandPacket.protocol.offset
    offset_of!(uefi_raw::protocol::ata::AtaPassThruCommandPacket, length) as u64, // AtaPassThruCommandPacket.length.offset
    size_of::<uefi_raw::protocol::ata::AtaPassThruProtocol>() as u64, // AtaPassThruProtocol.size
    align_of::<uefi_raw::protocol::ata::AtaPassThruProtocol>() as u64, // AtaPassThruProtocol.alignment
    offset_of!(uefi_raw::protocol::ata::AtaPassThruProtocol, mode) as u64, // AtaPassThruProtocol.mode.offset
    offset_of!(uefi_raw::protocol::ata::AtaPassThruProtocol, pass_thru) as u64, // AtaPassThruProtocol.pass_thru.offset
    offset_of!(uefi_raw::protocol::ata::AtaPassThruProtocol, get_next_port) as u64, // AtaPassThruProtocol.get_next_port.offset
    offset_of!(uefi_raw::protocol::ata::AtaPassThruProtocol, get_next_device) as u64, // AtaPassThruProtocol.get_next_device.offset
    offset_of!(uefi_raw::protocol::ata::AtaPassThruProtocol, build_device_path) as u64, // AtaPassThruProtocol.build_device_path.offset
    offset_of!(uefi_raw::protocol::ata::AtaPassThruProtocol, get_device) as u64, // AtaPassThruProtocol.get_device.offset
    offset_of!(uefi_raw::protocol::ata::AtaPassThruProtocol, reset_port) as u64, // AtaPassThruProtocol.reset_port.offset
    offset_of!(uefi_raw::protocol::ata::AtaPassThruProtocol, reset_device) as u64, // AtaPassThruProtocol.reset_device.offset
    size_of::<uefi_raw::protocol::scsi::ScsiIoScsiRequestPacket>() as u64, // ScsiIoScsiRequestPacket.size
    align_of::<uefi_raw::protocol::scsi::ScsiIoScsiRequestPacket>() as u64, // ScsiIoScsiRequestPacket.alignment
    offset_of!(uefi_raw::protocol::scsi::ScsiIoScsiRequestPacket, timeout) as u64, // ScsiIoScsiRequestPacket.timeout.offset
    offset_of!(uefi_raw::protocol::scsi::ScsiIoScsiRequestPacket, in_data_buffer) as u64, // ScsiIoScsiRequestPacket.in_data_buffer.offset
    offset_of!(uefi_raw::protocol::scsi::ScsiIoScsiRequestPacket, out_data_buffer) as u64, // ScsiIoScsiRequestPacket.out_data_buffer.offset
    offset_of!(uefi_raw::protocol::scsi::ScsiIoScsiRequestPacket, sense_data) as u64, // ScsiIoScsiRequestPacket.sense_data.offset
    offset_of!(uefi_raw::protocol::scsi::ScsiIoScsiRequestPacket, cdb) as u64, // ScsiIoScsiRequestPacket.cdb.offset
    offset_of!(uefi_raw::protocol::scsi::ScsiIoScsiRequestPacket, in_transfer_length) as u64, // ScsiIoScsiRequestPacket.in_transfer_length.offset
    offset_of!(uefi_raw::protocol::scsi::ScsiIoScsiRequestPacket, out_transfer_length) as u64, // ScsiIoScsiRequestPacket.out_transfer_length.offset
    offset_of!(uefi_raw::protocol::scsi::ScsiIoScsiRequestPacket, cdb_length) as u64, // ScsiIoScsiRequestPacket.cdb_length.offset
    offset_of!(uefi_raw::protocol::scsi::ScsiIoScsiRequestPacket, data_direction) as u64, // ScsiIoScsiRequestPacket.data_direction.offset
    offset_of!(uefi_raw::protocol::scsi::ScsiIoScsiRequestPacket, host_adapter_status) as u64, // ScsiIoScsiRequestPacket.host_adapter_status.offset
    offset_of!(uefi_raw::protocol::scsi::ScsiIoScsiRequestPacket, target_status) as u64, // ScsiIoScsiRequestPacket.target_status.offset
    offset_of!(uefi_raw::protocol::scsi::ScsiIoScsiRequestPacket, sense_data_length) as u64, // ScsiIoScsiRequestPacket.sense_data_length.offset
    size_of::<uefi_raw::protocol::scsi::ScsiIoProtocol>() as u64, // ScsiIoProtocol.size
    align_of::<uefi_raw::protocol::scsi::ScsiIoProtocol>() as u64, // ScsiIoProtocol.alignment
    offset_of!(uefi_raw::protocol::scsi::ScsiIoProtocol, get_device_type) as u64, // ScsiIoProtocol.get_device_type.offset
    offset_of!(uefi_raw::protocol::scsi::ScsiIoProtocol, get_device_location) as u64, // ScsiIoProtocol.get_device_location.offset
    offset_of!(uefi_raw::protocol::scsi::ScsiIoProtocol, reset_bus) as u64, // ScsiIoProtocol.reset_bus.offset
    offset_of!(uefi_raw::protocol::scsi::ScsiIoProtocol, reset_device) as u64, // ScsiIoProtocol.reset_device.offset
    offset_of!(uefi_raw::protocol::scsi::ScsiIoProtocol, execute_scsi_command) as u64, // ScsiIoProtocol.execute_scsi_command.offset
    offset_of!(uefi_raw::protocol::scsi::ScsiIoProtocol, io_align) as u64, // ScsiIoProtocol.io_align.offset
    size_of::<uefi_raw::protocol::scsi::ExtScsiPassThruMode>() as u64, // ExtScsiPassThruMode.size
    align_of::<uefi_raw::protocol::scsi::ExtScsiPassThruMode>() as u64, // ExtScsiPassThruMode.alignment
    offset_of!(uefi_raw::protocol::scsi::ExtScsiPassThruMode, adapter_id) as u64, // ExtScsiPassThruMode.adapter_id.offset
    offset_of!(uefi_raw::protocol::scsi::ExtScsiPassThruMode, attributes) as u64, // ExtScsiPassThruMode.attributes.offset
    offset_of!(uefi_raw::protocol::scsi::ExtScsiPassThruMode, io_align) as u64, // ExtScsiPassThruMode.io_align.offset
    size_of::<uefi_raw::protocol::scsi::ExtScsiPassThruProtocol>() as u64, // ExtScsiPassThruProtocol.size
    align_of::<uefi_raw::protocol::scsi::ExtScsiPassThruProtocol>() as u64, // ExtScsiPassThruProtocol.alignment
    offset_of!(uefi_raw::protocol::scsi::ExtScsiPassThruProtocol, passthru_mode) as u64, // ExtScsiPassThruProtocol.passthru_mode.offset
    offset_of!(uefi_raw::protocol::scsi::ExtScsiPassThruProtocol, pass_thru) as u64, // ExtScsiPassThruProtocol.pass_thru.offset
    offset_of!(uefi_raw::protocol::scsi::ExtScsiPassThruProtocol, get_next_target_lun) as u64, // ExtScsiPassThruProtocol.get_next_target_lun.offset
    offset_of!(uefi_raw::protocol::scsi::ExtScsiPassThruProtocol, build_device_path) as u64, // ExtScsiPassThruProtocol.build_device_path.offset
    offset_of!(uefi_raw::protocol::scsi::ExtScsiPassThruProtocol, get_target_lun) as u64, // ExtScsiPassThruProtocol.get_target_lun.offset
    offset_of!(uefi_raw::protocol::scsi::ExtScsiPassThruProtocol, reset_channel) as u64, // ExtScsiPassThruProtocol.reset_channel.offset
    offset_of!(uefi_raw::protocol::scsi::ExtScsiPassThruProtocol, reset_target_lun) as u64, // ExtScsiPassThruProtocol.reset_target_lun.offset
    offset_of!(uefi_raw::protocol::scsi::ExtScsiPassThruProtocol, get_next_target) as u64, // ExtScsiPassThruProtocol.get_next_target.offset
    size_of::<uefi_raw::protocol::nvme::NvmExpressPassThruMode>() as u64, // NvmExpressPassThruMode.size
    align_of::<uefi_raw::protocol::nvme::NvmExpressPassThruMode>() as u64, // NvmExpressPassThruMode.alignment
    offset_of!(uefi_raw::protocol::nvme::NvmExpressPassThruMode, attributes) as u64, // NvmExpressPassThruMode.attributes.offset
    offset_of!(uefi_raw::protocol::nvme::NvmExpressPassThruMode, io_align) as u64, // NvmExpressPassThruMode.io_align.offset
    offset_of!(uefi_raw::protocol::nvme::NvmExpressPassThruMode, nvme_version) as u64, // NvmExpressPassThruMode.nvme_version.offset
    size_of::<uefi_raw::protocol::nvme::NvmExpressCommand>() as u64, // NvmExpressCommand.size
    align_of::<uefi_raw::protocol::nvme::NvmExpressCommand>() as u64, // NvmExpressCommand.alignment
    offset_of!(uefi_raw::protocol::nvme::NvmExpressCommand, cdw0) as u64, // NvmExpressCommand.cdw0.offset
    offset_of!(uefi_raw::protocol::nvme::NvmExpressCommand, flags) as u64, // NvmExpressCommand.flags.offset
    offset_of!(uefi_raw::protocol::nvme::NvmExpressCommand, nsid) as u64, // NvmExpressCommand.nsid.offset
    offset_of!(uefi_raw::protocol::nvme::NvmExpressCommand, cdw2) as u64, // NvmExpressCommand.cdw2.offset
    offset_of!(uefi_raw::protocol::nvme::NvmExpressCommand, cdw3) as u64, // NvmExpressCommand.cdw3.offset
    offset_of!(uefi_raw::protocol::nvme::NvmExpressCommand, cdw10) as u64, // NvmExpressCommand.cdw10.offset
    offset_of!(uefi_raw::protocol::nvme::NvmExpressCommand, cdw11) as u64, // NvmExpressCommand.cdw11.offset
    offset_of!(uefi_raw::protocol::nvme::NvmExpressCommand, cdw12) as u64, // NvmExpressCommand.cdw12.offset
    offset_of!(uefi_raw::protocol::nvme::NvmExpressCommand, cdw13) as u64, // NvmExpressCommand.cdw13.offset
    offset_of!(uefi_raw::protocol::nvme::NvmExpressCommand, cdw14) as u64, // NvmExpressCommand.cdw14.offset
    offset_of!(uefi_raw::protocol::nvme::NvmExpressCommand, cdw15) as u64, // NvmExpressCommand.cdw15.offset
    size_of::<uefi_raw::protocol::nvme::NvmExpressCompletion>() as u64, // NvmExpressCompletion.size
    align_of::<uefi_raw::protocol::nvme::NvmExpressCompletion>() as u64, // NvmExpressCompletion.alignment
    offset_of!(uefi_raw::protocol::nvme::NvmExpressCompletion, dw0) as u64, // NvmExpressCompletion.dw0.offset
    offset_of!(uefi_raw::protocol::nvme::NvmExpressCompletion, dw1) as u64, // NvmExpressCompletion.dw1.offset
    offset_of!(uefi_raw::protocol::nvme::NvmExpressCompletion, dw2) as u64, // NvmExpressCompletion.dw2.offset
    offset_of!(uefi_raw::protocol::nvme::NvmExpressCompletion, dw3) as u64, // NvmExpressCompletion.dw3.offset
    size_of::<uefi_raw::protocol::nvme::NvmExpressPassThruCommandPacket>() as u64, // NvmExpressPassThruCommandPacket.size
    align_of::<uefi_raw::protocol::nvme::NvmExpressPassThruCommandPacket>() as u64, // NvmExpressPassThruCommandPacket.alignment
    offset_of!(uefi_raw::protocol::nvme::NvmExpressPassThruCommandPacket, command_timeout) as u64, // NvmExpressPassThruCommandPacket.command_timeout.offset
    offset_of!(uefi_raw::protocol::nvme::NvmExpressPassThruCommandPacket, transfer_buffer) as u64, // NvmExpressPassThruCommandPacket.transfer_buffer.offset
    offset_of!(uefi_raw::protocol::nvme::NvmExpressPassThruCommandPacket, transfer_length) as u64, // NvmExpressPassThruCommandPacket.transfer_length.offset
    offset_of!(uefi_raw::protocol::nvme::NvmExpressPassThruCommandPacket, meta_data_buffer) as u64, // NvmExpressPassThruCommandPacket.meta_data_buffer.offset
    offset_of!(uefi_raw::protocol::nvme::NvmExpressPassThruCommandPacket, meta_data_length) as u64, // NvmExpressPassThruCommandPacket.meta_data_length.offset
    offset_of!(uefi_raw::protocol::nvme::NvmExpressPassThruCommandPacket, queue_type) as u64, // NvmExpressPassThruCommandPacket.queue_type.offset
    offset_of!(uefi_raw::protocol::nvme::NvmExpressPassThruCommandPacket, nvme_cmd) as u64, // NvmExpressPassThruCommandPacket.nvme_cmd.offset
    offset_of!(uefi_raw::protocol::nvme::NvmExpressPassThruCommandPacket, nvme_completion) as u64, // NvmExpressPassThruCommandPacket.nvme_completion.offset
    size_of::<uefi_raw::protocol::nvme::NvmExpressPassThruProtocol>() as u64, // NvmExpressPassThruProtocol.size
    align_of::<uefi_raw::protocol::nvme::NvmExpressPassThruProtocol>() as u64, // NvmExpressPassThruProtocol.alignment
    offset_of!(uefi_raw::protocol::nvme::NvmExpressPassThruProtocol, mode) as u64, // NvmExpressPassThruProtocol.mode.offset
    offset_of!(uefi_raw::protocol::nvme::NvmExpressPassThruProtocol, pass_thru) as u64, // NvmExpressPassThruProtocol.pass_thru.offset
    offset_of!(uefi_raw::protocol::nvme::NvmExpressPassThruProtocol, get_next_namespace) as u64, // NvmExpressPassThruProtocol.get_next_namespace.offset
    offset_of!(uefi_raw::protocol::nvme::NvmExpressPassThruProtocol, build_device_path) as u64, // NvmExpressPassThruProtocol.build_device_path.offset
    offset_of!(uefi_raw::protocol::nvme::NvmExpressPassThruProtocol, get_namespace) as u64, // NvmExpressPassThruProtocol.get_namespace.offset
    size_of::<uefi_raw::protocol::firmware_volume::FvWriteFileData>() as u64, // FvWriteFileData.size
    align_of::<uefi_raw::protocol::firmware_volume::FvWriteFileData>() as u64, // FvWriteFileData.alignment
    offset_of!(uefi_raw::protocol::firmware_volume::FvWriteFileData, name_guid) as u64, // FvWriteFileData.name_guid.offset
    offset_of!(uefi_raw::protocol::firmware_volume::FvWriteFileData, r#type) as u64, // FvWriteFileData.file_type.offset
    offset_of!(uefi_raw::protocol::firmware_volume::FvWriteFileData, file_attributes) as u64, // FvWriteFileData.file_attributes.offset
    offset_of!(uefi_raw::protocol::firmware_volume::FvWriteFileData, buffer) as u64, // FvWriteFileData.buffer.offset
    offset_of!(uefi_raw::protocol::firmware_volume::FvWriteFileData, buffer_size) as u64, // FvWriteFileData.buffer_size.offset
    size_of::<uefi_raw::protocol::firmware_volume::FirmwareVolume2Protocol>() as u64, // FirmwareVolume2Protocol.size
    align_of::<uefi_raw::protocol::firmware_volume::FirmwareVolume2Protocol>() as u64, // FirmwareVolume2Protocol.alignment
    offset_of!(uefi_raw::protocol::firmware_volume::FirmwareVolume2Protocol, get_volume_attributes) as u64, // FirmwareVolume2Protocol.get_volume_attributes.offset
    offset_of!(uefi_raw::protocol::firmware_volume::FirmwareVolume2Protocol, set_volume_attributes) as u64, // FirmwareVolume2Protocol.set_volume_attributes.offset
    offset_of!(uefi_raw::protocol::firmware_volume::FirmwareVolume2Protocol, read_file) as u64, // FirmwareVolume2Protocol.read_file.offset
    offset_of!(uefi_raw::protocol::firmware_volume::FirmwareVolume2Protocol, read_section) as u64, // FirmwareVolume2Protocol.read_section.offset
    offset_of!(uefi_raw::protocol::firmware_volume::FirmwareVolume2Protocol, write_file) as u64, // FirmwareVolume2Protocol.write_file.offset
    offset_of!(uefi_raw::protocol::firmware_volume::FirmwareVolume2Protocol, get_next_file) as u64, // FirmwareVolume2Protocol.get_next_file.offset
    offset_of!(uefi_raw::protocol::firmware_volume::FirmwareVolume2Protocol, key_size) as u64, // FirmwareVolume2Protocol.key_size.offset
    offset_of!(uefi_raw::protocol::firmware_volume::FirmwareVolume2Protocol, parent_handle) as u64, // FirmwareVolume2Protocol.parent_handle.offset
    offset_of!(uefi_raw::protocol::firmware_volume::FirmwareVolume2Protocol, get_info) as u64, // FirmwareVolume2Protocol.get_info.offset
    offset_of!(uefi_raw::protocol::firmware_volume::FirmwareVolume2Protocol, set_info) as u64, // FirmwareVolume2Protocol.set_info.offset
    size_of::<uefi_raw::protocol::firmware_volume::FirmwareVolumeBlock2Protocol>() as u64, // FirmwareVolumeBlock2Protocol.size
    align_of::<uefi_raw::protocol::firmware_volume::FirmwareVolumeBlock2Protocol>() as u64, // FirmwareVolumeBlock2Protocol.alignment
    offset_of!(uefi_raw::protocol::firmware_volume::FirmwareVolumeBlock2Protocol, get_attributes) as u64, // FirmwareVolumeBlock2Protocol.get_attributes.offset
    offset_of!(uefi_raw::protocol::firmware_volume::FirmwareVolumeBlock2Protocol, set_attributes) as u64, // FirmwareVolumeBlock2Protocol.set_attributes.offset
    offset_of!(uefi_raw::protocol::firmware_volume::FirmwareVolumeBlock2Protocol, get_physical_address) as u64, // FirmwareVolumeBlock2Protocol.get_physical_address.offset
    offset_of!(uefi_raw::protocol::firmware_volume::FirmwareVolumeBlock2Protocol, get_block_size) as u64, // FirmwareVolumeBlock2Protocol.get_block_size.offset
    offset_of!(uefi_raw::protocol::firmware_volume::FirmwareVolumeBlock2Protocol, read) as u64, // FirmwareVolumeBlock2Protocol.read.offset
    offset_of!(uefi_raw::protocol::firmware_volume::FirmwareVolumeBlock2Protocol, write) as u64, // FirmwareVolumeBlock2Protocol.write.offset
    offset_of!(uefi_raw::protocol::firmware_volume::FirmwareVolumeBlock2Protocol, erase_blocks) as u64, // FirmwareVolumeBlock2Protocol.erase_blocks.offset
    offset_of!(uefi_raw::protocol::firmware_volume::FirmwareVolumeBlock2Protocol, parent_handle) as u64, // FirmwareVolumeBlock2Protocol.parent_handle.offset
    size_of::<uefi_raw::protocol::firmware_management::FirmwareManagementCapsuleHeader>() as u64, // FirmwareManagementCapsuleHeader.size
    align_of::<uefi_raw::protocol::firmware_management::FirmwareManagementCapsuleHeader>() as u64, // FirmwareManagementCapsuleHeader.alignment
    offset_of!(uefi_raw::protocol::firmware_management::FirmwareManagementCapsuleHeader, version) as u64, // FirmwareManagementCapsuleHeader.version.offset
    offset_of!(uefi_raw::protocol::firmware_management::FirmwareManagementCapsuleHeader, embedded_driver_count) as u64, // FirmwareManagementCapsuleHeader.embedded_driver_count.offset
    offset_of!(uefi_raw::protocol::firmware_management::FirmwareManagementCapsuleHeader, payload_item_count) as u64, // FirmwareManagementCapsuleHeader.payload_item_count.offset
    offset_of!(uefi_raw::protocol::firmware_management::FirmwareManagementCapsuleHeader, item_offset_list) as u64, // FirmwareManagementCapsuleHeader.item_offset_list.offset
    size_of::<uefi_raw::protocol::firmware_management::FirmwareManagementCapsuleImageHeader>() as u64, // FirmwareManagementCapsuleImageHeader.size
    align_of::<uefi_raw::protocol::firmware_management::FirmwareManagementCapsuleImageHeader>() as u64, // FirmwareManagementCapsuleImageHeader.alignment
    offset_of!(uefi_raw::protocol::firmware_management::FirmwareManagementCapsuleImageHeader, version) as u64, // FirmwareManagementCapsuleImageHeader.version.offset
    offset_of!(uefi_raw::protocol::firmware_management::FirmwareManagementCapsuleImageHeader, update_image_type_id) as u64, // FirmwareManagementCapsuleImageHeader.update_image_type_id.offset
    offset_of!(uefi_raw::protocol::firmware_management::FirmwareManagementCapsuleImageHeader, update_image_index) as u64, // FirmwareManagementCapsuleImageHeader.update_image_index.offset
    offset_of!(uefi_raw::protocol::firmware_management::FirmwareManagementCapsuleImageHeader, reserved_bytes) as u64, // FirmwareManagementCapsuleImageHeader.reserved_bytes.offset
    offset_of!(uefi_raw::protocol::firmware_management::FirmwareManagementCapsuleImageHeader, update_image_size) as u64, // FirmwareManagementCapsuleImageHeader.update_image_size.offset
    offset_of!(uefi_raw::protocol::firmware_management::FirmwareManagementCapsuleImageHeader, update_vendor_code_size) as u64, // FirmwareManagementCapsuleImageHeader.update_vendor_code_size.offset
    offset_of!(uefi_raw::protocol::firmware_management::FirmwareManagementCapsuleImageHeader, update_hardware_instance) as u64, // FirmwareManagementCapsuleImageHeader.update_hardware_instance.offset
    offset_of!(uefi_raw::protocol::firmware_management::FirmwareManagementCapsuleImageHeader, image_capsule_support) as u64, // FirmwareManagementCapsuleImageHeader.image_capsule_support.offset
    size_of::<uefi_raw::protocol::firmware_management::FirmwareImageDep>() as u64, // FirmwareImageDep.size
    align_of::<uefi_raw::protocol::firmware_management::FirmwareImageDep>() as u64, // FirmwareImageDep.alignment
    offset_of!(uefi_raw::protocol::firmware_management::FirmwareImageDep, dependencies) as u64, // FirmwareImageDep.dependencies.offset
    size_of::<uefi_raw::protocol::firmware_management::FirmwareImageDescriptor>() as u64, // FirmwareImageDescriptor.size
    align_of::<uefi_raw::protocol::firmware_management::FirmwareImageDescriptor>() as u64, // FirmwareImageDescriptor.alignment
    offset_of!(uefi_raw::protocol::firmware_management::FirmwareImageDescriptor, image_index) as u64, // FirmwareImageDescriptor.image_index.offset
    offset_of!(uefi_raw::protocol::firmware_management::FirmwareImageDescriptor, image_type_id) as u64, // FirmwareImageDescriptor.image_type_id.offset
    offset_of!(uefi_raw::protocol::firmware_management::FirmwareImageDescriptor, image_id) as u64, // FirmwareImageDescriptor.image_id.offset
    offset_of!(uefi_raw::protocol::firmware_management::FirmwareImageDescriptor, image_id_name) as u64, // FirmwareImageDescriptor.image_id_name.offset
    offset_of!(uefi_raw::protocol::firmware_management::FirmwareImageDescriptor, version) as u64, // FirmwareImageDescriptor.version.offset
    offset_of!(uefi_raw::protocol::firmware_management::FirmwareImageDescriptor, version_name) as u64, // FirmwareImageDescriptor.version_name.offset
    offset_of!(uefi_raw::protocol::firmware_management::FirmwareImageDescriptor, size) as u64, // FirmwareImageDescriptor.size.offset
    offset_of!(uefi_raw::protocol::firmware_management::FirmwareImageDescriptor, attributes_supported) as u64, // FirmwareImageDescriptor.attributes_supported.offset
    offset_of!(uefi_raw::protocol::firmware_management::FirmwareImageDescriptor, attributes_setting) as u64, // FirmwareImageDescriptor.attributes_setting.offset
    offset_of!(uefi_raw::protocol::firmware_management::FirmwareImageDescriptor, compatibilities) as u64, // FirmwareImageDescriptor.compatibilities.offset
    offset_of!(uefi_raw::protocol::firmware_management::FirmwareImageDescriptor, lowest_supported_image_version) as u64, // FirmwareImageDescriptor.lowest_supported_image_version.offset
    offset_of!(uefi_raw::protocol::firmware_management::FirmwareImageDescriptor, last_attempt_version) as u64, // FirmwareImageDescriptor.last_attempt_version.offset
    offset_of!(uefi_raw::protocol::firmware_management::FirmwareImageDescriptor, last_attempt_status) as u64, // FirmwareImageDescriptor.last_attempt_status.offset
    offset_of!(uefi_raw::protocol::firmware_management::FirmwareImageDescriptor, hardware_instance) as u64, // FirmwareImageDescriptor.hardware_instance.offset
    offset_of!(uefi_raw::protocol::firmware_management::FirmwareImageDescriptor, dependencies) as u64, // FirmwareImageDescriptor.dependencies.offset
    size_of::<uefi_raw::protocol::firmware_management::FirmwareManagementProtocol>() as u64, // FirmwareManagementProtocol.size
    align_of::<uefi_raw::protocol::firmware_management::FirmwareManagementProtocol>() as u64, // FirmwareManagementProtocol.alignment
    offset_of!(uefi_raw::protocol::firmware_management::FirmwareManagementProtocol, get_image_info) as u64, // FirmwareManagementProtocol.get_image_info.offset
    offset_of!(uefi_raw::protocol::firmware_management::FirmwareManagementProtocol, get_image) as u64, // FirmwareManagementProtocol.get_image.offset
    offset_of!(uefi_raw::protocol::firmware_management::FirmwareManagementProtocol, set_image) as u64, // FirmwareManagementProtocol.set_image.offset
    offset_of!(uefi_raw::protocol::firmware_management::FirmwareManagementProtocol, check_image) as u64, // FirmwareManagementProtocol.check_image.offset
    offset_of!(uefi_raw::protocol::firmware_management::FirmwareManagementProtocol, get_package_info) as u64, // FirmwareManagementProtocol.get_package_info.offset
    offset_of!(uefi_raw::protocol::firmware_management::FirmwareManagementProtocol, set_package_info) as u64, // FirmwareManagementProtocol.set_package_info.offset
    size_of::<uefi_raw::firmware_storage::FirmwareVolumeHeader>() as u64, // FirmwareVolumeHeader.size
    align_of::<uefi_raw::firmware_storage::FirmwareVolumeHeader>() as u64, // FirmwareVolumeHeader.alignment
    offset_of!(uefi_raw::firmware_storage::FirmwareVolumeHeader, zero_vector) as u64, // FirmwareVolumeHeader.zero_vector.offset
    offset_of!(uefi_raw::firmware_storage::FirmwareVolumeHeader, file_system_guid) as u64, // FirmwareVolumeHeader.file_system_guid.offset
    offset_of!(uefi_raw::firmware_storage::FirmwareVolumeHeader, fv_length) as u64, // FirmwareVolumeHeader.fv_length.offset
    offset_of!(uefi_raw::firmware_storage::FirmwareVolumeHeader, signature) as u64, // FirmwareVolumeHeader.signature.offset
    offset_of!(uefi_raw::firmware_storage::FirmwareVolumeHeader, attributes) as u64, // FirmwareVolumeHeader.attributes.offset
    offset_of!(uefi_raw::firmware_storage::FirmwareVolumeHeader, header_length) as u64, // FirmwareVolumeHeader.header_length.offset
    offset_of!(uefi_raw::firmware_storage::FirmwareVolumeHeader, checksum) as u64, // FirmwareVolumeHeader.checksum.offset
    offset_of!(uefi_raw::firmware_storage::FirmwareVolumeHeader, ext_header_offset) as u64, // FirmwareVolumeHeader.ext_header_offset.offset
    offset_of!(uefi_raw::firmware_storage::FirmwareVolumeHeader, reserved) as u64, // FirmwareVolumeHeader.reserved.offset
    offset_of!(uefi_raw::firmware_storage::FirmwareVolumeHeader, revision) as u64, // FirmwareVolumeHeader.revision.offset
    offset_of!(uefi_raw::firmware_storage::FirmwareVolumeHeader, block_map) as u64, // FirmwareVolumeHeader.block_map.offset
    size_of::<uefi_raw::firmware_storage::FirmwareVolumeBlockMap>() as u64, // FirmwareVolumeBlockMap.size
    align_of::<uefi_raw::firmware_storage::FirmwareVolumeBlockMap>() as u64, // FirmwareVolumeBlockMap.alignment
    offset_of!(uefi_raw::firmware_storage::FirmwareVolumeBlockMap, num_blocks) as u64, // FirmwareVolumeBlockMap.num_blocks.offset
    offset_of!(uefi_raw::firmware_storage::FirmwareVolumeBlockMap, length) as u64, // FirmwareVolumeBlockMap.length.offset
];
