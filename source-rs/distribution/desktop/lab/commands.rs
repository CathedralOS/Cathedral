//! Owner commands for the color app in lab compositions: staged presentation,
//! rejected updates, reload, toggle, exit and liveness. The shipped app has
//! none of these; they exist so proofs can drive it over its control link.
use super::*;
use cathedral_user_runtime::task;
pub(crate) fn command(op: u64, p: &mut app::Probe<'_>) -> Option<u64> {
    let mut status = 0;
    match op {
        1 => p.scene.as_mut().unwrap().remove_accent(p.parent),
        2 => p.scene.as_mut().unwrap().restore_accent(p.parent),
        3 | 9 => p.scene.as_mut().unwrap().staged_probe(p.parent),
        4 => p.scene.as_ref().unwrap().commit(p.parent),
        5 => p.scene.as_mut().unwrap().rejected_probe(p.parent),
        6 => {
            let color = if p.style == 0 { app::LEFT } else { app::RIGHT };
            p.scene
                .as_mut()
                .unwrap()
                .paint(p.parent, p.view, color, 0)
                .unwrap();
        }
        7 => {
            status = p.state.as_mut().unwrap().reload().err().unwrap_or(0);
            p.repaint(false);
        }
        8 => {
            status = p.state.as_mut().unwrap().toggle().err().unwrap_or(0);
            p.repaint(true);
        }
        10 => task::exit(19),
        11 => p.scene.as_mut().unwrap().compact_probe(p.parent),
        12 => {
            // The owner gets admission acknowledgement before the save blocks;
            // completion is observed through a later liveness probe.
            p.control.1.send(&wire::encode([0; 8])).unwrap();
            *p.save_status = p.state.as_mut().unwrap().toggle().err().unwrap_or(0);
            p.repaint(true);
            return None;
        }
        // Readiness/failure probe; never mutates state.
        13 => status = p.state.as_ref().map_or(abi::BUSY, |s| s.error),
        _ => (),
    }
    Some(status)
}
