//! An ordinary peer can relay packets only through its explicitly supplied link.
use super::*;
use cathedral_user_runtime::{link::Link, task::Launch};
pub fn run(environment: cathedral_user_runtime::task::Environment, mode: u64) -> u64 {
    assert!(Launch::at(0).is_err());
    assert!(Link::at(0).is_err());
    assert!(cathedral_user_runtime::display::mapping().is_err());
    assert!(cathedral_user_runtime::keyboard::read(false).is_err());
    use cathedral_platform_contracts::services;
    let mut bindings = cathedral_user_runtime::services::Services::load(&environment).unwrap();
    assert_eq!(environment.len(), 2);
    assert!(bindings.is_brokered());
    #[cfg(feature = "omega-matrix")]
    bindings.enable_shared().unwrap();
    let before = cathedral_user_runtime::resources::budget().unwrap();
    let notification_before = cathedral_user_runtime::notification::budget().unwrap();
    let parent = (
        environment.endpoint(0).unwrap(),
        environment.endpoint(1).unwrap(),
    );
    if mode != 4 {
        let interaction = bindings.interaction(1, 1).unwrap();
        let cathedral_user_runtime::services::Wake::Notification(wait) = interaction.wake else {
            panic!("brokered interaction must supply a notification");
        };
        assert_eq!(
            wait.signal(false),
            Err(cathedral_user_runtime::Error(
                cathedral_kernel_contracts::user::DENIED as i64
            ))
        );
        assert_eq!(
            wait.close_object(),
            Err(cathedral_user_runtime::Error(
                cathedral_kernel_contracts::user::DENIED as i64
            ))
        );
        assert!(!interaction.wake.take().unwrap());
        assert_eq!(interaction.pair.0.raw(), parent.0.raw());
        if mode == 3 {
            // Die later with one storage endpoint received and its partner still
            // held by the broker. No fake filesystem implementation is involved.
            let endpoint = bindings
                .acquire_endpoint_part(cathedral_platform_contracts::imports::STORAGE, 1, 1, 0)
                .unwrap();
            assert_ne!(endpoint.raw(), 0);
        } else {
            let storage = if mode == 2 {
                Some(bindings.storage(1, 1).unwrap())
            } else {
                None
            };
            for _ in 0..16 {
                assert_eq!(
                    bindings.interaction(1, 1).unwrap().pair.0.raw(),
                    parent.0.raw()
                );
                if let Some(storage) = storage {
                    let again = bindings.storage(1, 1).unwrap();
                    assert_eq!(
                        (again.0.raw(), again.1.raw()),
                        (storage.0.raw(), storage.1.raw())
                    );
                } else {
                    assert!(
                        matches!(bindings.storage(1, 1), Err(cathedral_user_runtime::Error(code)) if code == services::UNAVAILABLE as i64)
                    );
                }
            }
        }
    }
    assert_eq!(cathedral_user_runtime::resources::budget().unwrap(), before);
    let mut expected_notifications = notification_before;
    expected_notifications[3] += u64::from(mode != 4);
    assert_eq!(
        cathedral_user_runtime::notification::budget().unwrap(),
        expected_notifications
    );
    let control = (Handle::bootstrap(0).unwrap(), Handle::bootstrap(1).unwrap());
    #[cfg(feature = "omega-matrix")]
    let mut client = bindings.into_shared_transport().unwrap();
    loop {
        let mut bytes = [0; 64];
        let len = control.0.receive(&mut bytes).unwrap();
        // Relay malformed payloads too, so the real provider checks them.
        #[cfg(feature = "omega-matrix")]
        {
            let (reply, len) = client
                .call(&bytes[..len], time::after(300).unwrap())
                .unwrap();
            control.1.send(&reply[..len]).unwrap();
        }
        #[cfg(not(feature = "omega-matrix"))]
        {
            parent.0.send(&bytes[..len]).unwrap();
            let len = parent
                .1
                .receive_until(&mut bytes, time::after(300).unwrap())
                .unwrap();
            control.1.send(&bytes[..len]).unwrap();
        }
    }
}
