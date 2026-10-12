// Select the actual ISA's register-preservation experiment; both busy-loop under preemption.
#[cfg(target_arch = "x86_64")]
pub const STATE: &str = "GPR SSE x87 MXCSR";
#[cfg(target_arch = "aarch64")]
pub const STATE: &str = "GPR FP/SIMD FPCR FPSR";
