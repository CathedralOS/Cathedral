//! Reserve every admission name before mapping; publish only after admission succeeds.
use super::{Session, abi};
pub(super) struct Plan {
    pub child: u64,
    clock: u64,
    executable: Option<(usize, u64)>,
    links: [u64; abi::MAX_ENDPOINTS / 2],
}
impl Plan {
    pub fn prepare(session: &mut Session, owner: usize, child: usize) -> Result<Self, u64> {
        let mut plan = Self {
            child: session.names.mint(owner)?,
            clock: 0,
            executable: None,
            links: [0; abi::MAX_ENDPOINTS / 2],
        };
        if session.clock.allowed(child) {
            plan.clock = session.names.mint(child)?;
        }
        if let Some(template) = session
            .launches
            .iter()
            .find(|l| l.model.child == child && l.dynamic)
            .and_then(|l| l.template)
        {
            plan.executable = Some((template, session.names.mint(child)?));
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
        if let Some((template, name)) = self.executable {
            session
                .executables
                .install(child, template, name)
                .expect("fresh executable grant arena");
        }
        super::super::links::admit(session, child, &self.links);
    }
}
