//! Real timer contention: one outer peer versus an owner with a growing subtree.
use cathedral_user_runtime::{
    ipc::Handle,
    task::{Launch, Outcome, Port},
    time,
};

pub fn owner(descendants: u64) -> u64 {
    let loans = [Launch::at(1).unwrap(), Launch::at(2).unwrap()];
    let child = Launch::at(0)
        .unwrap()
        .spawn_tree(descendants, &[], &loans[..descendants as usize])
        .unwrap();
    let (_, ready) = Port::at(0).unwrap().connect().unwrap();
    ready.receive(&mut [0; 64]).unwrap();
    Handle::bootstrap(0).unwrap().send(b"measure").unwrap();
    Handle::bootstrap(1).unwrap().receive(&mut [0; 64]).unwrap();
    child.cancel().unwrap();
    assert_eq!(child.wait().unwrap(), Outcome::Cancelled);
    // Loans return only after the entire spinning subtree is reclaimed.
    for (index, old) in loans.iter().enumerate() {
        let returned = Launch::at(index as u64 + 1).unwrap();
        assert_eq!(returned.raw() != old.raw(), descendants != 0);
    }
    0
}
pub fn peer() -> u64 {
    Handle::bootstrap(0).unwrap().receive(&mut [0; 64]).unwrap();
    let deadline = time::after(128).unwrap();
    while !time::reached(time::now().unwrap(), deadline) {
        core::hint::spin_loop(); // Clock calls must retain the current quantum.
    }
    Handle::bootstrap(1).unwrap().send(b"done").unwrap();
    0
}
pub fn branch(descendants: u64) -> u64 {
    for index in 0..descendants {
        Launch::at(index).unwrap().spawn(0).unwrap();
    }
    Handle::bootstrap(1).unwrap().send(b"ready").unwrap();
    spin()
}
pub fn spin() -> ! {
    // SAFETY: Intentional userspace spin with no syscalls or cooperative yields.
    // The owner ends it using timer preemption followed by subtree cancellation.
    unsafe { core::arch::asm!("2:", "pause", "jmp 2b", options(noreturn)) }
}
