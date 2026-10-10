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
    let mut storage = Service::start(3)?;
    storage_healthy(&mut storage)?;
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
        if storage_healthy(&mut storage)? {
            last = 4;
        }
        #[cfg(feature = "recovery-lab")]
        probes.verify(&display, &input, &storage, &app)?;
        if app.stopped()? {
            let peers = (display.ticket(), input.ticket(), storage.ticket());
            app.respawn()?;
            assert_eq!((display.ticket(), input.ticket(), storage.ticket()), peers);
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
                    let peers = (display.ticket(), input.ticket(), storage.ticket());
                    app.restart()?;
                    assert_eq!((display.ticket(), input.ticket(), storage.ticket()), peers);
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
            storage: storage.generation(),
        };
        let encoded = status.encode();
        let response: &[u8] = match &bytes[..length] {
            session::STATUS => &encoded,
            session::READY => b"ok",
            #[cfg(feature = "recovery-lab")]
            [0xf0, index @ 0..=2] => {
                if *index == 0 {
                    probes.inject(0, [&display, &input, &storage], &app)?;
                } else {
                    probes.inject(*index as usize, [&display, &input, &storage], &app)?;
                }
                b"ok"
            }
            #[cfg(feature = "recovery-lab")]
            [0xf1, phase @ 1..=4] => {
                storage.send.send(&[0xf1, *phase])?;
                let (reply, len) = storage.receive()?;
                assert_eq!(&reply[..len], b"armed");
                b"armed"
            }
            _ => b"denied",
        };
        // App closure or an undrained response must not take init/providers down.
        if app.send.send(response).is_err() {
            let peers = (display.ticket(), input.ticket(), storage.ticket());
            app.restart()?;
            assert_eq!((display.ticket(), input.ticket(), storage.ticket()), peers);
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

fn storage_healthy(service: &mut Service) -> Result<bool, Error> {
    for attempt in 0..3 {
        if service
            .send
            .send(cathedral_contracts::storage::HEALTH)
            .and_then(|()| service.receive())
            .is_ok_and(|(bytes, len)| &bytes[..len] == b"ready")
        {
            if attempt != 0 {
                write(b"Cathedral: storage recovered\n")?;
            }
            return Ok(attempt != 0);
        }
        if attempt < 2 {
            service.restart()?;
        }
    }
    Err(Error(abi::IO_ERROR as i64))
}
