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
    pub fn ticket(&self) -> u64 {
        self.child.raw()
    }
    pub fn generation(&self) -> u64 {
        self.generation
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
    pub fn stopped(&self) -> Result<bool, Error> {
        match self.child.wait_until(time::now()?) {
            Ok(_) => Ok(true),
            Err(Error(code)) if code == cathedral_contracts::user::TIMED_OUT as i64 => Ok(false),
            Err(error) => Err(error),
        }
    }
    pub fn restart(&mut self) -> Result<(), Error> {
        self.child.cancel()?;
        self.child.wait()?;
        self.respawn()
    }
    pub fn respawn(&mut self) -> Result<(), Error> {
        self.generation += 1;
        self.child = self.launch.spawn(self.generation)?;
        (self.send, self.receive) = self.port.connect()?;
        Ok(())
    }
}
