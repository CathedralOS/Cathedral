// Select CPU mechanisms independently of board controllers. The crate entrance
// lists the shared interface; descend into the selected ISA's named entrance.
#[cfg(target_arch = "aarch64")]
pub(crate) use aarch64 as selected;
#[cfg(target_arch = "x86_64")]
pub(crate) use x86_64 as selected;
#[cfg(not(any(target_arch = "x86_64", target_arch = "aarch64")))]
compile_error!("Only x86-64 and AArch64 backends are implemented");
