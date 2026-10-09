//! One independently restartable provider and its accepted connection.
use cathedral_user_runtime::{
    Error,
    ipc::Handle,
    task::{Child, Launch, Port},
};

pub struct Service {
    launch: Launch,
    port: Port,
    child: Child,
    generation: u64,
    pub send: Handle,
    pub receive: Handle,
}
impl Service {
    pub fn start(index: u64) -> Result<Self, Error> {
        let launch = Launch::at(index)?;
        let port = Port::at(index)?;
        let child = launch.spawn(0)?;
        let (send, receive) = port.connect()?;
        Ok(Self {
            launch,
            port,
            child,
            generation: 0,
            send,
            receive,
        })
    }
    pub fn restart(&mut self) -> Result<(), Error> {
        self.child.cancel()?;
        self.child.wait()?;
        self.generation += 1;
        self.child = self.launch.spawn(self.generation)?;
        (self.send, self.receive) = self.port.connect()?;
        Ok(())
    }
}
