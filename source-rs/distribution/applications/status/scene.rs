//! Application-owned navigation, appearance and retained state across provider loss.
use super::connection::Connections;
use cathedral_contracts::{input, user as abi};
use cathedral_session_protocol as session;
use cathedral_user_runtime::{Error, time, write};
#[derive(Default)]
struct Scene {
    selected: u8,
    active: u8,
    saved: u64,
}
pub fn run(_generation: u64) -> Result<(), Error> {
    let mut connections = Connections::open()?;
    connections.refresh()?;
    #[cfg(feature = "recovery-lab")]
    super::probes::authority(&mut connections)?;
    let record = super::storage::load(&mut connections)?;
    let mut scene = match record.payload() {
        [] if record.generation == 0 => Scene::default(),
        [1, selected @ 0..=2, active @ 0..=7] => Scene {
            selected: *selected,
            active: *active,
            saved: record.generation,
        },
        _ => return Err(Error(abi::IO_ERROR as i64)),
    };
    scene.draw(&mut connections)?;
    connections.request(session::READY)?;
    loop {
        let event = next(&mut connections)?;
        let mut changed = connections.refresh()?;
        let previous = (scene.selected, scene.active);
        if event.state != input::RELEASE {
            let acted = match event.key {
                input::LEFT | input::UP => {
                    scene.selected = (scene.selected + 2) % 3;
                    true
                }
                input::RIGHT | input::DOWN => {
                    scene.selected = (scene.selected + 1) % 3;
                    true
                }
                input::ENTER if event.state == input::PRESS => {
                    scene.active ^= 1 << scene.selected;
                    true
                }
                #[cfg(feature = "recovery-lab")]
                input::F1 | input::F2 | input::F4 if event.state == input::PRESS => {
                    connections.request(&[
                        0xf0,
                        match event.key {
                            input::F1 => 1,
                            input::F4 => 2,
                            _ => 0,
                        },
                    ])?;
                    connections.refresh()?;
                    true
                }
                #[cfg(feature = "recovery-lab")]
                input::F3 if event.state == input::PRESS => super::probes::fail(_generation),
                #[cfg(feature = "recovery-lab")]
                input::F5..=input::F8 if event.state == input::PRESS => {
                    let phase = event.key - input::F5 + 1;
                    let (reply, len) = connections.request(&[0xf1, phase])?;
                    assert_eq!(&reply[..len], b"armed");
                    let mut message = *b"Cathedral: storage armed=0\n";
                    message[25] = b'0' + phase;
                    write(&message)?;
                    false
                }
                _ => false,
            };
            changed |= acted;
        }
        if previous != (scene.selected, scene.active) {
            scene.saved =
                super::storage::save(&mut connections, scene.saved, scene.selected, scene.active)?;
        }
        if changed {
            scene.draw(&mut connections)?;
        }
    }
}
fn next(connections: &mut Connections) -> Result<input::Event, Error> {
    for _ in 0..3 {
        let (send, receive) = connections.pair(1)?;
        let mut bytes = [0; 64];
        let result = send
            .send(&input::NEXT)
            .and_then(|()| receive.receive_until(&mut bytes, time::after(150)?));
        if let Ok(length) = result
            && let Some(event) = input::Event::decode(&bytes[..length])
        {
            return Ok(event);
        }
        connections.input = None;
        connections.refresh()?;
    }
    Err(Error(abi::IO_ERROR as i64))
}
impl Scene {
    fn draw(&self, connections: &mut Connections) -> Result<(), Error> {
        for _ in 0..3 {
            let (send, receive) = connections.pair(0)?;
            let client = cathedral_boot_scene::Client::until(send, receive, time::after(100)?);
            let result = client.dimensions().and_then(|(w, h)| {
                client.draw_interactive(w, h, self.selected, self.active)?;
                client.text(64, 96, 2, 0xe8edf4, b"CATHEDRAL / STATUS")?;
                for (x, text) in [
                    (80, &b"DISPLAY"[..]),
                    (400, &b"INPUT"[..]),
                    (720, &b"APPLICATION"[..]),
                ] {
                    client.text(x, 184, 2, 0x101827, text)?;
                }
                client.text(64, 592, 2, 0xe8edf4, b"ARROWS SELECT / ENTER TOGGLE")?;
                let status = connections.status;
                let mut counters = *b"DISPLAY READY 00  INPUT READY 00  APP 00";
                digits(&mut counters[14..16], status.display);
                digits(&mut counters[30..32], status.input);
                digits(&mut counters[38..40], status.application);
                client.text(64, 632, 2, 0x59d9cc, &counters)?;
                client.text(
                    64,
                    664,
                    2,
                    0xe8edf4,
                    match status.last {
                        1 => b"LAST RECOVERY: DISPLAY",
                        2 => b"LAST RECOVERY: INPUT",
                        3 => b"LAST RECOVERY: APPLICATION",
                        4 => b"LAST RECOVERY: STORAGE",
                        _ => b"LAST RECOVERY: NONE",
                    },
                )?;
                let mut storage = *b"STORAGE READY 00  SAVED 0000";
                digits(&mut storage[14..16], status.storage);
                digits(&mut storage[24..28], self.saved);
                client.text(64, 696, 2, 0x59d9cc, &storage)
            });
            if result.is_ok() {
                return self.report(connections.status);
            }
            connections.display = None;
            connections.refresh()?;
        }
        Err(Error(abi::IO_ERROR as i64))
    }
    fn report(&self, status: session::Status) -> Result<(), Error> {
        let mut message = *b"Cathedral: selection=0 toggles=0\n";
        message[21] = b'0' + self.selected;
        message[31] = b'0' + self.active;
        write(&message)?;
        let mut health = *b"Cathedral: health=00/00/00 last=0\n";
        digits(&mut health[18..20], status.display);
        digits(&mut health[21..23], status.input);
        digits(&mut health[24..26], status.application);
        health[32] = b'0' + status.last as u8;
        write(&health)?;
        let mut storage = *b"Cathedral: storage=00 saved=0000\n";
        digits(&mut storage[19..21], status.storage);
        digits(&mut storage[28..32], self.saved);
        write(&storage)
    }
}
fn digits(bytes: &mut [u8], value: u64) {
    let mut value = value.min(10u64.pow(bytes.len() as u32) - 1);
    for byte in bytes.iter_mut().rev() {
        *byte = b'0' + (value % 10) as u8;
        value /= 10;
    }
}
