// Persistent app loop: retain desired state, wait for input/control, reconnect on command.
use super::*;
use cathedral_platform_contracts::display::interaction as s;
use cathedral_platform_contracts::display::pointer as p;
use cathedral_platform_contracts::input;
use cathedral_user_runtime::{
    server::{Event, Request},
    time,
};
struct App {
    role: u64,
    control: Server,
    connection: Option<Data>,
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
    configure: Option<Request>,
    drive: bool,
    events: u64,
    wakes: u64,
}
pub fn run(role: u64, control: Server, connection: Data, link: Link) -> u64 {
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
        configure: None,
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
            if self.configure.is_some() && !self.pending {
                let reply = self.configure_host().unwrap_or_else(|_| {
                    self.offline();
                    [abi::IO_ERROR, 0, 0, 0, 0, 0]
                });
                let request = self.configure.take().unwrap();
                self.control
                    .reply(&request, &display::encode(reply))
                    .unwrap();
            }
            if let Some(pair) = &mut self.connection {
                if !self.pending {
                    if pair
                        .begin(&display::encode([p::WAIT, 0, 0, 0, 0, 0]), None)
                        .is_err()
                    {
                        self.offline();
                        continue;
                    }
                    self.pending = true;
                }
                match pair.poll() {
                    Ok(Some((bytes, len, object))) => {
                        assert!(object.is_none());
                        self.wakes += 1;
                        self.pending = false;
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
                    Ok(None) => {
                        let interests = pair.interests().unwrap();
                        match self
                            .control
                            .next_event_for(&interests, Some(time::after(500).unwrap()))
                        {
                            Ok(Some(Event::Request(request))) => {
                                if !self.control(request) {
                                    return 0;
                                }
                            }
                            Ok(_) => (),
                            // A provider can die between poll and mixed wait.
                            // Drop its interests, then keep owner control alive
                            // for reconnection; only owner closure ends the app.
                            Err(_) => self.offline(),
                        }
                    }
                    Err(_) => self.offline(),
                }
            } else {
                let Ok(request) = self.control.next_request() else {
                    return 0;
                };
                if !self.control(request) {
                    return 0;
                }
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
    fn rpc(&mut self, words: [u64; 6]) -> Result<[u64; 6], Error> {
        crate::try_exchange(
            self.connection
                .as_mut()
                .ok_or(Error(abi::PEER_CLOSED as i64))?,
            words,
        )
    }
    fn control(&mut self, request: Request) -> bool {
        let Some(words) = display::decode(&request.bytes[..request.len]) else {
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
                match self
                    .link
                    .connect()
                    .and_then(|pair| Data::connect(pair, time::after(300)?))
                {
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
                self.configure = Some(request);
                return true;
            }
            [22, lease, focus, host, 0, 0] => {
                self.lease = lease;
                self.focus = focus;
                self.host = host;
                self.configure = Some(request);
                return true;
            }
            _ => [abi::INVALID_ARGUMENT, 0, 0, 0, 0, 0],
        };
        self.control
            .reply(&request, &display::encode(reply))
            .is_ok()
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
