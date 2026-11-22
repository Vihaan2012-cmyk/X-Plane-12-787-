#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct ZoneDiff {
    timer_us: u64,
    tripped: bool,
}

impl ZoneDiff {
    pub fn step(&mut self, leak_a: f64, threshold_a: f64, delay_us: u64, dt_us: u64) -> bool {
        if self.tripped {
            return true;
        }
        if leak_a > threshold_a {
            self.timer_us = self.timer_us.saturating_add(dt_us);
            if self.timer_us >= delay_us {
                self.tripped = true;
            }
        } else {
            self.timer_us = 0;
        }
        self.tripped
    }

    pub fn reset(&mut self) {
        *self = ZoneDiff::default();
    }

    pub fn tripped(&self) -> bool {
        self.tripped
    }
}

pub const TIE_DIFF_A: f64 = 30.0;
pub const TIE_DIFF_HW_US: u64 = 50_000;
pub const TIE_DIFF_SW_US: u64 = 210_000;
pub const ATU_DIFF_A: f64 = 40.0;
pub const ATU_DIFF_US: u64 = 100_000;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn trips_only_after_continuous_leak_for_the_delay_then_latches() {
        let mut z = ZoneDiff::default();
        for _ in 0..9 {
            assert!(!z.step(ATU_DIFF_A + 1.0, ATU_DIFF_A, ATU_DIFF_US, 10_000));
        }
        assert!(!z.step(0.0, ATU_DIFF_A, ATU_DIFF_US, 10_000));
        for _ in 0..9 {
            assert!(!z.step(ATU_DIFF_A + 1.0, ATU_DIFF_A, ATU_DIFF_US, 10_000));
        }
        assert!(z.step(ATU_DIFF_A + 1.0, ATU_DIFF_A, ATU_DIFF_US, 10_000));
        assert!(z.step(0.0, ATU_DIFF_A, ATU_DIFF_US, 10_000));
        z.reset();
        assert!(!z.tripped());
    }

    #[test]
    fn tie_diff_needs_hardware_plus_software_time() {
        let mut z = ZoneDiff::default();
        let total = TIE_DIFF_HW_US + TIE_DIFF_SW_US;
        let steps = total / 10_000;
        for _ in 0..steps - 1 {
            assert!(!z.step(TIE_DIFF_A + 5.0, TIE_DIFF_A, total, 10_000));
        }
        assert!(z.step(TIE_DIFF_A + 5.0, TIE_DIFF_A, total, 10_000));
    }
}
