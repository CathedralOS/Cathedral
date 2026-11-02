//! Two separately launched Omega peers exercise one bounded request window.
use super::*;
pub fn run() -> u64 {
    for (mode, scenario) in [
        (400, "held first reply and full mailbox"),
        (410, "full typed-grant slot"),
        (402, "disconnect with two requests"),
        (404, "unknown reply ID"),
        (406, "duplicate completed reply ID"),
        (408, "attachment without a frame"),
        (400, "healthy reuse after faults"),
    ] {
        let budget = cathedral_user_runtime::resources::budget().unwrap();
        let frames = cathedral_user_runtime::resources::physical_budget().unwrap();
        let bells = cathedral_user_runtime::notification::budget().unwrap();
        let (send, input) = ipc::channel().unwrap();
        let (output, receive) = ipc::channel().unwrap();
        let (release, barrier) = ipc::channel().unwrap();
        let provider = Launch::at(0)
            .unwrap()
            .spawn_with(mode + 1, &[input, output, barrier])
            .unwrap();
        let client = Launch::at(1)
            .unwrap()
            .spawn_with(mode, &[send, receive, release])
            .unwrap();
        let client_outcome = client.wait_until(time::after(2000).unwrap()).unwrap();
        let provider_outcome = provider.wait_until(time::after(300).unwrap()).unwrap();
        assert_eq!(
            client_outcome,
            Outcome::Returned(0),
            "{scenario}: Omega client; provider {provider_outcome:?}"
        );
        assert_eq!(
            provider_outcome,
            Outcome::Returned(0),
            "{scenario}: Omega deferred provider"
        );
        assert_eq!(cathedral_user_runtime::resources::budget().unwrap(), budget);
        assert_eq!(
            cathedral_user_runtime::resources::physical_budget().unwrap(),
            frames
        );
        assert_eq!(
            cathedral_user_runtime::notification::budget().unwrap(),
            bells
        );
    }
    cathedral_user_runtime::write(b"Omega overlapping requests PASS\n").unwrap();
    0
}
