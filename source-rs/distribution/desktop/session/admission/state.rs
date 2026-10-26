//! Admitted tasks and their teardown order: input, root subtree, display, storage.
use super::*;
pub(crate) struct Session {
    #[cfg(all(feature = "database-probes", not(feature = "shell")))]
    pub(crate) database_fresh: bool,
    pub matrix: Pair,
    /// The realm of the user matrix in the workspace slot, or zero.
    #[cfg(feature = "shell")]
    pub(crate) user: u64,
    #[cfg(feature = "shell")]
    pub(crate) login: shell::Login,
    /// Lab compositions add a capped spinner; the shell never does.
    #[cfg(not(feature = "shell"))]
    pub pressure: bool,
    pub extent: [u64; 2],
    pub(crate) source: Pair,
    pub(crate) storage: Option<persistence::Provider>,
    pub(crate) storage_watch: persistence::Watchdog,
    /// Held so the owner keeps its children's handles; lab compositions also
    /// cancel and collect them when rebuilding the tree.
    pub(crate) display_task: task::Child,
    pub(crate) root: task::Child,
    pub(crate) input: Option<task::Child>,
}
impl Session {
    #[cfg(not(feature = "shell"))]
    pub(crate) fn provider(&self) -> &persistence::Provider {
        self.storage.as_ref().expect("lab requires ready storage")
    }
    pub(crate) fn stop(mut self) {
        for child in [self.input, Some(self.root), Some(self.display_task)]
            .into_iter()
            .flatten()
        {
            attempt::retire_task(child);
        }
        if let Some(provider) = self.storage.take() {
            provider.retire();
        }
        if self.input.is_none() {
            self.source.0.close().unwrap();
            self.source.1.close().unwrap();
        }
        self.matrix.0.close().unwrap();
        self.matrix.1.close().unwrap();
        // Descendant creator death returns dynamic reservations to this owner.
        let mut index = 0;
        while let Ok(launch) = Launch::at(index) {
            match launch.release() {
                Ok(()) => (),
                Err(Error(code)) if code == abi::DENIED as i64 || code == abi::BUSY as i64 => {
                    index += 1
                }
                Err(error) => panic!("desktop reservation release {error:?}"),
            }
        }
    }
}
