//! Ownership during partial admission. Track only endpoints still held here;
//! successful spawn moves remove them before any later allocation can reuse names.
use super::*;
pub(super) struct Attempt {
    tasks: [Option<task::Child>; 3],
    pub display_control: Option<display::Control>,
    handles: [Option<Handle>; 10],
}
impl Attempt {
    pub fn new() -> Self {
        Self {
            tasks: [None; 3],
            display_control: None,
            handles: [None; 10],
        }
    }
    pub fn track(&mut self, pair: Pair) -> Pair {
        for handle in [pair.0, pair.1] {
            *self
                .handles
                .iter_mut()
                .find(|entry| entry.is_none())
                .expect("admission endpoints") = Some(handle);
        }
        pair
    }
    pub fn channel(&mut self) -> Result<Pair, Error> {
        Ok(self.track(ipc::channel()?))
    }
    pub fn spawn(&mut self, stage: Stage, argument: u64, moved: &[Handle]) -> Result<(), Error> {
        #[cfg(feature = "bootstrap-probes")]
        probes::refusal(argument)?;
        let child = Launch::at(stage as u64)?.spawn_with(argument, moved)?;
        self.tasks[stage as usize] = Some(child);
        for handle in moved {
            let entry = self
                .handles
                .iter_mut()
                .find(|entry| entry.is_some_and(|h| h.raw() == handle.raw()))
                .expect("owned endpoint move");
            *entry = None;
        }
        Ok(())
    }
    pub fn retire(self) {
        for index in [1, 2, 0] {
            if let Some(child) = self.tasks[index] {
                retire_task(child);
            }
        }
        for handle in self.handles.into_iter().flatten() {
            handle.close().expect("admission endpoint cleanup");
        }
    }
    pub fn commit(self, matrix: RootControl, source: Pair, extent: [u64; 2]) -> Session {
        // Root presentation has its own ready connection now. Release early
        // configuration before starting storage. Its remote reader/signal can
        // outlive this local drop, so each composition funds that overlap explicitly.
        drop(self.display_control);
        let root = self.tasks[2].unwrap();
        crate::ROOT_TASK.store(root.raw(), core::sync::atomic::Ordering::Relaxed);
        Session {
            #[cfg(all(feature = "database-probes", not(feature = "shell")))]
            database_fresh: false,
            matrix,
            source: Some(source),
            extent,
            #[cfg(feature = "shell")]
            user: 0,
            #[cfg(feature = "shell")]
            login: shell::Login::Stopped,
            #[cfg(not(feature = "shell"))]
            pressure: task::Executable::at(2).is_ok(),
            storage: None,
            storage_watch: persistence::Watchdog::default(),
            display_task: self.tasks[0].unwrap(),
            root,
            input: None,
        }
    }
}
pub(super) fn retire_task(child: task::Child) {
    let outcome = match child.wait_until(time::now().unwrap()) {
        Ok(outcome) => outcome,
        Err(Error(code)) if code == abi::BAD_HANDLE as i64 => return,
        Err(Error(code)) if code == abi::TIMED_OUT as i64 => {
            child.cancel().unwrap();
            child.wait().unwrap()
        }
        Err(error) => panic!("bootstrap collection: {error:?}"),
    };
    match outcome {
        task::Outcome::Fault {
            vector,
            error,
            address,
            instruction,
        } => {
            // Collection consumes the outcome. Preserve the fault before rollback
            // erases the only explanation for a closed bootstrap connection.
            time::storage_sample(
                "bootstrap-child-fault",
                &[child.raw(), vector, error, address, instruction],
            );
        }
        task::Outcome::Returned(code) => {
            time::storage_sample("bootstrap-child-returned", &[child.raw(), code]);
        }
        task::Outcome::Cancelled => (),
    }
}
