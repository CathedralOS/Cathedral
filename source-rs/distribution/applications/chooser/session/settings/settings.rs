// Edit a saved arrangement through the launcher's existing realm grant.
// Saving uses revision-checked storage, then requests geometry from the matrix.
use super::*;
use cathedral_desktop_matrix_contracts::matrix::config::Config;
use cathedral_user_runtime::storage::Scope;

pub struct Editor {
    object: u64,
    revision: u64,
    saved: Config,
    pub draft: Config,
    pub selected: usize,
    pub status: &'static str,
    uncertain: bool,
}
impl Editor {
    pub fn open(scope: &mut Scope) -> Result<Self, u64> {
        let (object, revision, saved) = persistence::load(scope)?;
        Ok(Self {
            object,
            revision,
            saved,
            draft: saved,
            selected: 0,
            status: "",
            uncertain: false,
        })
    }
    pub fn dirty(&self) -> bool {
        self.saved != self.draft || self.uncertain
    }
    /// Return repaint/close. Pointer movement alone does not repaint the panel.
    pub fn event(
        &mut self,
        event: wire::Packet,
        extent: wire::Packet,
        pressed: &mut bool,
        scope: &mut Scope,
        parent: Pair,
    ) -> (bool, bool) {
        let action = match event[2] {
            wire::POINTER => {
                let down = event[5] & 1 != 0;
                let action = if down && !*pressed {
                    view::hit(extent, event[3], event[4])
                } else {
                    None
                };
                *pressed = down;
                action
            }
            wire::CANCEL => {
                *pressed = false;
                None
            }
            wire::KEY if event[4] == input::PRESS as u64 => match event[3] as u8 {
                input::UP | input::LEFT => {
                    self.selected = (self.selected + 5) % 6;
                    return (true, false);
                }
                input::DOWN | input::RIGHT => {
                    self.selected = (self.selected + 1) % 6;
                    return (true, false);
                }
                input::ENTER => Some(self.selected),
                _ => None,
            },
            _ => None,
        };
        let Some(action) = action else {
            return (false, false);
        };
        self.selected = action;
        match action {
            0 => {
                self.draft.launcher_width =
                    self.draft.launcher_width.saturating_sub(64).clamp(192, 320)
            }
            1 => {
                self.draft.launcher_width =
                    self.draft.launcher_width.saturating_add(64).clamp(192, 320)
            }
            2 => self.draft.margin = self.draft.margin.saturating_sub(16).clamp(8, 40),
            3 => self.draft.margin = self.draft.margin.saturating_add(16).clamp(8, 40),
            4 => {
                self.status = match self.save(scope) {
                    Ok(()) => {
                        let reply = call(
                            parent,
                            [
                                wire::APPLY_LAYOUT,
                                extent[1],
                                self.draft.margin,
                                self.draft.launcher_width,
                                self.draft.gap,
                                0,
                                0,
                                0,
                            ],
                        );
                        if reply[0] == 0 {
                            "APPLIED"
                        } else {
                            "SAVED; RESTART"
                        }
                    }
                    Err(abi::WOULD_BLOCK) => "CHANGED",
                    Err(_) => "SAVE FAILED",
                };
                return (true, false);
            }
            5 => return (true, true), // CANCEL explicitly discards the local draft.
            _ => unreachable!(),
        }
        self.status = "UNSAVED";
        (true, false)
    }
    fn save(&mut self, scope: &mut Scope) -> Result<(), u64> {
        if !self.dirty() {
            self.status = "SAVED";
            return Ok(());
        }
        // Reconcile a lost acknowledgement before retrying. Never overwrite
        // someone else's intervening edit, even if the path still resolves.
        let (object, revision, current) = persistence::load(scope)?;
        if object != self.object {
            return Err(abi::WOULD_BLOCK);
        }
        if current == self.draft {
            self.saved = current;
            self.revision = revision;
            self.uncertain = false;
            return Ok(());
        }
        if revision != self.revision || current != self.saved {
            return Err(abi::WOULD_BLOCK);
        }
        self.uncertain = true;
        self.revision = persistence::save(scope, object, revision, self.draft)?;
        self.saved = self.draft;
        self.uncertain = false;
        Ok(())
    }
}
