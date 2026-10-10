//! Distribution lifetime policy. Drawing and application state live in another task.
use super::service::Service;
use cathedral_contracts::{display, input, user as abi};
use cathedral_session_protocol as session;
use cathedral_user_runtime::{Error, time, write};

pub fn run() -> Result<(), Error> {
    let mut display = Service::start(0)?;
    let mut input = Service::start(1)?;
    input_ready(&mut input)?;
    healthy(&mut display, false)?;
    let mut app = Service::start(2)?;
    let mut last = 0;
    let mut seen = time::now()?;
    let mut started = false;
    #[cfg(feature = "recovery-lab")]
    let mut probes = super::recovery::Probes::default();
    loop {
        if healthy(&mut display, false)? {
            last = 1;
        }
        if healthy(&mut input, true)? {
            last = 2;
        }
        #[cfg(feature = "recovery-lab")]
        probes.verify(&display, &input, &app)?;
        if app.stopped()? {
            let peers = (display.ticket(), input.ticket());
            app.respawn()?;
            assert_eq!((display.ticket(), input.ticket()), peers);
            last = 3;
            seen = time::now()?;
            write(b"Cathedral: application restarted; providers preserved\n")?;
        }
        let mut bytes = [0; 64];
        let length = match app.receive.receive_until(&mut bytes, time::after(10)?) {
            Ok(length) => length,
            Err(Error(code)) if code == abi::PEER_CLOSED as i64 => continue,
            Err(Error(code)) if code == abi::TIMED_OUT as i64 => {
                if time::reached(time::now()?, seen.wrapping_add(300)) {
                    let peers = (display.ticket(), input.ticket());
                    app.restart()?;
                    assert_eq!((display.ticket(), input.ticket()), peers);
                    last = 3;
                    seen = time::now()?;
                    write(b"Cathedral: application restarted; providers preserved\n")?;
                }
                continue;
            }
            Err(error) => return Err(error),
        };
        seen = time::now()?;
        let status = session::Status {
            display: display.generation(),
            input: input.generation(),
            application: app.generation(),
            last,
        };
        let encoded = status.encode();
        let response: &[u8] = match &bytes[..length] {
            session::STATUS => &encoded,
            session::READY => b"ok",
            #[cfg(feature = "recovery-lab")]
            [0xf0, index @ 0..=1] => {
                if *index == 0 {
                    probes.inject(0, &display, &input, &app)?;
                } else {
                    probes.inject(1, &input, &display, &app)?;
                }
                b"ok"
            }
            _ => b"denied",
        };
        // App closure or an undrained response must not take init/providers down.
        if app.send.send(response).is_err() {
            let peers = (display.ticket(), input.ticket());
            app.restart()?;
            assert_eq!((display.ticket(), input.ticket()), peers);
            last = 3;
            seen = time::now()?;
            write(b"Cathedral: application restarted; providers preserved\n")?;
        } else if &bytes[..length] == session::READY {
            write(if started {
                b"Cathedral: application ready\n"
            } else {
                b"Cathedral: startup ready\n"
            })?;
            started = true;
        }
    }
}
fn input_ready(input: &mut Service) -> Result<(), Error> {
    for attempt in 0..3 {
        if let Ok((bytes, length)) = input.receive()
            && input::Event::decode(&bytes[..length]) == Some(input::Event::reset())
        {
            if attempt != 0 {
                input.recovered()?;
            }
            return Ok(());
        }
        if attempt < 2 {
            input.restart()?;
        }
    }
    Err(Error(abi::IO_ERROR as i64))
}
fn healthy(service: &mut Service, keyboard: bool) -> Result<bool, Error> {
    for attempt in 0..3 {
        let send = if keyboard {
            service.send.send(&input::HEALTH)
        } else {
            service
                .send
                .send(&display::encode([display::INFO, 0, 0, 0, 0, 0]))
        };
        let valid = send
            .and_then(|()| service.receive())
            .is_ok_and(|(bytes, length)| {
                if keyboard {
                    input::Event::decode(&bytes[..length]) == Some(input::Event::idle())
                } else {
                    display::decode(&bytes[..length]).is_some_and(|reply| reply[0] == 0)
                }
            });
        if valid {
            if attempt != 0 {
                service.recovered()?;
            }
            return Ok(attempt != 0);
        }
        if attempt < 2 {
            service.restart()?;
            if keyboard {
                input_ready(service)?;
            }
        }
    }
    Err(Error(abi::IO_ERROR as i64))
}
