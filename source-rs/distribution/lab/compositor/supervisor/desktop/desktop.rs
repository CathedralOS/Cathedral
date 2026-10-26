// Persistent session policy: admit, authorize, measure idle, supervise and restore.
use super::*;
use cathedral_platform_contracts::display::interaction as s;
use cathedral_platform_contracts::display::pointer as p;
use cathedral_user_runtime::Error;
struct Desktop {
    base: Session,
    child: Child,
    c: Client,
    source: Child,
    input: Client,
    last_focus: u64,
    pressure: Option<pressure::Pressure>,
}
impl Session {
    pub(super) fn desktop(self, child_launch: Launch) -> u64 {
        let child = child_launch.spawn(0).unwrap();
        let mut c = connect(4);
        assert_eq!(call(&mut c, 1, 1)[0], 0);
        let source = Launch::at(3).unwrap().spawn(0).unwrap();
        let input = source_ready();
        let mut desktop = Desktop {
            base: self,
            child,
            c,
            source,
            input,
            last_focus: u64::MAX,
            pressure: None,
        };
        desktop.layout();
        if cfg!(feature = "session-probes") {
            desktop.boundaries();
        }
        for pair in desktop.apps() {
            assert_eq!(call(pair, 14, 0)[0], 0);
        }
        desktop.authorize();
        if cfg!(feature = "session-probes") {
            desktop.idle_probe();
            desktop.repaint_probe();
        }
        desktop.pressure = pressure::Pressure::start();
        desktop.chrome();
        marker(b"Cathedral session: ready\n");
        desktop.run()
    }
}
impl Desktop {
    fn apps(&mut self) -> [&mut Client; 3] {
        [&mut self.base.a, &mut self.base.b, &mut self.c]
    }
    fn layout(&mut self) {
        assert_eq!(call(&mut self.base.display, w::BACKGROUND, 0x101827)[0], 0);
        place(&mut self.base.display, 0, 80, 160);
        place(&mut self.base.display, 1, 400, 160);
        assert_eq!(call(&mut self.base.display, p::ENABLE, 1)[0], 0);
    }
    fn authorize(&mut self) {
        let left = exchange(
            &mut self.base.display,
            [s::FOCUS_GRANT, 0, s::KEY_MASK, 0, 0, 0],
        );
        assert_eq!(left[0], 0);
        self.configure(0, [22, 0, left[1], 0, 0, 0]);
        self.authorize_host();
    }
    fn authorize_host(&mut self) {
        let display = &mut self.base.display;
        let focus = exchange(display, [s::FOCUS_GRANT, 1, s::KEY_MASK, 1, 0, 0]);
        assert_eq!(focus[0], 0);
        let host = exchange(display, [s::HOST_GRANT, 1, 2, 0, 0, 0]);
        assert_eq!(host[0], 0);
        let lease = exchange(display, [p::LEASE, 1, 1, 0, 0, 0]);
        assert_eq!(lease[0], 0);
        let child_focus = self.configure(1, [22, lease[1], focus[1], host[1], 0, 0])[1];
        self.configure(2, [22, 0, child_focus, 0, 0, 0]);
    }
    fn configure(&mut self, client: usize, words: [u64; 6]) -> [u64; 6] {
        self.apps()[client]
            .begin(&display::encode(words), None)
            .unwrap();
        assert_eq!(call(&mut self.base.display, p::WAKE, client as u64)[0], 0);
        let reply =
            crate::transport::finish(self.apps()[client], time::after(300).unwrap()).unwrap();
        assert_eq!(reply[0], 0);
        reply
    }
    fn run(&mut self) -> u64 {
        loop {
            match crate::try_exchange(&mut self.base.display, [s::SESSION_CONTROL, 0, 0, 0, 0, 0]) {
                Ok(reply) if reply[1] == 1 => self.restore_display(),
                Ok(reply) if reply[1] == 2 => self.replace_app(1),
                Ok(reply) if reply[1] == 3 => {
                    let focus = call(&mut self.base.display, s::FOCUS_STATE, 0);
                    self.configure(1, [23, 0, 0, 0, 0, 0]);
                    assert_eq!(call(&mut self.base.display, s::FOCUS_STATE, 0), focus);
                    marker(b"Cathedral session: synthetic click preserved keyboard focus\n");
                    self.notice(b"SYNTHETIC CLICK");
                }
                Ok(_) => (),
                Err(_) => self.restore_display(),
            }
            for client in 0..3 {
                if health(self.apps()[client]).is_err() {
                    self.replace_app(client);
                }
            }
            if source_call(&mut self.input, 1).is_err() {
                self.replace_source();
            }
            self.update_focus();
            if let Some(pressure) = &mut self.pressure {
                pressure.tick();
            }
            crate::transport::idle(50);
        }
    }
}
fn health(pair: &mut Client) -> Result<[u64; 6], Error> {
    let (bytes, len) = pair.call(&display::encode([20, 0, 0, 0, 0, 0]), time::after(100)?)?;
    display::decode(&bytes[..len]).ok_or(Error(abi::IO_ERROR as i64))
}
fn source_ready() -> Client {
    let mut pair = connect(3);
    let (bytes, len) = pair.call(&[1], time::after(300).unwrap()).unwrap();
    assert_eq!(&bytes[..len], b"ready");
    pair
}
fn source_call(pair: &mut Client, command: u8) -> Result<(), Error> {
    let (bytes, len) = pair.call(&[command], time::after(100)?)?;
    if &bytes[..len] != b"ready" {
        return Err(Error(abi::IO_ERROR as i64));
    }
    Ok(())
}
