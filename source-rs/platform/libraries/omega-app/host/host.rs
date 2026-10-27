// Acquire -> dispatch app-selected work -> resume with the result. There is
// no app schema, widget, input mapping, launch policy or rendering decision here.
use crate::{Instance, PACKET_WORDS, STATE_WORDS, Step};
use cathedral_kernel_contracts::user as abi;
use cathedral_platform_contracts::matrix as wire;
use cathedral_user_runtime::{
    Error,
    memory::Private,
    server::{OwnerControl, SharedSlot},
    services::{Interaction, Services},
    storage::Scope,
    time,
};

pub(super) struct Host {
    services: Services,
    pub interaction: Option<Interaction>,
    pub storage: Option<Scope<'static>>,
    control: OwnerControl,
    staged: Option<attachments::Staged>,
}
/// # Safety
/// The export must satisfy `Instance::new` for the complete task lifetime.
pub unsafe fn run(step: Step, services: Services) -> Result<u64, Error> {
    // SAFETY: Same export lifetime and layout contract as this entry.
    unsafe { run_with_argument(step, services, 0) }
}
/// The ordinary task argument is startup data, never authority. It occupies
/// packet word 12 on the first step; subsequent operations may reuse that word.
/// # Safety
/// The export must satisfy `Instance::new` for the complete task lifetime.
pub unsafe fn run_with_argument(
    step: Step,
    mut services: Services,
    argument: u64,
) -> Result<u64, Error> {
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
    app.packet[12] = argument;
    static mut CONTROL: SharedSlot = SharedSlot::empty();
    let control = &raw mut CONTROL;
    // SAFETY: The single app entry owns this slot throughout its lifetime.
    let mut control = OwnerControl::open(unsafe { &mut *control })?;
    // Admission initiates this independent connection. Finish it before app
    // storage/rendering can consume its setup deadline in a deeply nested realm.
    control.establish(time::after(300)?)?;
    cathedral_user_runtime::write(b"Cathedral Omega host: shared owner control ready\n")?;
    services.enable_shared()?;
    cathedral_user_runtime::write(b"Cathedral Omega host: shared interaction broker ready\n")?;
    let mut host = Host {
        services,
        interaction: None,
        storage: None,
        control,
        staged: None,
    };
    loop {
        let operation = app.advance();
        if operation == 0 {
            return Ok(0);
        }
        if operation == 12 {
            return Err(Error(app.packet[0] as i64));
        }
        app.packet[9] = 0;
        if let Err(error) = host.dispatch(operation, app.packet) {
            #[cfg(feature = "trace-storage")]
            time::storage_sample("omega-host-error", &[operation, error.0 as u64]);
            app.packet[9] = error.0 as u64;
        }
    }
}
impl Host {
    fn dispatch(&mut self, operation: u64, packet: &mut [u64; PACKET_WORDS]) -> Result<(), Error> {
        match operation {
            1 => {
                let interaction = self.services.interaction(packet[0], packet[1])?;
                if self.interaction.is_none()
                    && matches!(
                        interaction.wake,
                        cathedral_user_runtime::services::Wake::Notification(_)
                    )
                {
                    cathedral_user_runtime::write(
                        b"Cathedral Omega host: notification input wake ready\n",
                    )?;
                }
                self.interaction = Some(interaction);
            }
            2 => {
                let pair = self.services.storage(packet[0], packet[1])?;
                if self.storage.is_none() {
                    self.storage = Some(Scope::connect(pair)?);
                    cathedral_user_runtime::write(b"Cathedral Omega host: shared storage ready\n")?;
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
                .reply(&wire::encode(packet[..8].try_into().unwrap()))?,
            8 => cathedral_user_runtime::write(exchange::payload(packet)?)?,
            9 => attachments::append(&mut self.staged, packet)?,
            10 => self.staged = None,
            11 => {
                packet[..8].fill(0);
                packet[0] = time::now()?;
            }
            _ => return Err(Error(abi::INVALID_ARGUMENT as i64)),
        }
        Ok(())
    }
    fn wait(&mut self, packet: &mut [u64; PACKET_WORDS]) -> Result<(), Error> {
        let wake = self
            .interaction
            .as_ref()
            .ok_or(Error(abi::DENIED as i64))?
            .wake;
        // Optional caller-selected refresh deadline. Zero preserves an indefinite
        // event wait; a timeout is an empty wake, never an invented input event.
        let refresh = packet[13];
        packet[13] = 0;
        let deadline = if refresh == 0 {
            None
        } else {
            Some(time::after(refresh)?)
        };
        loop {
            let words = match self.control.next(wake.interest(), deadline)? {
                Some((bytes, len)) => match wire::decode(&bytes[..len]) {
                    Some(words) => Some(words),
                    None => {
                        self.control
                            .reply(&wire::encode(wire::error(abi::INVALID_ARGUMENT)))?;
                        continue;
                    }
                },
                None if wake.take()?
                    || deadline
                        .is_some_and(|d| time::now().is_ok_and(|now| time::reached(now, d))) =>
                {
                    None
                }
                None => continue,
            };
            packet[..8].copy_from_slice(&words.unwrap_or([0; 8]));
            packet[9] = u64::from(words.is_some());
            packet[10] = time::cpu_us().unwrap_or(0);
            return Ok(());
        }
    }
}
