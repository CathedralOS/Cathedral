//! Send prepared geometry with retained image readers; no world mutation here.
use super::*;
impl Parent {
    pub(super) fn send_scene(&mut self) -> Result<Option<Completed>, Error> {
        let p = self.scene.as_mut().unwrap();
        #[cfg(feature = "scene-delay")]
        if !cfg!(feature = "scene-commit-hold")
            && p.step == p.count + 1
            && let Some(until) = p.hold
        {
            if until == 0 {
                p.hold = Some(time::after(300)?);
                time::storage_sample("launch-scene-held", &[p.ticket.raw()]);
                return Ok(None);
            }
            if !time::reached(time::now()?, until) {
                return Ok(None);
            }
            p.hold = None;
            time::storage_sample("launch-scene-released", &[p.ticket.raw()]);
            #[cfg(feature = "scene-failure")]
            if p.status == 0 {
                p.status = abi::IO_ERROR;
            }
        }
        if p.status != 0 {
            if p.step == 0 {
                return Ok(Some(self.finish_scene(false)));
            }
            p.step = p.count + 2;
        }
        let op = if p.step == 0 {
            wire::PRESENT_BEGIN
        } else if p.step == p.count + 1 {
            wire::PRESENT_COMMIT
        } else if p.step == p.count + 2 {
            wire::PRESENT_ABORT
        } else {
            0
        };
        let words = if op != 0 {
            [op, self.grant.epoch, p.revision, 0, 0, 0, 0, 0]
        } else {
            self.scene_packets[p.step - 1]
        };
        let object = if words[0] == wire::SURFACE_IMAGE {
            Some(self.images.get(words[2])?)
        } else {
            None
        };
        match self.send_output(words, object) {
            Ok(()) => {
                let p = self.scene.as_mut().unwrap();
                p.waiting = true;
                p.retry = None;
            }
            Err(Error(code)) if code == abi::WOULD_BLOCK as i64 => {
                self.scene.as_mut().unwrap().retry = Some(time::after(1)?)
            }
            Err(e) => return Err(e),
        }
        Ok(None)
    }
}
