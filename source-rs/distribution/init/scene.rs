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
    input_ready(&mut input)?;
    let mut scene = Scene::default();
    scene.draw(&mut display)?;
    write(b"Cathedral: startup ready\n")?;
    #[cfg(feature = "recovery-lab")]
    let mut faults = super::recovery::Probes::default();
    loop {
        let input_generation = input.generation();
        let event = next(&mut input)?;
        // A healthy idle reply is also an opportunity to check display health.
        if event.key == wire::IDLE {
            if display.drawing()?.dimensions().is_err() {
                display.restart()?;
                scene.draw(&mut display)?;
                display.recovered()?;
                scene.report()?;
            }
            if input.generation() != input_generation {
                scene.report()?;
            }
            #[cfg(feature = "recovery-lab")]
            faults.verify(&display, &input)?;
            continue;
        }
        // Repeats navigate, but toggles and restart shortcuts require a new press.
        if event.state == wire::RELEASE {
            continue;
        }
        match event.key {
            wire::LEFT | wire::UP => scene.selected = (scene.selected + 2) % 3,
            wire::RIGHT | wire::DOWN => scene.selected = (scene.selected + 1) % 3,
            wire::ENTER if event.state == wire::PRESS => scene.active ^= 1 << scene.selected,
            wire::F1 if event.state == wire::PRESS => {
                #[cfg(feature = "recovery-lab")]
                faults.inject(1, &input, &display)?;
                #[cfg(not(feature = "recovery-lab"))]
                {
                    input.restart()?;
                    input_ready(&mut input)?;
                    write(b"Cathedral: input restarted\n")?;
                }
            }
            wire::F2 if event.state == wire::PRESS => {
                #[cfg(feature = "recovery-lab")]
                {
                    faults.inject(0, &display, &input)?;
                    continue; // Let idle health checks or the next real draw discover failure.
                }
                #[cfg(not(feature = "recovery-lab"))]
                {
                    display.restart()?;
                    write(b"Cathedral: display restarted\n")?;
                }
            }
            _ => continue,
        }
        scene.draw(&mut display)?;
        scene.report()?;
        #[cfg(feature = "recovery-lab")]
        faults.verify(&display, &input)?;
    }
}
fn input_ready(input: &mut Service) -> Result<(), Error> {
    for attempt in 0..3 {
        match receive(input) {
            Ok(event) if event == wire::Event::reset() => {
                if attempt != 0 {
                    input.recovered()?;
                }
                return Ok(());
            }
            _ if attempt < 2 => input.restart()?,
            _ => return Err(Error(abi::IO_ERROR as i64)),
        }
    }
    unreachable!()
}
fn receive(input: &Service) -> Result<wire::Event, Error> {
    let (bytes, length) = input.receive()?;
    wire::Event::decode(&bytes[..length]).ok_or(Error(abi::IO_ERROR as i64))
}
fn next(input: &mut Service) -> Result<wire::Event, Error> {
    for attempt in 0..3 {
        match input.send.send(&wire::NEXT).and_then(|()| receive(input)) {
            Ok(event) => return Ok(event),
            Err(_) if attempt < 2 => {
                input.restart()?;
                input_ready(input)?;
                input.recovered()?;
            }
            Err(error) => return Err(error),
        }
    }
    unreachable!()
}
impl Scene {
    fn draw(&self, display: &mut Service) -> Result<(), Error> {
        for attempt in 0..3 {
            let client = display.drawing()?;
            let result = client
                .dimensions()
                .and_then(|(w, h)| client.draw_interactive(w, h, self.selected, self.active));
            match result {
                Ok(()) => {
                    if attempt != 0 {
                        display.recovered()?;
                    }
                    return Ok(());
                }
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
