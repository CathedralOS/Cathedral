//! One-shot fixture gates carry no data. Publish a signal before handing off its
//! wait right, so a sender's immediate exit cannot strand an accepted waiter.
//! Endpoint escrow installs authority only; no application payload is copied.
use cathedral_user_runtime::{
    ipc::Handle,
    notification::{Notification, Outcome},
};

pub fn post(endpoint: Handle) {
    let [wait, signal, control] = Notification::create().unwrap();
    signal.signal(false).unwrap();
    wait.delegate(endpoint, &[]).unwrap();
    for right in [wait, signal, control] {
        right.release().unwrap();
    }
}

pub fn wait(endpoint: Handle) {
    let (len, right) = Notification::accept(endpoint, &mut [0; 64]).unwrap();
    assert_eq!(len, 0);
    let right = right.expect("gate must install a wait right");
    // No clock authority is needed. The queued grant retains both the signal
    // and its original funding until the receiver consumes and releases it.
    assert_eq!(right.wait(None).unwrap(), Outcome::Signaled);
    right.release().unwrap();
}
