#![no_std]
#![no_main]
//! The initial supervisor has no disk grant; its child exercises the platform
//! block region. tools/boot-harness-rs/volume_test.py owns fixtures and cold boots.
mod probe;
use cathedral_user_runtime::{
    task::{Launch, Outcome},
    write, yield_now,
};
cathedral_user_runtime::entry!(main);
fn main(role: u64, _: u64) -> u64 {
    if role == 1 {
        return probe::run();
    }
    assert!(cathedral_user_runtime::disk::read(8).is_err());
    let child = Launch::bootstrap().unwrap().spawn(0).unwrap();
    match child.wait().unwrap() {
        Outcome::Returned(0) => write(b"Cathedral volume: flushed\n").unwrap(),
        Outcome::Returned(1) => write(b"Cathedral volume: reopened\n").unwrap(),
        Outcome::Returned(2) => write(b"Cathedral volume: refused\n").unwrap(),
        other => panic!("volume child failed: {other:?}"),
    }
    loop {
        yield_now();
    }
}
