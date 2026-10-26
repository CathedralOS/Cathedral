// List the realm, present users/apps, then launch a choice or copy the template.
// Labels and hidden entries are distribution presentation; every request stays
// within the scoped object identities returned by this child's realm.
use cathedral_chooser::policy::{self, Action, Operation};
use cathedral_desktop_matrix_contracts::matrix as wire;
use cathedral_kernel_contracts::user as abi;
use cathedral_platform_contracts::input;
use cathedral_user_runtime::{
    Error,
    server::{OwnerControl, SharedSlot},
    services::Services,
    time,
};
pub use policy::{ENTRIES, NEW, USER_ENTRIES};

/// `own` is this program's object in its realm, from its launch argument; the
/// launcher hides that tile instead of reading which entry it is.
pub fn run(
    mut services: Services,
    user_picker: bool,
    initial_status: &'static str,
    own: u64,
) -> u64 {
    policy::verify();
    static mut CONTROL: SharedSlot = SharedSlot::empty();
    let slot = &raw mut CONTROL;
    // SAFETY: The single probe entry owns this control slot until exit.
    let mut control = OwnerControl::open(unsafe { &mut *slot }).unwrap();
    control.establish(time::after(300).unwrap()).unwrap();
    services.enable_shared().unwrap();
    let wake = services.interaction(1, 1).unwrap().wake;
    assert!(matches!(
        wake,
        cathedral_user_runtime::services::Wake::Notification(_)
    ));
    let scope =
        cathedral_user_runtime::storage::Scope::connect(services.storage(1, 1).unwrap()).unwrap();
    let parent = &mut services;
    cathedral_user_runtime::write(
        b"Cathedral chooser probe: shared interaction and storage ready\n",
    )
    .unwrap();
    let mut realm = realm::Realm::new(scope, user_picker, own);
    let mut status = match realm.reload() {
        Ok(()) => initial_status,
        Err(_) => "CANNOT OPEN",
    };
    let mut selection = selection::Selection::new();
    let mut pointers = cathedral_matrix::reception::Pointers::default();
    let mut scene: Option<view::Scene> = None;
    let mut repaint = true;
    let mut editor: Option<settings::Editor> = None;
    let mut launched = None;
    let mut tasks = tasks::Tasks::default();
    let mut management = management::Controls::default();
    let mut saves = wire::SAVES_READY;
    #[cfg(feature = "recovery-probes")]
    let mut management_probe = management::probe::Probe::default();
    loop {
        if !user_picker {
            repaint |= tasks.refresh(parent, false);
            repaint |= management.refresh(parent);
            let reply = call(parent, [wire::SAVE_AVAILABILITY, 0, 0, 0, 0, 0, 0, 0]);
            let next = if reply[0] == 0 {
                reply[1]
            } else {
                wire::SAVES_READY
            };
            repaint |= saves != next;
            saves = next;
        }
        if let Some(receipt) = launched {
            let reply = call(parent, [wire::LAUNCH_STATUS, receipt, 0, 0, 0, 0, 0, 0]);
            let (running, message) = policy::observation(reply);
            if !running {
                launched = None;
                status = message;
                repaint = true;
            }
        }
        let snapshot = call(parent, [wire::POLL, 0, 0, 0, 0, 0, 0, 0]);
        let extent = wire::poll_view(snapshot);
        let event = wire::poll_event(snapshot);
        let clicked = pointers.update(event).is_some_and(|b| b.left_pressed());
        if extent[0] == 0 {
            #[cfg(feature = "picker-startup-fault")]
            if user_picker {
                panic!("deliberate picker startup failure");
            }
            #[cfg(feature = "launcher-startup-fault")]
            if !user_picker {
                panic!("deliberate launcher startup failure");
            }
            #[cfg(feature = "recovery-probes")]
            if !user_picker {
                management_probe.event(parent, event);
            }
            #[cfg(feature = "recovery-probes")]
            if !user_picker && event[2] == wire::KEY && event[4] == input::PRESS as u64 {
                match event[3] as u8 {
                    input::F7 => return 0,
                    input::F8 => panic!("deliberate launcher failure"),
                    _ => {}
                }
            }
            let mut choose = false;
            if management.open() {
                repaint |= management.event(event, extent, clicked, parent);
            } else if let Some(edit) = &mut editor {
                let (changed, closed) =
                    edit.event(event, extent, clicked, &mut realm.scope, parent);
                repaint |= changed;
                if closed {
                    editor = None;
                    status = "";
                }
                if changed {
                    // Saving may replace our geometry epoch. Poll before painting,
                    // instead of drawing into the old coordinate space.
                    continue;
                }
            } else if !user_picker
                && let Some(message) = tasks.click(
                    parent,
                    extent,
                    event,
                    clicked,
                    layout::tasks_left(realm.count),
                )
            {
                status = message;
                tasks.refresh(parent, true);
                repaint = true;
            } else {
                let (changed, activate) = selection.event(
                    event,
                    extent,
                    clicked,
                    realm.count,
                    user_picker,
                    management.available(),
                    saves == wire::SAVES_OFFLINE,
                );
                repaint |= changed;
                choose = activate;
            }
            if choose {
                let action = policy::action(
                    user_picker,
                    realm.count,
                    selection.index,
                    &realm.ids,
                    saves == wire::SAVES_OFFLINE,
                );
                match action {
                    Action::Retry => {
                        let reply = call(parent, [wire::RETRY_SAVES, 0, 0, 0, 0, 0, 0, 0]);
                        status = policy::outcome(Operation::Retry, reply).0;
                    }
                    Action::Manage => management.begin(),
                    Action::Settings => match settings::Editor::open(&mut realm.scope) {
                        Ok(edit) => {
                            editor = Some(edit);
                            status = "";
                        }
                        Err(_) => status = "CANNOT OPEN",
                    },
                    other => status = act(&mut realm, parent, other, &mut launched),
                }
                if status.is_empty() && selection.index == NEW {
                    selection.index = realm.count.saturating_sub(1);
                }
                if !user_picker {
                    tasks.refresh(parent, true);
                }
                repaint = true;
            }
            let painted = (|| -> Result<(), u64> {
                if scene.as_ref().is_none_or(|s| s.epoch != extent[1]) {
                    scene = Some(view::Scene::new(parent, extent[1], user_picker)?);
                    repaint = true;
                }
                if repaint {
                    if management.open() {
                        scene
                            .as_ref()
                            .unwrap()
                            .management(parent, extent, &management)?;
                    } else if let Some(edit) = &editor {
                        scene.as_ref().unwrap().settings(parent, extent, edit)?;
                    } else {
                        scene.as_ref().unwrap().paint(
                            parent,
                            extent,
                            &realm,
                            selection.index,
                            match saves {
                                wire::SAVES_OFFLINE => "RETRY SAVES",
                                wire::SAVES_RESTARTING => "STARTING SAVES",
                                _ => status,
                            },
                            management.available(),
                        )?;
                        if !user_picker {
                            scene
                                .as_ref()
                                .unwrap()
                                .tasks(parent, extent, &tasks, &realm)?;
                        }
                    }
                    repaint = false;
                }
                Ok(())
            })();
            if let Err(status) = painted {
                let now = call(parent, [wire::VIEW, 0, 0, 0, 0, 0, 0, 0]);
                assert!(
                    now[0] != 0 || now[1] != extent[1],
                    "chooser drawing refused: {status}"
                );
                scene = None;
                pointers.reset();
                repaint = true;
                continue;
            }
        } else {
            scene = None;
        }
        if snapshot[5] & wire::MORE != 0 {
            continue;
        }
        match control.next(wake.interest(), Some(time::after(100).unwrap())) {
            Ok(None) => {
                wake.take().unwrap();
            }
            Ok(Some((bytes, len))) => {
                // Settings drafts participate in ordinary close/loss reporting.
                let reply = match wire::decode(&bytes[..len]) {
                    Some([wire::HEALTH, 0, 0, 0, 0, 0, 0, 0]) => [0; 8],
                    Some([wire::LOSS, 0, 0, 0, 0, 0, 0, 0]) => [
                        0,
                        u64::from(editor.as_ref().is_some_and(settings::Editor::dirty)),
                        0,
                        0,
                        0,
                        0,
                        0,
                        0,
                    ],
                    _ => wire::error(abi::DENIED),
                };
                control.reply(&wire::encode(reply)).unwrap();
            }
            Err(Error(code)) if code == abi::TIMED_OUT as i64 => (),
            Err(error) => panic!("chooser wait {error:?}"),
        }
    }
}
/// Execute an Omega intent only through this child's already supplied services.
fn act(
    realm: &mut realm::Realm,
    parent: &mut Services,
    action: Action,
    launched: &mut Option<u64>,
) -> &'static str {
    match action {
        Action::Copy(template) => {
            let status = realm.copy_template(template).err().unwrap_or(0);
            policy::outcome(Operation::Copy, [status, 0, 0, 0, 0, 0, 0, 0]).0
        }
        Action::Launch(object) => {
            let reply = call(parent, [wire::LAUNCH, object, 0, 0, 0, 0, 0, 0]);
            let (message, receipt) = policy::outcome(Operation::Launch, reply);
            if let Some(receipt) = receipt {
                *launched = Some(receipt);
            }
            message
        }
        _ => "",
    }
}
fn call(services: &mut Services, words: wire::Packet) -> wire::Packet {
    services
        .request_interaction(words, None)
        .unwrap_or_else(|error| panic!("chooser request {words:?}: {error:?}"))
}
