pub const TRU_OC_TIERS: [(f64, f64); 3] = [(280.0, 305.0), (360.0, 48.0), (420.0, 12.0)];
pub const TRU_NAMES: [&str; 4] = ["TRU_L", "TRU_R", "TRU_C1", "TRU_C2"];
pub const TRU_INPUT_LIVE_V: f64 = 100.0;

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct TruOc {
    timers_us: [u64; 3],
    tripped: bool,
}

impl TruOc {
    pub fn step(&mut self, current_a: f64, input_v: f64, dt_us: u64) -> bool {
        if input_v < TRU_INPUT_LIVE_V {
            *self = TruOc::default();
            return false;
        }
        if self.tripped {
            return true;
        }
        for (k, (amps, secs)) in TRU_OC_TIERS.iter().enumerate() {
            if current_a >= *amps {
                self.timers_us[k] = self.timers_us[k].saturating_add(dt_us);
                if self.timers_us[k] as f64 >= secs * 1.0e6 {
                    self.tripped = true;
                }
            } else {
                self.timers_us[k] = 0;
            }
        }
        self.tripped
    }

    pub fn tripped(&self) -> bool {
        self.tripped
    }

    pub fn timer_s(&self, tier: usize) -> f64 {
        self.timers_us
            .get(tier)
            .map(|t| *t as f64 * 1.0e-6)
            .unwrap_or(0.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn run(oc: &mut TruOc, amps: f64, secs: f64) -> bool {
        let mut t = false;
        for _ in 0..(secs * 100.0).round() as u64 {
            t = oc.step(amps, 235.0, 10_000);
        }
        t
    }

    #[test]
    fn each_tier_trips_at_its_documented_time() {
        for (amps, secs) in TRU_OC_TIERS {
            let mut oc = TruOc::default();
            assert!(!run(&mut oc, amps + 1.0, secs - 0.05), "{amps} A early");
            assert!(run(&mut oc, amps + 1.0, 0.1), "{amps} A late");
        }
    }

    #[test]
    fn below_tier_one_never_trips_and_dip_resets_timer() {
        let mut oc = TruOc::default();
        assert!(!run(&mut oc, 279.0, 1000.0));
        run(&mut oc, 425.0, 11.0);
        run(&mut oc, 200.0, 0.01);
        assert!(!run(&mut oc, 425.0, 11.0));
    }

    #[test]
    fn trip_latches_until_input_power_is_removed() {
        let mut oc = TruOc::default();
        assert!(run(&mut oc, 500.0, 12.1));
        assert!(run(&mut oc, 0.0, 5.0));
        assert!(!oc.step(0.0, 0.0, 10_000));
        assert!(!oc.tripped());
    }
}
