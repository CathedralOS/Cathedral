use uefi_raw::protocol::console::GraphicsOutputProtocolMode;
    use uefi_raw::protocol::device_path::{DevicePathProtocol, DeviceType, DeviceSubType};
    #[test]
    fn every_length_encoding() {
        // Exhaustive upstream operation check; not execution of Omega helper.
        for n in 0..=u16::MAX {
            let header = DevicePathProtocol { major_type: DeviceType::END,
                sub_type: DeviceSubType::END_ENTIRE, length: n.to_le_bytes() };
            assert_eq!(header.length(), n);
        }
    }
    #[test]
    fn graphics_mode_default() {
        let mode=GraphicsOutputProtocolMode::default();
        assert_eq!(mode.max_mode,0); assert_eq!(mode.mode,0);
        assert!(mode.info.is_null()); assert_eq!(mode.size_of_info,0);
        assert_eq!(mode.frame_buffer_base,0); assert_eq!(mode.frame_buffer_size,0);
    }
