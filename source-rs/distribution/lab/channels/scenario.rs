//! Prove local names, checked move/copyout, worker death, quotas and slot reuse.
use super::*;
pub fn run() -> u64 {
    resources::run();
    delivery::probes::creation();
    delivery::quota();
    notifications::run();
    shared::run();
    sponsorship::run();
    for _ in 0..32 {
        delivery::exchange();
    }
    for _ in 0..16 {
        delivery::abandoned();
    }
    delivery::closed_wait();
    startup::run();
    tree::run();
    let worker = cathedral_user_runtime::task::Launch::bootstrap()
        .unwrap()
        .spawn(9)
        .unwrap();
    assert_eq!(
        worker.wait().unwrap(),
        cathedral_user_runtime::task::Outcome::Returned(0)
    );
    delivery::quota();
    cathedral_user_runtime::write(
        b"Cathedral IPC: channels, startup bundles, recursive admission and reclamation passed\n",
    )
    .unwrap();
    let (_, idle) = ipc::channel().unwrap();
    let _ = idle.receive(&mut [0; 64]);
    0
}
