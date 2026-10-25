// Private lab adapter: execute Omega-selected operations using existing mechanisms.
// Policy entrance: source/distribution/lab/matrix-composition/matrix/main.omg.
// World supplies attenuation; runtime admission/server own task and endpoint custody.
use crate::{children::Children, probes::Pending};
use cathedral_kernel_contracts::user as abi;
use cathedral_matrix::world::{Grant, World};
use cathedral_omega_app::{Instance, PACKET_WORDS, STATE_WORDS};
use cathedral_platform_contracts::matrix as wire;
use cathedral_user_runtime::{
    Error,
    memory::Private,
    server::{Request, Server},
};
core::arch::global_asm!(include_str!(env!("OMEGA_MATRIX_ASSEMBLY")));

struct Host {
    world: &'static mut World<3>,
    server: Server,
    children: Children,
    request: Option<Request>,
    pending: Option<Pending>,
    parent: Option<cathedral_user_runtime::services::Services>,
}
pub fn run(environment: cathedral_user_runtime::task::Environment) -> Result<(), Error> {
    static mut WORLD: World<3> = World::empty();
    let address = &raw mut WORLD;
    // SAFETY: One single-threaded entry, with no other reference to this arena.
    let world = unsafe { &mut *address };
    let mut server = Server::open_clients(3)?;
    server.enable_memory();
    let mut host = Host {
        world,
        server,
        children: Children([None, None, None]),
        request: None,
        pending: None,
        parent: match environment.len() {
            0 => None,
            2 => Some(cathedral_user_runtime::services::Services::load(
                &environment,
            )?),
            _ => return Err(Error(abi::INVALID_ARGUMENT as i64)),
        },
    };
    let mut backing = Private::allocate(9)?;
    let (state, packet) = backing.bytes_mut().split_at_mut(STATE_WORDS * 8);
    // SAFETY: The private pages align and exclusively own these disjoint arrays.
    let (state, packet) = unsafe {
        (
            &mut *state.as_mut_ptr().cast::<[u64; STATE_WORDS]>(),
            &mut *packet.as_mut_ptr().cast::<[u64; PACKET_WORDS]>(),
        )
    };
    unsafe extern "sysv64" {
        fn omega_app_step(state: *mut u64, packet: *mut u64) -> u64;
    }
    // SAFETY: The packager checks this linked export; buffers outlive every step.
    let mut app = unsafe { Instance::new(omega_app_step, state, packet) }
        .ok_or(Error(abi::INVALID_ARGUMENT as i64))?;
    app.packet[14] = u64::from(host.parent.is_some());
    loop {
        let operation = app.advance();
        if operation == 0 {
            return Err(Error(abi::IO_ERROR as i64));
        }
        app.packet[9] = 0;
        if let Err(error) = host.execute(operation, app.packet) {
            app.packet[9] = error.0 as u64;
        }
    }
}
impl Host {
    fn current(&self) -> Result<Request, Error> {
        self.request.ok_or(Error(abi::INVALID_ARGUMENT as i64))
    }
    fn execute(&mut self, operation: u64, p: &mut [u64; PACKET_WORDS]) -> Result<(), Error> {
        let words: wire::Packet = p[..8].try_into().unwrap();
        match operation {
            10 => self.next(p)?,
            11 => {
                self.children.start_with(
                    p[0] as usize,
                    p[1],
                    p[2],
                    &mut self.server,
                    self.world,
                )?;
                p[..8].fill(0);
            }
            12 => {
                self.children
                    .stop(p[0] as usize, &mut self.server, self.world)?;
                p[..8].fill(0);
            }
            13 => self.initialize(p)?,
            21 => {
                self.refresh()?;
            }
            14 => p[..8].copy_from_slice(&self.world.owner_command(words)),
            15 => {
                let request = self.current()?;
                let slot = request.client.ok_or(Error(abi::DENIED as i64))?;
                p[..8].copy_from_slice(&self.world.request(slot, request.incarnation, words));
            }
            16 => {
                let request = self.current()?;
                self.server.reply(&request, &wire::encode(words))?;
                self.request = None;
            }
            17 => {
                let request = self.current()?;
                let slot = request.client.ok_or(Error(abi::DENIED as i64))?;
                let child = self.children.0[slot]
                    .as_mut()
                    .ok_or(Error(abi::PEER_CLOSED as i64))?;
                if child.data.0.raw() != request.incarnation {
                    return Err(Error(abi::DENIED as i64));
                }
                child
                    .services
                    .as_mut()
                    .ok_or(Error(abi::DENIED as i64))?
                    .reply(&mut self.server, &request, words)?;
                self.request = None;
            }
            18 => {
                let request = self.current()?;
                if !request.control || self.pending.is_some() {
                    return Err(Error(abi::DENIED as i64));
                }
                self.pending = Some(Pending::start(request, words, &self.children)?);
                self.request = None;
            }
            19 => p[..8].copy_from_slice(&self.world.source_command(words)),
            // Existing proof-only fault exercise; not a runtime or app-host API.
            20 => self.children.prove_rollback(p[0])?,
            _ => return Err(Error(abi::INVALID_ARGUMENT as i64)),
        }
        Ok(())
    }
}
