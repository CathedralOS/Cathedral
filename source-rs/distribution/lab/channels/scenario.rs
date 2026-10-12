//! Prove local names, checked move/copyout, worker death, quotas and slot reuse.
use super::*;
pub fn run() -> u64 {
    resources::run();
    delivery::probes::creation();
    delivery::quota();
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
    delivery::quota();
    cathedral_user_runtime::write(
        b"Cathedral IPC: channels, startup bundles, recursive admission and reclamation passed\n",
    )
    .unwrap();
    let (_, idle) = ipc::channel().unwrap();
    let _ = idle.receive(&mut [0; 64]);
    0
}
