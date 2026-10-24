core::arch::global_asm!(include_str!(env!("OMEGA_CHOOSER_APP_ASSEMBLY")));
pub fn step() -> cathedral_omega_app::Step {
    unsafe extern "sysv64" {
        fn omega_app_step(state: *mut u64, packet: *mut u64) -> u64;
    }
    omega_app_step
}
