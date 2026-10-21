// Real-frame staged admission regressions, shared by the x86 and ARM64 smoke labs.
use super::*;
use crate::user_tasks::address_space::{USER_CODE, USER_DATA};
use crate::user_tasks::execution::deferred::Work;
use crate::user_tasks::{Config, Supervision, lifecycle, startup};
use crate::{
    ipc::{EndpointSpec, Right},
    memory::FrameAllocator,
    scheduler::TaskState,
};
use cathedral_kernel_contracts::{memory as wire, user as abi};
use fixture::{discard_draft, finish, reclaim, restored, setup, unpublished};

/// # Safety
/// Sole boot CPU, inactive user machinery, initialized heap and live kernel layout.
/// Any supplied framebuffer is the exclusively held, reserved boot aperture.
#[inline(never)]
pub(in crate::user_tasks) unsafe fn exercise(
    frames: &mut FrameAllocator,
    layout: &arch::BootLayout,
    image: arch::ImageRange,
    framebuffer: Option<cathedral_kernel_contracts::framebuffer::Framebuffer>,
    output: fn(&[u8]) -> bool,
) {
    output(b"staged admission: physical rollback\n");
    let bytes = fixture::executable();
    let program = Program {
        executable: Executable::Elf(&bytes),
        arguments: [0, 7],
    };
    // SAFETY: No fixture root is ever executed; all frames are privately held here.
    unsafe {
        construction::physical(frames, layout, image, &program, framebuffer, output);
        output(b"staged admission: refused launch cleanup\n");
        for failure in 0..3 {
            refusal::exercise(frames, layout, image, &program, failure);
        }
        output(b"staged admission: bounded refusal, peer turns and retry passed\n");
        output(b"staged admission: interrupted authority\n");
        for death in 0..3 {
            authority::interrupted(frames, layout, image, &program, death);
        }
        output(b"staged admission: abandoned source\n");
        for refusing in [false, true] {
            authority::abandoned_object(frames, layout, image, &program, refusing);
        }
        output(b"staged admission: rollback, cancellation and source reclamation passed\n");
    }
}
