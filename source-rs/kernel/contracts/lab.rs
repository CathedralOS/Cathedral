//! What the kernel's lab sessions and the lab programs they run agree on.

/// The echo exchange: ECHO_BURSTS bursts of ECHO_ROUNDS round trips between the
/// client and the server. The client times every burst and reports the fastest
/// beside the whole exchange, so one figure stays clear of timer ticks under
/// emulation and of host preemption under a hypervisor while the other shows
/// what landed in the rest.
pub const ECHO_ROUNDS: usize = 32;
pub const ECHO_BURSTS: usize = 8;
