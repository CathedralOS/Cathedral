//! Menu interaction over local input. Its private owner connection reports a
//! sequence and fixed choice; the distribution validates and performs admission.
use super::*;
use cathedral_user_runtime::task::Environment;
pub fn run(environment: Environment) -> u64 {
    let connections = cathedral_user_runtime::matrix::Connections::load(&environment).unwrap();
    let parent = connections.interaction;
    let control = (Handle::bootstrap(0).unwrap(), Handle::bootstrap(1).unwrap());
    let (mut selected, mut sequence, mut requested) = (NOTES, 0u64, 0);
    let (mut active, mut blocked, mut pressed) = (0, false, false);
    let mut scene: Option<view::Scene> = None;
    let mut repaint = true;
    loop {
        let snapshot = call(parent, [wire::POLL, 0, 0, 0, 0, 0, 0, 0]);
        let extent = wire::poll_view(snapshot);
        if extent[0] == 0 {
            let event = wire::poll_event(snapshot);
            let mut choose = false;
            match event[2] {
                wire::POINTER => {
                    let down = event[5] & 1 != 0;
                    if down
                        && !pressed
                        && (10..190).contains(&event[3])
                        && (20..68).contains(&event[4])
                    {
                        selected = (event[4] - 20) / 16 + 1;
                        choose = true;
                    }
                    pressed = down;
                }
                wire::CANCEL => pressed = false,
                wire::KEY if event[4] == input::PRESS as u64 => match event[3] as u8 {
                    input::UP => {
                        selected = if selected == NOTES {
                            CLOSE
                        } else {
                            selected - 1
                        };
                        repaint = true;
                    }
                    input::DOWN => {
                        selected = if selected == CLOSE {
                            NOTES
                        } else {
                            selected + 1
                        };
                        repaint = true;
                    }
                    input::ENTER => choose = true,
                    _ => (),
                },
                _ => (),
            }
            if choose {
                requested = selected;
                sequence = sequence.wrapping_add(1);
                repaint = true;
            }
            if scene.as_ref().is_none_or(|s| s.epoch != extent[1]) {
                scene = Some(view::Scene::new(parent, extent[1]));
                repaint = true;
            }
            if repaint {
                scene
                    .as_ref()
                    .unwrap()
                    .paint(parent, extent, selected, active, blocked);
                repaint = false;
            }
        } else {
            scene = None;
        }
        if snapshot[5] & wire::MORE != 0 {
            continue;
        }
        let mut bytes = [0; 64];
        match ipc::wait_many(&[connections.wake, control.0], time::after(100).unwrap()) {
            Ok(0) => {
                connections.wake.receive(&mut bytes).unwrap();
            }
            Ok(_) => {
                let len = control.0.receive(&mut bytes).unwrap();
                let words = wire::decode(&bytes[..len]).unwrap();
                if words[0] == MENU_STATUS {
                    repaint |= active != words[1] || blocked != (words[2] != 0);
                    active = words[1];
                    blocked = words[2] != 0;
                    control
                        .1
                        .send(&wire::encode([0, sequence, requested, 0, 0, 0, 0, 0]))
                        .unwrap();
                } else {
                    control
                        .1
                        .send(&wire::encode(wire::error(abi::DENIED)))
                        .unwrap();
                }
            }
            Err(Error(code)) if code == abi::TIMED_OUT as i64 => (),
            Err(error) => panic!("launcher wait {error:?}"),
        }
    }
}
