// Rust plumbing; isa.rs selects the compiled instruction-set implementation.
#[cfg(target_arch = "aarch64")]
pub(crate) mod aarch64;
#[cfg(target_arch = "x86_64")]
pub(crate) mod x86;
#[cfg(target_arch = "x86_64")]
pub(crate) mod x86_64;
include!("isa.rs");
