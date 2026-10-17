//! Session-bounded recovery. A healthy reply never replenishes restart authority.
//! A probe outlasts the matrix's storage deadline, so a stalled provider is
//! retired by the matrix's verdict before the owner replaces it.
pub const PROBE_INTERVAL: u64 = 500;
pub const PROBE_TIMEOUT: u64 = cathedral_platform_contracts::matrix::STORAGE_DEADLINE_TICKS + 100;
pub const MAX_RESTARTS: u8 = 3;
#[derive(Default)]
pub struct Watchdog {
    pub pending: Option<u64>,
    pub next_probe: u64,
    pub offline: bool,
    restarts: u8,
}
impl Watchdog {
    pub fn allow_restart(&mut self) -> bool {
        self.pending = None;
        if self.offline || self.restarts == MAX_RESTARTS {
            self.offline = true;
            false
        } else {
            self.restarts += 1;
            true
        }
    }
    pub fn healthy(&mut self, now: u64) {
        self.pending = None;
        self.next_probe = now.wrapping_add(PROBE_INTERVAL);
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn healthy_intervals_do_not_enable_a_restart_loop() {
        let mut watch = Watchdog::default();
        for _ in 0..MAX_RESTARTS {
            assert!(watch.allow_restart());
            watch.healthy(1000);
        }
        assert!(!watch.allow_restart());
        watch.healthy(2000);
        assert!(!watch.allow_restart());
        assert!(watch.offline);
    }
}
