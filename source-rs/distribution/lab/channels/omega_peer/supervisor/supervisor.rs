// A Rust owner drives the standalone Omega supervisor through its kernel-installed
// control pair; Rust peers are the children it admits from lent launches.
// Owner words: 1 admit [slot, granted, launch, argument], 2 stop [slot], 3 status,
// 99 exit. Status: slot liveness, echoes handled, storage messages, exit kinds, held requests.
use cathedral_kernel_contracts::user as abi;
use cathedral_platform_contracts::{imports, matrix, services};
use cathedral_user_runtime::{
    Error,
    connection::Client,
    ipc, notification, resources,
    services::{Services, Wake},
    task::{Environment, Launch, Outcome, Port},
    time,
};

const RETURNED: u64 = 1;
const CANCELLED: u64 = 3;
// Child programs, selected by the admission argument.
const ECHO: u64 = 10;
const STORAGE: u64 = 11;
const IDLE: u64 = 12;
const HOLD_EXIT: u64 = 13;
const HOLD_IDLE: u64 = 14;
const REFUSALS: u64 = 15;
const NO_STORAGE: u64 = 16;

pub(super) fn run() {
    for scenario in 1..=6 {
        let budget = resources::budget().unwrap();
        let physical = resources::physical_budget().unwrap();
        let bells = notification::budget().unwrap();
        run_scenario(scenario);
        assert_eq!(
            resources::budget().unwrap(),
            budget,
            "admission scenario {scenario}"
        );
        assert_eq!(resources::physical_budget().unwrap(), physical);
        assert_eq!(notification::budget().unwrap(), bells);
    }
    cathedral_user_runtime::write(b"Omega child admission PASS\n").unwrap();
}

struct Owner {
    client: Client,
}
impl Owner {
    fn call(&mut self, words: [u64; 8]) -> [u64; 8] {
        let (reply, n) = self
            .client
            .call(&matrix::encode(words), time::after(300).unwrap())
            .unwrap();
        matrix::decode(&reply[..n]).unwrap()
    }
    fn admit(&mut self, slot: u64, granted: u64, launch: u64, argument: u64) -> u64 {
        self.call([1, slot, granted, launch, argument, 0, 0, 0])[0]
    }
    fn stop(&mut self, slot: u64) -> [u64; 8] {
        self.call([2, slot, 0, 0, 0, 0, 0, 0])
    }
    fn status(&mut self) -> [u64; 8] {
        self.call([3, 0, 0, 0, 0, 0, 0, 0])
    }
    fn observed(&mut self, check: impl Fn([u64; 8]) -> bool) -> [u64; 8] {
        let deadline = time::after(600).unwrap();
        loop {
            let state = self.status();
            if check(state) {
                return state;
            }
            assert!(
                !time::reached(time::now().unwrap(), deadline),
                "supervisor observation expired: {state:?}"
            );
            let _ = cathedral_user_runtime::event::wait(&[], time::after(1).unwrap());
        }
    }
}

fn run_scenario(number: u64) {
    // Loans only flow to lower launch slots: the supervisor is launch 0, peers 1 and 2.
    let loans = [1, 2].map(|index| Launch::at(index).unwrap());
    let supervisor = Launch::at(0)
        .unwrap()
        .spawn_tree(300, &[], &loans)
        .unwrap();
    // Lent launches are unusable here until the supervisor's subtree is reclaimed.
    assert!(loans[0].spawn(0).is_err());
    let pair = Port::at(0).unwrap().connect().unwrap();
    let mut owner = Owner {
        client: Client::connect(pair, time::after(300).unwrap()).unwrap(),
    };
    match number {
        1 => {
            // Echo children exit by themselves. The slot must be reclaimed and
            // reusable under a channel reservation that a per-child leak would exhaust.
            for round in 1..=6u64 {
                assert_eq!(owner.admit(0, imports::INTERACTION, 0, ECHO), 0);
                let state = owner.observed(|v| v[0] == 0 && v[4] == RETURNED);
                assert_eq!(state[2], round * 8, "echoes after round {round}");
            }
        }
        2 => {
            // Storage endpoints move to the child; its copied messages arrive here.
            assert_eq!(owner.admit(1, imports::ALL, 1, STORAGE), 0);
            let state = owner.observed(|v| v[1] == 0 && v[5] == RETURNED);
            assert_eq!(state[3], 3, "storage messages");
            assert_eq!(state[2], 4);
        }
        3 => {
            assert_eq!(owner.admit(0, imports::INTERACTION, 0, IDLE), 0);
            for _ in 0..4 {
                assert_eq!(owner.status()[0], 1);
            }
            assert_eq!(owner.stop(0)[..2], [0, CANCELLED]);
            assert_eq!(owner.status()[0], 0);
            // Cancelling the supervisor with a live child reclaims the whole subtree.
            assert_eq!(owner.admit(1, imports::INTERACTION, 1, IDLE), 0);
            owner.observed(|v| v[1] == 1);
            supervisor.cancel().unwrap();
            assert_eq!(supervisor.wait().unwrap(), Outcome::Cancelled);
            owner.client.retire();
            return;
        }
        4 => {
            // A child dies with its request held; another holds while the owner
            // keeps getting answers, then is stopped.
            assert_eq!(owner.admit(0, imports::INTERACTION, 0, HOLD_EXIT), 0);
            owner.observed(|v| v[0] == 0 && v[4] == RETURNED);
            assert_eq!(owner.admit(1, imports::INTERACTION, 1, HOLD_IDLE), 0);
            owner.observed(|v| v[7] == 1);
            for _ in 0..6 {
                let state = owner.status();
                assert_eq!(state[1], 1);
                assert_eq!(state[7], 1);
            }
            assert_eq!(owner.stop(1)[..2], [0, CANCELLED]);
        }
        5 => {
            // Refused admissions leave nothing behind; later admissions still work.
            assert_eq!(owner.admit(0, 0, 0, ECHO), abi::INVALID_ARGUMENT);
            assert_eq!(
                owner.admit(0, imports::INTERACTION | imports::CAPTURE, 0, ECHO),
                abi::INVALID_ARGUMENT
            );
            assert_eq!(owner.admit(0, imports::INTERACTION, 5, ECHO), abi::BAD_HANDLE);
            assert_eq!(owner.admit(2, imports::INTERACTION, 0, ECHO), abi::INVALID_ARGUMENT);
            assert_eq!(owner.admit(0, imports::INTERACTION, 0, IDLE), 0);
            assert_eq!(owner.admit(0, imports::INTERACTION, 1, IDLE), abi::BUSY);
            // Launch 0 is in use by slot 0: the kernel refuses and slot 1 stays empty.
            assert_ne!(owner.admit(1, imports::INTERACTION, 0, IDLE), 0);
            assert_eq!(owner.status()[1], 0);
            assert_eq!(owner.admit(1, imports::ALL, 1, STORAGE), 0);
            owner.observed(|v| v[1] == 0 && v[5] == RETURNED);
            assert_eq!(owner.stop(0)[..2], [0, CANCELLED]);
            assert_eq!(owner.stop(0)[0], abi::PEER_CLOSED);
        }
        6 => {
            assert_eq!(owner.admit(0, imports::ALL, 0, REFUSALS), 0);
            owner.observed(|v| v[0] == 0 && v[4] == RETURNED);
            assert_eq!(owner.admit(1, imports::INTERACTION, 1, NO_STORAGE), 0);
            owner.observed(|v| v[1] == 0 && v[5] == RETURNED);
        }
        _ => unreachable!(),
    }
    assert_eq!(owner.call([99, 0, 0, 0, 0, 0, 0, 0])[0], 0);
    assert_eq!(
        supervisor.wait_until(time::after(600).unwrap()).unwrap(),
        Outcome::Returned(0)
    );
    owner.client.retire();
}

/// A child admitted by the Omega supervisor: brokered services only, no launches.
pub(super) fn child(mode: u64, environment: Environment) -> u64 {
    assert!(Launch::at(0).is_err());
    assert_eq!(environment.len(), 2);
    let mut bindings = Services::load(&environment).unwrap();
    assert!(bindings.is_brokered());
    bindings.enable_shared().unwrap();
    let interaction = bindings.interaction(1, 1).unwrap();
    let Wake::Notification(wait) = interaction.wake else {
        panic!("brokered interaction must supply a notification");
    };
    assert_eq!(wait.signal(false), Err(Error(abi::DENIED as i64)));
    assert!(!interaction.wake.take().unwrap());
    match mode {
        ECHO => echo(&mut bindings, 8),
        STORAGE => {
            let (send, receive) = bindings.storage(1, 1).unwrap();
            assert_ne!(send.raw(), receive.raw());
            // The copied channel queues one message; wait for the supervisor to drain it.
            for value in 1..=3u8 {
                let deadline = time::after(300).unwrap();
                loop {
                    match send.send(&[value; 8]) {
                        Ok(()) => break,
                        Err(Error(code)) if code == abi::WOULD_BLOCK as i64 => {
                            assert!(!time::reached(time::now().unwrap(), deadline));
                            let _ = cathedral_user_runtime::event::wait(&[], time::after(1).unwrap());
                        }
                        Err(error) => panic!("storage send: {error:?}"),
                    }
                }
            }
            echo(&mut bindings, 4);
        }
        IDLE => idle(),
        HOLD_EXIT | HOLD_IDLE => {
            let mut client = bindings.into_shared_transport().unwrap();
            client
                .begin(&matrix::encode([10, 0, 0, 0, 0, 0, 0, 0]), None)
                .unwrap();
            if mode == HOLD_IDLE {
                idle();
            }
            // Exit with the request outstanding; the supervisor must reclaim it.
        }
        REFUSALS => {
            let taken = [services::TAKEN, 0, 0, 0, 0, 0, 0, 0];
            assert_eq!(
                request(&mut bindings, services::request(imports::INTERACTION, 1, 1, 0)),
                taken
            );
            for (words, code) in [
                (
                    [services::ACQUIRE, services::VERSION, imports::INTERACTION, 2, 2, 0, 0, 0],
                    services::INCOMPATIBLE,
                ),
                (
                    [services::ACQUIRE, 1, imports::INTERACTION, 1, 1, 0, 0, 0],
                    services::INCOMPATIBLE,
                ),
                (
                    [services::ACQUIRE, services::VERSION, imports::INTERACTION, 0, 1, 0, 0, 0],
                    services::INVALID,
                ),
                (
                    [services::ACQUIRE, services::VERSION, 77, 1, 1, 0, 0, 0],
                    services::UNAVAILABLE,
                ),
                (
                    services::request(imports::INPUT_PRODUCER, 1, 1, 0),
                    services::UNAVAILABLE,
                ),
                (
                    [services::ACQUIRE, services::VERSION, imports::STORAGE, 1, 1, 2, 0, 0],
                    services::INVALID,
                ),
                (
                    [services::ACQUIRE, services::VERSION, imports::STORAGE, 1, 1, 0, 9, 0],
                    services::INVALID,
                ),
            ] {
                assert_eq!(request(&mut bindings, words), [code, 0, 0, 0, 0, 0, 0, 0]);
            }
            let (send, receive) = bindings.storage(1, 1).unwrap();
            assert_eq!(
                request(&mut bindings, services::request(imports::STORAGE, 1, 1, 0)),
                taken
            );
            assert_eq!(
                request(&mut bindings, services::request(imports::STORAGE, 1, 1, 1)),
                taken
            );
            send.close().unwrap();
            receive.close().unwrap();
        }
        NO_STORAGE => {
            assert!(
                matches!(bindings.storage(1, 1), Err(Error(code)) if code == services::UNAVAILABLE as i64)
            );
            echo(&mut bindings, 2);
        }
        _ => panic!("unknown admission child {mode}"),
    }
    0
}
fn request(bindings: &mut Services, words: [u64; 8]) -> [u64; 8] {
    bindings.request_interaction(words, None).unwrap()
}
fn echo(bindings: &mut Services, count: u64) {
    for index in 1..=count {
        let words = [20, index, index * 3, 0, 0, 0, 0, 0xabc];
        assert_eq!(bindings.request_interaction(words, None).unwrap(), words);
    }
}
fn idle() -> ! {
    let (_, idle) = ipc::channel().unwrap();
    let _ = idle.receive(&mut [0; 64]);
    unreachable!("idle child resumed without cancellation")
}
