//! One independently restartable provider and its accepted connection.
use cathedral_user_runtime::{
    Error,
    ipc::Handle,
    task::{Child, Launch, Port},
    time, write,
};

pub struct Service {
    launch: Launch,
    port: Port,
    child: Child,
    generation: u64,
    index: u64,
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
            index,
            send,
            receive,
        })
    }
    #[cfg(feature = "recovery-lab")]
    pub fn ticket(&self) -> u64 {
        self.child.raw()
    }
    pub fn generation(&self) -> u64 {
        self.generation
    }
    pub fn drawing(&self) -> Result<cathedral_boot_scene::Client, Error> {
        Ok(cathedral_boot_scene::Client::until(
            self.send,
            self.receive,
            time::after(100)?,
        ))
    }
    pub fn receive(&self) -> Result<([u8; 64], usize), Error> {
        let mut bytes = [0; 64];
        let length = self.receive.receive_until(&mut bytes, time::after(100)?)?;
        Ok((bytes, length))
    }
    pub fn recovered(&self) -> Result<(), Error> {
        write(if self.index == 0 {
            b"Cathedral: display recovered\n"
        } else {
            b"Cathedral: input recovered\n"
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
