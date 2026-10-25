// Prove a real parent -> Omega matrix -> peer tree, then reclaim the whole tree.
use cathedral_user_runtime::{
    ipc, resources,
    task::{Launch, Port},
};
pub fn run() -> u64 {
    let before = resources::budget().unwrap();
    for _ in 0..2 {
        let loans = [1, 2, 3, 4].map(|i| Launch::at(i).unwrap());
        let parent = Launch::at(0).unwrap().spawn_tree(0, &[], &loans).unwrap();
        let control = Port::at(0).unwrap().connect().unwrap();
        assert!(loans[0].spawn(0).is_err());
        let outcome = parent.wait().unwrap();
        assert_eq!(outcome, cathedral_user_runtime::task::Outcome::Returned(0));
        control.0.close().unwrap();
        control.1.close().unwrap();
        assert_eq!(resources::budget().unwrap(), before);
    }
    cathedral_user_runtime::write(b"Cathedral Omega upstream: nested attenuation, revocation and subtree reclamation passed\n").unwrap();
    let (_, idle) = ipc::channel().unwrap();
    let _ = idle.receive(&mut [0; 64]);
    0
}
