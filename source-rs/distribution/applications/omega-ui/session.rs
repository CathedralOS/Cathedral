//! Scoped input and the app model stay here; shared UI handles construction and
//! atomic publication. The presentation regions come from this app's view.
use super::view::Demo;
use cathedral_kernel_contracts::user as abi;
use cathedral_matrix::reception::Pointers;
use cathedral_omega_ui::{Presenter, Stats, Ui};
use cathedral_platform_contracts::matrix as wire;
use cathedral_user_runtime::{
    Error,
    ipc::{self, Handle},
    memory::Private,
    services::Services,
    time,
};
const WIDTH: i32 = 360;
const HEIGHT: i32 = 336;
fn origin(view: wire::Packet) -> (usize, usize) {
    (
        (view[2] as usize).saturating_sub(WIDTH as usize) / 2,
        (view[3] as usize).saturating_sub(HEIGHT as usize) / 2,
    )
}
pub fn run(mut services: Services) -> u64 {
    services.enable_shared().unwrap();
    let connection = services.interaction(1, 1).unwrap();
    cathedral_user_runtime::write(b"Cathedral Omega UI: shared interaction ready\n").unwrap();
    let control = (Handle::bootstrap(0).unwrap(), Handle::bootstrap(1).unwrap());
    let api = cathedral_omega_ui::native_api();
    let (bytes, _) = api.layout().unwrap();
    let mut state = Private::allocate(bytes.div_ceil(4096)).unwrap();
    let mut packet = Private::allocate(4).unwrap();
    let mut ui = Ui::new(api, state.bytes_mut(), packet.bytes_mut()).unwrap();
    ui.resize(WIDTH, HEIGHT).unwrap();
    let mut demo = Demo::default();
    let mut stats = rebuild(&mut demo, &mut ui);
    let mut last_ms = time::stamp().unwrap()[1] / 1000;
    let mut pointers = Pointers::default();
    let mut scene: Option<Presenter> = None;
    let mut announced = false;
    loop {
        let now_ms = time::stamp().unwrap()[1] / 1000;
        if stats.animating {
            ui.tick(now_ms.saturating_sub(last_ms).min(1000) as i32)
                .unwrap();
            stats = rebuild(&mut demo, &mut ui);
        }
        last_ms = now_ms;
        let mut snapshot = services
            .request_interaction([wire::POLL, 0, 0, 0, 0, 0, 0, 0], None)
            .unwrap();
        let mut view = wire::poll_view(snapshot);
        let mut more = false;
        for index in 0..16 {
            view = wire::poll_view(snapshot);
            if view[0] != 0 {
                scene = None;
                pointers.reset();
                break;
            }
            if scene.as_ref().is_none_or(|scene| scene.epoch() != view[1]) {
                pointers.reset();
                ui.resize(WIDTH, HEIGHT).unwrap();
                stats = rebuild(&mut demo, &mut ui);
                scene = Some(Presenter::new(view).unwrap());
                announced = false;
            }
            let event = wire::poll_event(snapshot);
            let (left, top) = origin(view);
            if let Some(buttons) = pointers.update(event) {
                ui.pointer(
                    event[3] as i32 - left as i32,
                    event[4] as i32 - top as i32,
                    buttons.held & 1 != 0,
                    buttons.pressed & 1 != 0,
                    buttons.released & 1 != 0,
                )
                .unwrap();
                stats = rebuild(&mut demo, &mut ui);
            } else if matches!(event[2], wire::CANCEL | wire::RETIRED) {
                ui.pointer(-1, -1, false, false, true).unwrap();
                stats = rebuild(&mut demo, &mut ui);
            } else if event[2] == wire::KEY
                && matches!(event[4], 1 | 2)
                && let Some((key, character)) = super::keys::translate(event[3])
            {
                ui.key(key, character, false, false).unwrap();
                stats = rebuild(&mut demo, &mut ui);
                #[cfg(feature = "trace-input")]
                if character != 0 || key == 8 {
                    cathedral_user_runtime::write(b"Omega UI text: ").unwrap();
                    cathedral_user_runtime::write(demo.text.as_str().as_bytes()).unwrap();
                    cathedral_user_runtime::write(b"\n").unwrap();
                }
            }
            if demo.closed {
                if let Some(scene) = scene.as_mut() {
                    scene.retire(&mut services).unwrap();
                }
                return 0;
            }
            more = snapshot[5] & wire::MORE != 0;
            if !more || index == 15 {
                break;
            }
            snapshot = services
                .request_interaction([wire::POLL, 0, 0, 0, 0, 0, 0, 0], None)
                .unwrap();
        }
        // Apply every edge, then publish the final model once per bounded batch.
        if let Some(scene) = scene.as_mut() {
            scene
                .present(
                    &mut services,
                    &mut ui,
                    &demo.regions[..demo.region_count],
                    origin(view),
                )
                .unwrap();
            if !announced {
                cathedral_user_runtime::write(b"Omega UI surface ready\n").unwrap();
                announced = true;
            }
        }
        if more {
            continue;
        }
        let mut bytes = [0; 64];
        match ipc::wait_many(
            &[connection.wake, control.0],
            time::after(if stats.animating { 1 } else { 100 }).unwrap(),
        ) {
            Ok(0) => {
                connection.wake.receive(&mut bytes).unwrap();
            }
            Ok(_) => {
                let length = control.0.receive(&mut bytes).unwrap();
                let operation = wire::decode(&bytes[..length]).map_or(0, |packet| packet[0]);
                control
                    .1
                    .send(&wire::encode(if operation == wire::LOSS {
                        [0; 8]
                    } else {
                        [
                            0,
                            demo.clicks as u64,
                            demo.volume as u64,
                            demo.text.as_str().len() as u64,
                            stats.revision as u64,
                            0,
                            0,
                            0,
                        ]
                    }))
                    .unwrap();
            }
            Err(Error(code)) if code == abi::TIMED_OUT as i64 => (),
            Err(error) => panic!("Omega UI wait: {error:?}"),
        }
    }
}
fn rebuild(demo: &mut Demo, ui: &mut Ui<'_, '_>) -> Stats {
    #[cfg(feature = "trace-input")]
    let before = (
        demo.clicks,
        demo.volume,
        demo.enabled,
        demo.text.as_str().len(),
    );
    let stats = demo.build(ui).unwrap();
    #[cfg(feature = "trace-input")]
    {
        if before.0 != demo.clicks {
            cathedral_user_runtime::write(b"Omega UI button applied\n").unwrap();
        }
        if before.1 != demo.volume && demo.volume > 65 {
            cathedral_user_runtime::write(b"Omega UI slider above 65\n").unwrap();
        }
        if before.2 != demo.enabled {
            cathedral_user_runtime::write(b"Omega UI switch applied\n").unwrap();
        }
        if before.3 > 0 && demo.text.as_str().is_empty() {
            cathedral_user_runtime::write(b"Omega UI field cleared\n").unwrap();
        }
        if before.1 != 50 && demo.volume == 50 {
            cathedral_user_runtime::write(b"Omega UI controls reset\n").unwrap();
        }
    }
    stats
}
