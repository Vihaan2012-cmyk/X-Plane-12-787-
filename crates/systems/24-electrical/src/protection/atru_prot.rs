pub const ATRU_NAMES: [&str; 4] = ["ATRU_L1", "ATRU_L2", "ATRU_R1", "ATRU_R2"];
pub const ATRU_FEED_BUS: [&str; 4] = ["L1", "L2", "R1", "R2"];
pub const ATRU_KVA_TIERS: [(f64, f64); 5] = [
    (155.0, 600.0),
    (165.0, 300.0),
    (187.5, 30.0),
    (200.0, 5.0),
    (245.0, 0.0),
];
pub const ATRU_IN_HZ: (f64, f64) = crate::sources::machine::VFSG_FREQ_RANGE_HZ;
pub const ATRU_IN_115_HZ: (f64, f64) = (380.0, 430.0);
pub const ATRU_OUT_HALF_V: (f64, f64, f64) = (250.0, 280.0, 320.0);
pub const ATRU_OV_US: u64 = 100_000;
pub const ATRU_115_INPUT_KVA: f64 = 65.0;
pub const ATRU_INPUT_LIVE_V: f64 = 100.0;

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct AtruInput {
    pub dt_us: u64,
    pub input_v: f64,
    pub input_hz: f64,
    pub on_115_input: bool,
    pub load_kva: f64,
    pub out_half_v: f64,
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct AtruProt {
    timers_us: [u64; 5],
    t_ov: u64,
    overload: bool,
    overvoltage: bool,
    freq_out: bool,
}

impl AtruProt {
    pub fn step(&mut self, i: &AtruInput) -> bool {
        if i.input_v < ATRU_INPUT_LIVE_V {
            *self = AtruProt::default();
            return false;
        }
        let (lo, hi) = if i.on_115_input {
            ATRU_IN_115_HZ
        } else {
            ATRU_IN_HZ
        };
        self.freq_out = i.input_hz < lo || i.input_hz > hi;
        let scale = if i.on_115_input {
            ATRU_KVA_TIERS[0].0 / ATRU_115_INPUT_KVA
        } else {
            1.0
        };
        let kva = i.load_kva * scale;
        for (k, (limit, secs)) in ATRU_KVA_TIERS.iter().enumerate() {
            if kva > *limit {
                self.timers_us[k] = self.timers_us[k].saturating_add(i.dt_us);
                if self.timers_us[k] as f64 >= secs * 1.0e6 {
                    self.overload = true;
                }
            } else {
                self.timers_us[k] = 0;
            }
        }
        if i.out_half_v > ATRU_OUT_HALF_V.2 {
            self.t_ov = self.t_ov.saturating_add(i.dt_us);
            if self.t_ov >= ATRU_OV_US {
                self.overvoltage = true;
            }
        } else {
            self.t_ov = 0;
        }
        self.shut_down()
    }

    pub fn shut_down(&self) -> bool {
        self.overload || self.overvoltage || self.freq_out
    }

    pub fn overload(&self) -> bool {
        self.overload
    }

    pub fn overvoltage(&self) -> bool {
        self.overvoltage
    }

    pub fn freq_out(&self) -> bool {
        self.freq_out
    }
}

pub fn out_of_tolerance(out_half_v: f64) -> bool {
    out_half_v < ATRU_OUT_HALF_V.0 || out_half_v > ATRU_OUT_HALF_V.2
}

#[cfg(test)]
mod tests {
    use super::*;

    fn input(kva: f64) -> AtruInput {
        AtruInput {
            dt_us: 100_000,
            input_v: 235.0,
            input_hz: 400.0,
            load_kva: kva,
            out_half_v: 280.0,
            ..AtruInput::default()
        }
    }

    fn run(p: &mut AtruProt, i: &AtruInput, s: f64) -> bool {
        let mut t = false;
        for _ in 0..(s * 10.0).round() as u64 {
            t = p.step(i);
        }
        t
    }

    #[test]
    fn continuous_rating_never_trips_and_each_overload_band_has_its_time() {
        let mut p = AtruProt::default();
        assert!(!run(&mut p, &input(155.0), 7200.0));
        for (kva, secs) in [(160.0, 600.0), (180.0, 300.0), (190.0, 30.0), (220.0, 5.0)] {
            let mut p = AtruProt::default();
            assert!(!run(&mut p, &input(kva), secs - 0.2), "{kva} early");
            assert!(run(&mut p, &input(kva), 0.3), "{kva} late");
        }
        let mut p = AtruProt::default();
        assert!(p.step(&input(250.0)));
    }

    #[test]
    fn input_frequency_window_depends_on_input_and_recovers() {
        let mut p = AtruProt::default();
        let mut i = input(50.0);
        i.input_hz = 820.0;
        assert!(p.step(&i));
        i.input_hz = 700.0;
        assert!(!p.step(&i));
        i.on_115_input = true;
        assert!(p.step(&i));
        i.input_hz = 400.0;
        assert!(!p.step(&i));
    }

    #[test]
    fn output_overvoltage_latches_and_input_loss_resets() {
        let mut p = AtruProt::default();
        let mut i = input(50.0);
        i.out_half_v = 330.0;
        assert!(run(&mut p, &i, 0.1));
        i.out_half_v = 280.0;
        assert!(p.step(&i));
        i.input_v = 0.0;
        assert!(!p.step(&i));
        assert!(!out_of_tolerance(280.0) && out_of_tolerance(240.0) && out_of_tolerance(325.0));
    }
}
