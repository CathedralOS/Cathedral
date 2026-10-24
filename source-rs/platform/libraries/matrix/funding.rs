//! Local requested amounts and owner-selected ceilings. No global resource view.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Grant {
    pub funded: [u64; 5],
    pub ceiling: [u64; 5],
}
impl Grant {
    pub fn increased(self, delta: [u64; 5]) -> Option<Self> {
        let mut next = self;
        for (i, amount) in delta.into_iter().enumerate() {
            next.funded[i] = self.funded[i].checked_add(amount)?;
            if next.funded[i] > self.ceiling[i] {
                return None;
            }
        }
        Some(next)
    }
}
pub fn shortfall(available: [u64; 5], needed: [u64; 5]) -> [u64; 5] {
    core::array::from_fn(|i| needed[i].saturating_sub(available[i]))
}
/// Physical backing covers the current load plus its private objects, while
/// retaining the existing per-process allowance for a declared subtree.
pub fn initial_frames(admission_frames: u64, private_pages: u64, descendants: u64) -> Option<u64> {
    Some(
        admission_frames
            .checked_add(private_pages)?
            .max(descendants.checked_add(1)?.checked_mul(admission_frames)?),
    )
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn initial_backing_covers_load_and_drawings_and_checks_overflow() {
        assert_eq!(initial_frames(256, 128, 0), Some(384));
        assert_eq!(initial_frames(256, 384, 5), Some(1536));
        assert_eq!(initial_frames(256, 0, 0), Some(256));
        assert_eq!(initial_frames(256, u64::MAX, 0), None);
        assert_eq!(initial_frames(256, 0, u64::MAX), None);
    }
    #[test]
    fn ceiling_refuses_all_dimensions_and_overflow_without_mutation() {
        let grant = Grant {
            funded: [2; 5],
            ceiling: [4; 5],
        };
        assert_eq!(grant.increased([1; 5]).unwrap().funded, [3; 5]);
        for i in 0..5 {
            let mut delta = [0; 5];
            delta[i] = 3;
            assert!(grant.increased(delta).is_none());
            delta[i] = u64::MAX;
            assert!(grant.increased(delta).is_none());
        }
        assert_eq!(grant.funded, [2; 5]);
        assert_eq!(
            shortfall([3, 1, 0, 5, 9], [2, 4, 7, 1, 10]),
            [0, 3, 7, 0, 1]
        );
    }
}
