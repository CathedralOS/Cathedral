include!(env!("OMEGA_MATRIX_CONSTANTS"));
std::thread_local! { static CODE: code::Code = code::Code::new(BYTES); }
pub(crate) fn decide(packet: &mut [u64; 128]) -> u64 {
    CODE.with(|code| {
        // SAFETY: Packaging checked the export, the mapping is RX and remains
        // live for this call, and its sole parameter is this exclusive packet.
        let decide: unsafe extern "sysv64" fn(*mut u64) -> u64 =
            unsafe { core::mem::transmute(code.address.add(ENTRY)) };
        // SAFETY: Same export contract; no references survive this call.
        unsafe { decide(packet.as_mut_ptr()) }
    })
}
