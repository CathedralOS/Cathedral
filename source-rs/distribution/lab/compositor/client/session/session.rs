// Persistent app loop: retain desired state, wait for input/control, reconnect on command.
use super::*;
use cathedral_contracts::{input, interaction as s, pointer as p};
use cathedral_user_runtime::{ipc, time};
type Pair = (Handle, Handle);
struct App {
    role: u64,
    control: Pair,
    connection: Option<Pair>,
    link: Link,
    pending: bool,
    revision: u64,
    value: u64,
    selected: u64,
    marker: Option<(u64, u64)>,
    held: [u8; 2],
    lease: u64,
    focus: u64,
    host: u64,
    configure: bool,
    drive: bool,
    events: u64,
    wakes: u64,
}
pub fn run(role: u64, control: Pair, connection: Pair, link: Link) -> u64 {
    let mut app = App {
        role,
        control,
        connection: Some(connection),
        link,
        pending: false,
        revision: 1,
        value: 0,
        selected: 0,
        marker: None,
        held: [0; 2],
        lease: 0,
        focus: 0,
        host: 0,
        configure: false,
        drive: false,
        events: 0,
        wakes: 0,
    };
    if app.publish().is_err() {
        app.offline();
    }
    app.run()
}
impl App {
    fn run(&mut self) -> u64 {
        loop {
            if self.configure && !self.pending {
                let reply = self.configure_host().unwrap_or_else(|_| {
                    self.offline();
                    [abi::IO_ERROR, 0, 0, 0, 0, 0]
                });
                self.configure = false;
                self.control.1.send(&display::encode(reply)).unwrap();
            }
            if let Some(pair) = self.connection {
                if !self.pending {
                    if pair
                        .0
                        .send(&display::encode([p::WAIT, 0, 0, 0, 0, 0]))
                        .is_err()
                    {
                        self.offline();
                        continue;
                    }
                    self.pending = true;
                }
                match ipc::wait_two(self.control.0, pair.1, time::after(500).unwrap()) {
                    Ok(0) => {
                        if !self.control() {
                            return 0;
                        }
                    }
                    Ok(_) => {
                        self.wakes += 1;
                        let mut bytes = [0; 64];
                        self.pending = false;
                        match pair.1.receive(&mut bytes) {
                            Ok(len) => {
                                if let Some(words) = display::decode(&bytes[..len]) {
                                    if words[0] != 0 {
                                        self.offline();
                                    } else if words[2] != 0 {
                                        self.events += 1;
                                        if self.event(words).is_err() {
                                            self.offline();
                                        }
                                    }
                                }
                            }
                            Err(_) => self.offline(),
                        }
                    }
                    Err(Error(code)) if code == abi::TIMED_OUT as i64 => (),
                    Err(_) => self.offline(),
                }
            } else if !self.control() {
                return 0;
            }
        }
    }
    fn offline(&mut self) {
        self.connection = None;
        self.pending = false;
        self.held = [0; 2];
        self.lease = 0;
        self.focus = 0;
        self.host = 0;
    }
    fn rpc(&self, words: [u64; 6]) -> Result<[u64; 6], Error> {
        crate::try_exchange(
            self.connection.ok_or(Error(abi::PEER_CLOSED as i64))?,
            words,
        )
    }
    fn control(&mut self) -> bool {
        let mut bytes = [0; 64];
        let Ok(len) = self.control.0.receive(&mut bytes) else {
            return false;
        };
        let Some(words) = display::decode(&bytes[..len]) else {
            return false;
        };
        let reply = match words {
            [20, 0, 0, 0, 0, 0] => [
                0,
                self.events,
                self.wakes,
                self.value,
                u64::from(self.connection.is_some()),
                self.revision,
            ],
            [21, 0, 0, 0, 0, 0] => {
                self.offline();
                match self.link.connect() {
                    Ok(pair) => {
                        self.connection = Some(pair);
                        self.revision = 0;
                        if self.publish().is_ok() {
                            [0; 6]
                        } else {
                            self.offline();
                            [abi::IO_ERROR, 0, 0, 0, 0, 0]
                        }
                    }
                    Err(error) => [error.0 as u64, 0, 0, 0, 0, 0],
                }
            }
            [23, 0, 0, 0, 0, 0] => {
                self.drive = true;
                self.configure = true;
                return true;
            }
            [22, lease, focus, host, 0, 0] => {
                self.lease = lease;
                self.focus = focus;
                self.host = host;
                self.configure = true;
                return true;
            }
            _ => [abi::INVALID_ARGUMENT, 0, 0, 0, 0, 0],
        };
        self.control.1.send(&display::encode(reply)).is_ok()
    }
    fn configure_host(&mut self) -> Result<[u64; 6], Error> {
        if self.drive {
            self.drive = false;
            self.rpc([p::DRIVE, self.lease, 40, 40, 1, 0])?;
            return self.rpc([p::DRIVE, self.lease, 40, 40, 0, 0]);
        }
        if self.host == 0 {
            return Ok([0; 6]);
        }
        assert_eq!(
            self.rpc([s::HOST_PLACE, self.host, 100, 80, 200, 160])?[0],
            0
        );
        let child = self.rpc([
            s::FOCUS_DELEGATE,
            self.focus,
            2,
            1 << (input::ENTER - 1),
            0,
            0,
        ])?;
        assert_eq!(child[0], 0);
        Ok(child)
    }
}
