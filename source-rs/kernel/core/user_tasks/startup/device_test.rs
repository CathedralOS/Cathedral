//! Duplicate or unavailable hardware must fail before any process admission or I/O.
use super::composition::validate;
use crate::user_tasks::{Config, Error, Executable, Program, Supervision};
use cathedral_arch::board::DeviceId;

#[test]
fn device_custody_is_exclusive_and_board_specific() {
    let mut launches = [0, 1].map(|_| Supervision {
        device: Some(DeviceId::AuxiliarySerial),
        disk: false,
        keyboard: false,
        framebuffer: None,
        owner: 0,
        peer: 0,
        program: Program {
            executable: Executable::Probe(&[]),
            arguments: [0, 0],
        },
        frame_limit: usize::MAX,
    });
    fn config<'a>(supervision: &'a [Supervision<'a>]) -> Config<'a> {
        Config {
            executables: &[],
            dynamic_tasks: 0,
            channel_limits: &[],
            cpu_limits: &[],
            memory: &[],
            frame_limit: usize::MAX,
            links: &[],
            endpoints: &[],
            supervision,
            clock_readers: &[],
        }
    }
    assert!(matches!(
        validate(1, &config(&launches)),
        Err(Error::InvalidEndpoints)
    ));
    launches[1].device = None;
    assert_eq!(
        validate(1, &config(&launches)).is_ok(),
        DeviceId::AuxiliarySerial.available()
    );
    launches[0].device = None;
    assert_eq!(validate(1, &config(&launches)).unwrap(), 3);
}
