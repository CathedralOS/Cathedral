//! Select local launch grants, create the child's parent connection, admit, install.
use super::*;
use cathedral_user_runtime::{
    ipc,
    task::{Launch, Port},
};
impl Children {
    pub(super) fn start(
        &mut self,
        slot: usize,
        launch: u64,
        mask: u64,
        argument: u64,
        server: &mut Server,
        world: &mut World,
    ) -> Result<(), Error> {
        if self.slots[slot].is_some() || !server.available(slot) {
            return Err(Error(abi::BUSY as i64));
        }
        if mask >> abi::MAX_LAUNCHES != 0 {
            return Err(Error(abi::DENIED as i64));
        }
        let target = Launch::at(launch)?;
        let port = Port::at(launch)?;
        let mut loans = [Launch::from_raw(0); abi::MAX_LAUNCHES];
        let mut count = 0;
        for index in 0..abi::MAX_LAUNCHES {
            if mask & (1 << index) != 0 {
                loans[count] = Launch::at(index as u64)?;
                count += 1;
            }
        }
        let (requests, input) = ipc::channel()?;
        let (output, replies) = match ipc::channel() {
            Ok(pair) => pair,
            Err(error) => {
                requests.close()?;
                input.close()?;
                return Err(error);
            }
        };
        let child = match target.spawn_tree(argument, &[requests, replies], &loans[..count]) {
            Ok(child) => child,
            Err(error) => {
                for endpoint in [requests, input, output, replies] {
                    endpoint.close()?;
                }
                return Err(error);
            }
        };
        let control = match port.connect() {
            Ok(pair) => pair,
            Err(error) => {
                child.cancel()?;
                child.wait()?;
                input.close()?;
                output.close()?;
                return Err(error);
            }
        };
        world.disconnected(slot);
        server
            .install(slot, (input, output))
            .expect("validated local client slot changed");
        self.slots[slot] = Some(Owned {
            task: child,
            control,
            data: (input, output),
        });
        Ok(())
    }
}
