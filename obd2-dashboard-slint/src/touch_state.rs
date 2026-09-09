// The software renderer maps landscape (x, y) to portrait (y, 927 - x).
// Apply its inverse to normalized libinput coordinates: (u, v) -> (1-v, u).
pub const CALIBRATION: [f32; 6] = [0., -1., 1., 1., 0., 0.];
pub const MAX_TOUCHES: usize = 10;

#[derive(Default)]
pub struct TouchState {
    pub positions: [Option<(f32, f32)>; MAX_TOUCHES],
}

impl TouchState {
    pub fn update(&mut self, slot: Option<u32>, position: Option<(f32, f32)>) {
        if let Some(target) = slot.and_then(|s| self.positions.get_mut(s as usize)) {
            *target = position;
        }
    }

    pub fn clear(&mut self) {
        self.positions.fill(None);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn portrait_corners_map_to_rotated_dashboard() {
        for ((u, v), expected) in [
            ((0., 0.), (1., 0.)),
            ((1., 0.), (1., 1.)),
            ((0., 1.), (0., 0.)),
            ((1., 1.), (0., 1.)),
            ((0.5, 0.5), (0.5, 0.5)),
        ] {
            let [a, b, c, d, e, f] = CALIBRATION;
            assert_eq!((a * u + b * v + c, d * u + e * v + f), expected);
        }
    }

    #[test]
    fn multiple_fingers_release_independently_and_reuse_slots() {
        let mut state = TouchState::default();
        state.update(Some(0), Some((0.2, 0.3)));
        state.update(Some(9), Some((0.8, 0.7)));
        state.update(Some(0), None);
        assert_eq!(state.positions[0], None);
        assert_eq!(state.positions[9], Some((0.8, 0.7)));
        state.update(Some(0), Some((0.4, 0.5)));
        state.update(Some(10), Some((1., 1.)));
        state.update(None, Some((1., 1.)));
        assert_eq!(state.positions.iter().flatten().count(), 2);
        state.clear();
        assert!(state.positions.iter().all(Option::is_none));
    }
}
