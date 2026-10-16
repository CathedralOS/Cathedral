//! Optional pointer profile: authorize, probe boundaries, then supervise interactions.
use super::*;
use cathedral_platform_contracts::display::pointer as p;
use cathedral_user_runtime::Error;
impl Session {
    pub(super) fn pointers(&mut self, source_launch: Launch) -> u64 {
        place(self.display, 1, 240, 160); // Deliberate overlap; app two is on top.
        assert_eq!(call(self.display, p::ENABLE, 1)[0], 0);
        let lease = self.grant_pointer();
        self.pointer_boundaries(lease);
        let lease = self.grant_pointer();
        assert_eq!(call(self.a, 12, 0)[0], 0);
        assert_eq!(call(self.b, 12, lease)[0], 0);
        let mut source = source_launch.spawn(0).unwrap();
        source_ready();
        marker(b"Cathedral pointer: ready\n");
        loop {
            // Control and input stay independent. A spinning/flooding app cannot
            // indefinitely retain its distribution slot or its compositor capture.
            for index in 0..2 {
                let (pair, child) = if index == 0 {
                    (self.a, self.left)
                } else {
                    (self.b, self.right)
                };
                if healthy(pair).is_err() {
                    child.cancel().unwrap();
                    let _ = child.wait().unwrap();
                    gone(self.display, index);
                    let replacement = Launch::at(index + 1).unwrap().spawn(0).unwrap();
                    let pair = connect(index + 1);
                    assert_eq!(call(pair, 1, 1)[0], 0);
                    let lease = if index == 1 { self.grant_pointer() } else { 0 };
                    assert_eq!(call(pair, 12, lease)[0], 0);
                    if index == 0 {
                        self.left = replacement;
                        self.a = pair;
                        marker(b"Cathedral pointer: app 1 recovered\n");
                    } else {
                        self.right = replacement;
                        self.b = pair;
                        marker(b"Cathedral pointer: app 2 recovered\n");
                    }
                }
            }
            match source.wait_until(time::now().unwrap()) {
                Ok(_) => {
                    source = source_launch.spawn(0).unwrap();
                    source_ready();
                    marker(b"Cathedral pointer: source recovered\n");
                }
                Err(Error(code)) if code == abi::TIMED_OUT as i64 => (),
                Err(error) => panic!("source supervision: {error:?}"),
            }
            let _ = self
                .display
                .1
                .receive_until(&mut [0; 64], time::after(10).unwrap());
        }
    }
    fn grant_pointer(&self) -> u64 {
        let reply = exchange(self.display, [p::LEASE, 1, 1, 0, 0, 0]);
        assert_eq!(reply[0], 0);
        reply[1]
    }
    fn pointer_boundaries(&self, lease: u64) {
        assert_eq!(exchange(self.a, [p::DEVICE, 1, 1, 1, 0, 0])[0], abi::DENIED);
        assert_eq!(
            exchange(self.a, [p::DRIVE, lease, 0, 0, 0, 0])[0],
            abi::DENIED
        );
        assert_eq!(
            exchange(self.b, [p::CAPTURE, 0, 1, 0, 0, 0])[0],
            abi::DENIED
        );
        assert_eq!(
            exchange(self.b, [p::DRIVE, lease, u64::MAX, u64::MAX, 0, 0])[0],
            0
        );
        let state = call(self.display, p::STATE, 1);
        assert_eq!(&state[1..3], &[495, 359]);
        assert_eq!(exchange(self.display, [p::LEASE, 1, 0, 0, 0, 0])[0], 0);
        assert_eq!(
            exchange(self.b, [p::DRIVE, lease, 0, 0, 0, 0])[0],
            abi::DENIED
        );
        // Leave a fresh incarnation-bound lease for the interactive app.
        marker(b"Cathedral pointer: hardware spoof wrong owner unearned capture bounds and stale lease passed\n");
    }
}
fn healthy(pair: Pair) -> Result<(), Error> {
    pair.0.send(&display::encode([20, 0, 0, 0, 0, 0]))?;
    pair.1.receive_until(&mut [0; 64], time::after(75)?)?;
    Ok(())
}
fn source_ready() {
    let pair = connect(3);
    let mut bytes = [0; 64];
    let len = pair
        .1
        .receive_until(&mut bytes, time::after(300).unwrap())
        .unwrap();
    assert_eq!(&bytes[..len], b"ready");
}
