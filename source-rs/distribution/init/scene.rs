//! All navigation, appearance and restart shortcuts belong to the distribution.
use super::service::Service;
use cathedral_contracts::{input as wire, user as abi};
use cathedral_user_runtime::{Error, write};

#[derive(Default)]
struct Scene {
    selected: u8,
    active: u8,
}

pub fn run() -> Result<(), Error> {
    let mut display = Service::start(0)?;
    let mut input = Service::start(1)?;
    input_ready(&input)?;
    let mut scene = Scene::default();
    scene.draw(&mut display)?;
    write(b"Cathedral: startup ready\n")?;
    loop {
        let event = next(&mut input)?;
        // Repeats navigate, but toggles and restart shortcuts require a new press.
        if event.state == wire::RELEASE {
            continue;
        }
        match event.key {
            wire::LEFT | wire::UP => scene.selected = (scene.selected + 2) % 3,
            wire::RIGHT | wire::DOWN => scene.selected = (scene.selected + 1) % 3,
            wire::ENTER if event.state == wire::PRESS => scene.active ^= 1 << scene.selected,
            wire::F1 if event.state == wire::PRESS => {
                input.restart()?;
                input_ready(&input)?;
                write(b"Cathedral: input restarted\n")?;
            }
            wire::F2 if event.state == wire::PRESS => {
                display.restart()?;
                write(b"Cathedral: display restarted\n")?;
            }
            _ => continue,
        }
        scene.draw(&mut display)?;
        scene.report()?;
    }
}
fn input_ready(input: &Service) -> Result<(), Error> {
    if receive(input)? != wire::Event::reset() {
        return Err(Error(abi::IO_ERROR as i64));
    }
    Ok(())
}
fn receive(input: &Service) -> Result<wire::Event, Error> {
    let mut bytes = [0; 64];
    let length = input.receive.receive(&mut bytes)?;
    wire::Event::decode(&bytes[..length]).ok_or(Error(abi::IO_ERROR as i64))
}
fn next(input: &mut Service) -> Result<wire::Event, Error> {
    for attempt in 0..3 {
        match input.send.send(&wire::NEXT).and_then(|()| receive(input)) {
            Ok(event) => return Ok(event),
            Err(_) if attempt < 2 => {
                input.restart()?;
                input_ready(input)?;
            }
            Err(error) => return Err(error),
        }
    }
    unreachable!()
}
impl Scene {
    fn draw(&self, display: &mut Service) -> Result<(), Error> {
        for attempt in 0..3 {
            let result = cathedral_boot_scene::dimensions(display.send, display.receive).and_then(
                |(w, h)| {
                    cathedral_boot_scene::draw_interactive(
                        display.send,
                        display.receive,
                        w,
                        h,
                        self.selected,
                        self.active,
                    )
                },
            );
            match result {
                Ok(()) => return Ok(()),
                Err(_) if attempt < 2 => display.restart()?,
                Err(error) => return Err(error),
            }
        }
        unreachable!()
    }
    fn report(&self) -> Result<(), Error> {
        let mut message = *b"Cathedral: selection=0 toggles=0\n";
        message[21] = b'0' + self.selected;
        message[31] = b'0' + self.active;
        write(&message)
    }
}
