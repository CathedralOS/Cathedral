use crate::{PACKET_WORDS, STATE_WORDS};
/// ABI 1: opaque state and packet pointers, x86-64 System V. Packet word 15
/// selects step (0), state size (1), alignment (2), or ABI version (3).
pub type Step = unsafe extern "sysv64" fn(*mut u64, *mut u64) -> u64;
pub struct Instance<'a> {
    step: Step,
    state: &'a mut [u64; STATE_WORDS],
    pub packet: &'a mut [u64; PACKET_WORDS],
}
impl<'a> Instance<'a> {
    /// # Safety
    /// `step` must be a validated ABI 1 export, executable for this instance's
    /// lifetime. Queries must not touch state; steps may use only the reported
    /// state layout and packet. The export must not retain borrowed pointers.
    pub unsafe fn new(
        step: Step,
        state: &'a mut [u64; STATE_WORDS],
        packet: &'a mut [u64; PACKET_WORDS],
    ) -> Option<Self> {
        state.fill(0);
        packet.fill(0);
        let mut query = |kind| {
            packet[15] = kind;
            // SAFETY: Caller supplies the query contract; both buffers are live.
            unsafe { step(state.as_mut_ptr(), packet.as_mut_ptr()) }
        };
        let (bytes, align, version) = (query(1), query(2), query(3));
        packet[15] = 0;
        if bytes == 0
            || bytes > (STATE_WORDS * 8) as u64
            || align == 0
            || align > 8
            || !align.is_power_of_two()
            || version != 1
        {
            return None;
        }
        Some(Self {
            step,
            state,
            packet,
        })
    }
    pub fn advance(&mut self) -> u64 {
        self.packet[15] = 0;
        // SAFETY: Constructor checked the buffer bounds against the export's
        // layout. Exclusive borrows persist; the code lifetime is its contract.
        unsafe { (self.step)(self.state.as_mut_ptr(), self.packet.as_mut_ptr()) }
    }
}
