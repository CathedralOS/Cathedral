//! Restore shared services without replacing surviving application processes.
use super::*;
impl Desktop {
    pub(super) fn restore_display(&mut self) {
        let before = self.apps().map(|pair| health(pair).unwrap()[3]);
        let identities = [
            self.base.left.raw(),
            self.base.right.raw(),
            self.child.raw(),
            self.source.raw(),
        ];
        self.base.provider.cancel().unwrap();
        self.base.provider.wait().unwrap();
        self.base.provider = Launch::at(0).unwrap().spawn(0).unwrap();
        self.base.display = connect(0);
        self.layout();
        for pair in self.apps() {
            assert_eq!(call(pair, 21, 0)[0], 0);
        }
        self.authorize();
        source_call(self.input, 2).unwrap();
        self.chrome();
        self.notice(b"DISPLAY RESTORED");
        assert_eq!(self.apps().map(|pair| health(pair).unwrap()[3]), before);
        assert_eq!(
            [
                self.base.left.raw(),
                self.base.right.raw(),
                self.child.raw(),
                self.source.raw()
            ],
            identities
        );
        assert_eq!(call(self.base.display, p::STATE, 0)[4], 0);
        assert_eq!(call(self.base.display, s::FOCUS_STATE, 0)[1], 0);
        marker(b"Cathedral session: compositor restored; three app identities and values preserved; capture and focus cleared\n");
    }
    pub(super) fn replace_app(&mut self, client: usize) {
        self.notice(b"RESTARTING APP");
        let child_value = if client == 1 {
            Some(health(self.c).unwrap()[3])
        } else {
            None
        };
        let (child, launch) = match client {
            0 => (self.base.left, 1),
            1 => (self.base.right, 2),
            _ => (self.child, 4),
        };
        child.cancel().unwrap();
        child.wait().unwrap();
        gone(self.base.display, client as u64);
        if client == 1 {
            // Root drops the hosted child's pixels without terminating that child.
            pixel(self.base.display, 510, 250, 0x101827);
            marker(b"Cathedral session: host loss hid child and revoked descendant input\n");
        }
        let replacement = Launch::at(launch).unwrap().spawn(0).unwrap();
        let pair = connect(launch);
        assert_eq!(call(pair, 1, 1)[0], 0);
        assert_eq!(call(pair, 14, 0)[0], 0);
        match client {
            0 => {
                self.base.left = replacement;
                self.base.a = pair;
            }
            1 => {
                self.base.right = replacement;
                self.base.b = pair;
            }
            _ => {
                self.child = replacement;
                self.c = pair;
            }
        }
        if client == 0 {
            let grant = exchange(self.base.display, [s::FOCUS_GRANT, 0, s::KEY_MASK, 0, 0, 0]);
            assert_eq!(grant[0], 0);
            self.configure(0, [22, 0, grant[1], 0, 0, 0]);
        } else {
            self.authorize_host();
        }
        if let Some(value) = child_value {
            assert_eq!(health(self.c).unwrap()[3], value);
            marker(b"Cathedral session: host replaced; child process and value preserved\n");
            self.notice(b"HOST RESTORED");
        } else {
            marker(b"Cathedral session: app recovered\n");
            self.notice(b"APP RESTORED");
        }
    }
    pub(super) fn replace_source(&mut self) {
        self.notice(b"RESTARTING INPUT");
        self.source.cancel().unwrap();
        self.source.wait().unwrap();
        self.source = Launch::at(3).unwrap().spawn(0).unwrap();
        self.input = source_ready();
        marker(b"Cathedral session: input source restored\n");
        self.notice(b"INPUT RESTORED");
    }
}
