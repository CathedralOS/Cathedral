// Acquire -> dispatch app-selected work -> resume with the result. There is
// no app schema, widget, input mapping, launch policy or rendering decision here.
use crate::{Instance, PACKET_WORDS, STATE_WORDS, Step};
use cathedral_kernel_contracts::user as abi;
use cathedral_platform_contracts::matrix as wire;
use cathedral_user_runtime::{
    Error,
    ipc::{self, Handle},
    memory::Private,
    services::{Interaction, Services},
    storage::Scope,
    time,
};

pub(super) struct Host {
    services: Services,
    pub interaction: Option<Interaction>,
    pub storage: Option<Scope>,
    control: (Handle, Handle),
}
/// # Safety
/// The export must satisfy `Instance::new` for the complete task lifetime.
pub unsafe fn run(step: Step, services: Services) -> Result<u64, Error> {
    // Keep buffers off the bounded task stack. Nine private pages, charged to
    // this app; temporary immutable attachments are separately charged.
    let mut backing = Private::allocate(9)?;
    let (state, packet) = backing.bytes_mut().split_at_mut(STATE_WORDS * 8);
    // SAFETY: Private pages are aligned, exclusively borrowed, and exactly fit
    // these padding-free arrays. Their backing outlives the instance.
    let (state, packet) = unsafe {
        (
            &mut *state.as_mut_ptr().cast::<[u64; STATE_WORDS]>(),
            &mut *packet.as_mut_ptr().cast::<[u64; PACKET_WORDS]>(),
        )
    };
    // SAFETY: Forwarded caller contract; the two disjoint buffers remain live.
    let mut app =
        unsafe { Instance::new(step, state, packet) }.ok_or(Error(abi::INVALID_ARGUMENT as i64))?;
    let mut host = Host {
        services,
        interaction: None,
        storage: None,
        control: (Handle::bootstrap(0)?, Handle::bootstrap(1)?),
    };
    loop {
        let operation = app.advance();
        if operation == 0 {
            return Ok(0);
        }
        app.packet[9] = 0;
        if let Err(error) = host.dispatch(operation, app.packet) {
            app.packet[9] = error.0 as u64;
        }
    }
}
impl Host {
    fn dispatch(&mut self, operation: u64, packet: &mut [u64; PACKET_WORDS]) -> Result<(), Error> {
        match operation {
            1 => {
                self.interaction = Some(self.services.interaction(packet[0], packet[1])?);
            }
            2 => {
                let pair = self.services.storage(packet[0], packet[1])?;
                if self.storage.is_none() {
                    self.storage = Some(Scope::new(pair));
                }
            }
            3 => {
                let scope = self.storage.as_mut().ok_or(Error(abi::DENIED as i64))?;
                scope.bind().map_err(|status| Error(status as i64))?;
                packet[..8].fill(0);
                packet[1] = scope.realm();
            }
            4 | 5 => exchange::exchange(self, operation, packet)?,
            6 => self.wait(packet)?,
            7 => self
                .control
                .1
                .send(&wire::encode(packet[..8].try_into().unwrap()))?,
            8 => cathedral_user_runtime::write(exchange::payload(packet)?)?,
            _ => return Err(Error(abi::INVALID_ARGUMENT as i64)),
        }
        Ok(())
    }
    fn wait(&self, packet: &mut [u64; PACKET_WORDS]) -> Result<(), Error> {
        let wake = self
            .interaction
            .as_ref()
            .ok_or(Error(abi::DENIED as i64))?
            .wake;
        loop {
            // A continuous wake stream must not starve owner lifecycle questions.
            let ready = match ipc::wait_many(&[self.control.0, wake], time::after(100)?) {
                Err(Error(code)) if code == abi::TIMED_OUT as i64 => continue,
                result => result?,
            };
            let mut bytes = [0; 64];
            let len = [self.control.0, wake][ready].receive(&mut bytes)?;
            packet[..8].fill(0);
            if ready == 0 {
                packet[..8].copy_from_slice(
                    &wire::decode(&bytes[..len]).ok_or(Error(abi::IO_ERROR as i64))?,
                );
            }
            packet[9] = u64::from(ready == 0);
            packet[10] = time::cpu_us().unwrap_or(0);
            return Ok(());
        }
    }
}
