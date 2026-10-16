#![no_std]
#![no_main]
//! Supervise a heap-backed worker through leak, fault and normal exit. Withdrawing
//! its page reservation after each death proves no retained backing charge remains.
use cathedral_user_runtime::{
    resources,
    task::{Launch, Outcome},
    write, yield_now,
};
cathedral_user_runtime::entry!(main);
fn main(_: u64, _: u64) -> u64 {
    let launch = Launch::bootstrap().unwrap();
    for mode in [1, 2, 0] {
        let child = launch.spawn(mode).unwrap();
        let result = child.wait().unwrap();
        if mode == 2 {
            assert!(matches!(result, Outcome::Fault { vector: 14, .. }));
        } else {
            assert_eq!(result, Outcome::Returned(if mode == 1 { 9 } else { 0 }));
        }
        let before = resources::budget().unwrap();
        let limits = resources::Limits {
            private_pages: 0,
            reader_pages: 0,
            channels: 4,
        };
        resources::assign(launch, limits).unwrap();
        assert_eq!(
            resources::budget().unwrap().limits.private_pages,
            before.limits.private_pages + 64
        );
        resources::assign(
            launch,
            resources::Limits {
                private_pages: 64,
                ..limits
            },
        )
        .unwrap();
        assert_eq!(resources::budget().unwrap(), before);
        write(b"Cathedral database: worker budget reclaimed\n").unwrap();
    }
    write(b"Cathedral database: heap reclamation and durable transaction passed\n").unwrap();
    loop {
        yield_now();
    }
}
