//! Reserve every admission name before mapping; publish only after admission succeeds.
use super::{Session, abi};
use crate::user_tasks::admission::pool::Code;
pub(super) struct Plan {
    pub child: u64,
    clock: u64,
    device: u64,
    executables: [Option<(usize, u64)>; abi::MAX_LAUNCHES],
    links: [u64; abi::MAX_ENDPOINTS / 2],
}
impl Plan {
    pub fn prepare(session: &mut Session, owner: usize, child: usize) -> Result<Self, u64> {
        let mut plan = Self {
            child: session.names.mint(owner)?,
            clock: 0,
            device: 0,
            executables: [None; abi::MAX_LAUNCHES],
            links: [0; abi::MAX_ENDPOINTS / 2],
        };
        if session.clock.allowed(child) {
            plan.clock = session.names.mint(child)?;
        }
        let launch = session
            .launches
            .iter()
            .find(|l| l.model.child == child)
            .unwrap();
        if launch.device.is_some() {
            plan.device = session.names.mint(child)?;
        }
        let mut templates = launch.images;
        // The selected dynamic template occupies the first child-local grant;
        // object code is consumed by its own admission and lends nothing.
        if launch.dynamic
            && let Some(Code::Template(selected)) = launch.code
        {
            let mut staged = [None; abi::MAX_LAUNCHES];
            staged[0] = Some(selected);
            let mut count = 1;
            for template in templates.into_iter().flatten().filter(|&t| t != selected) {
                if count == staged.len() {
                    return Err(abi::NO_MEMORY);
                }
                staged[count] = Some(template);
                count += 1;
            }
            templates = staged;
        }
        for (grant, template) in plan
            .executables
            .iter_mut()
            .zip(templates.into_iter().flatten())
        {
            *grant = Some((template, session.names.mint(child)?));
        }
        for (index, link) in session.links.iter().enumerate() {
            if link.model.spec.client == child || link.model.spec.service == child {
                plan.links[index] = session.names.mint(child)?;
            }
        }
        Ok(plan)
    }
    pub fn commit(self, session: &mut Session, child: usize) {
        session.clock.install(child, self.clock);
        if let Some(device) = session
            .launches
            .iter()
            .find(|l| l.model.child == child)
            .and_then(|l| l.device)
        {
            session
                .devices
                .install(child, self.device, device.register_bytes());
        }
        for (template, name) in self.executables.into_iter().flatten() {
            session
                .executables
                .install(child, Code::Template(template), name)
                .expect("fresh executable grant arena");
        }
        super::links::admit(session, child, &self.links);
    }
}
