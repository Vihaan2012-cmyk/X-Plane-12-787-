use crate::time::{SimDuration, SimTime};

const PPB: i128 = 1_000_000_000;
const MICROS_PER_YEAR: f64 = 365.25 * 86_400.0 * 1.0e6;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct OscillatorConfig {
    pub initial_offset_ppb: i64,
    pub turnover_celsius: f64,
    pub temp_coeff_ppb_per_c2: f64,
    pub aging_ppb_per_year: f64,
    pub max_aging_ppb: f64,
    pub initial_age: SimDuration,
}

impl OscillatorConfig {
    pub fn ideal() -> Self {
        OscillatorConfig {
            initial_offset_ppb: 0,
            turnover_celsius: 25.0,
            temp_coeff_ppb_per_c2: 0.0,
            aging_ppb_per_year: 0.0,
            max_aging_ppb: 0.0,
            initial_age: SimDuration::ZERO,
        }
    }

    pub fn tcxo(initial_offset_ppb: i64) -> Self {
        OscillatorConfig {
            initial_offset_ppb,
            turnover_celsius: 25.0,
            temp_coeff_ppb_per_c2: 35.0,
            aging_ppb_per_year: 1_000.0,
            max_aging_ppb: 5_000.0,
            initial_age: SimDuration::ZERO,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Oscillator {
    config: OscillatorConfig,
    local_us: u64,
    remainder: i128,
    age: SimDuration,
    sync_count: u64,
    last_step_us: i128,
}

impl Oscillator {
    pub fn new(config: OscillatorConfig, local_start: SimTime) -> Self {
        Oscillator {
            age: config.initial_age,
            config,
            local_us: local_start.as_micros(),
            remainder: 0,
            sync_count: 0,
            last_step_us: 0,
        }
    }

    pub fn config(&self) -> &OscillatorConfig {
        &self.config
    }

    pub fn age(&self) -> SimDuration {
        self.age
    }

    pub fn local_time(&self) -> SimTime {
        SimTime::from_micros(self.local_us)
    }

    pub fn sync_count(&self) -> u64 {
        self.sync_count
    }

    pub fn last_sync_step_us(&self) -> i128 {
        self.last_step_us
    }

    pub fn temperature_ppb(&self, temp_celsius: f64) -> i64 {
        let d = temp_celsius - self.config.turnover_celsius;
        let v = -(self.config.temp_coeff_ppb_per_c2 * d * d);
        if v.is_finite() { v.round() as i64 } else { 0 }
    }

    pub fn aging_ppb(&self) -> i64 {
        let years = self.age.as_micros() as f64 / MICROS_PER_YEAR;
        let v = (self.config.aging_ppb_per_year * years).min(self.config.max_aging_ppb);
        if v.is_finite() { v.round() as i64 } else { 0 }
    }

    pub fn frequency_error_ppb(&self, temp_celsius: f64) -> i64 {
        let total = i128::from(self.config.initial_offset_ppb)
            + i128::from(self.temperature_ppb(temp_celsius))
            + i128::from(self.aging_ppb());
        total.clamp(-(PPB - 1), PPB) as i64
    }

    pub fn advance(&mut self, dt: SimDuration, temp_celsius: f64) {
        let ppb = i128::from(self.frequency_error_ppb(temp_celsius));
        let delta = i128::from(dt.as_micros()) * (PPB + ppb);
        let total = self.remainder + delta;
        let whole = total.div_euclid(PPB);
        self.remainder = total.rem_euclid(PPB);
        let new = i128::from(self.local_us) + whole;
        self.local_us = u64::try_from(new.max(0)).unwrap_or(u64::MAX);
        self.age = self.age.saturating_add(dt);
    }

    pub fn advance_unpowered(&mut self, dt: SimDuration) {
        self.age = self.age.saturating_add(dt);
    }

    pub fn error_vs(&self, reference: SimTime) -> i128 {
        self.local_time().signed_micros_since(reference)
    }

    pub fn resync(&mut self, reference: SimTime) {
        self.last_step_us = reference.signed_micros_since(self.local_time());
        self.local_us = reference.as_micros();
        self.remainder = 0;
        self.sync_count += 1;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn secs(s: u64) -> SimDuration {
        SimDuration::from_secs(s)
    }

    #[test]
    fn ideal_oscillator_tracks_sim_time() {
        let mut o = Oscillator::new(OscillatorConfig::ideal(), SimTime::ZERO);
        for _ in 0..1_000 {
            o.advance(SimDuration::from_micros(16_667), 80.0);
        }
        assert_eq!(o.local_time().as_micros(), 16_667_000);
    }

    #[test]
    fn fixed_ppm_offset_gives_exact_drift() {
        let mut cfg = OscillatorConfig::ideal();
        cfg.initial_offset_ppb = 100_000;
        let mut o = Oscillator::new(cfg, SimTime::ZERO);
        o.advance(secs(1), 25.0);
        assert_eq!(o.local_time().as_micros(), 1_000_100);
        o.advance(secs(1_000_000 - 1), 25.0);
        assert_eq!(o.local_time().as_micros(), 1_000_100_000_000);
        assert_eq!(o.error_vs(SimTime::from_secs(1_000_000)), 100_000_000);
    }

    #[test]
    fn sub_microsecond_remainder_accumulates() {
        let mut cfg = OscillatorConfig::ideal();
        cfg.initial_offset_ppb = 1;
        let mut o = Oscillator::new(cfg, SimTime::ZERO);
        for _ in 0..1_000 {
            o.advance(secs(1), 25.0);
        }
        assert_eq!(o.local_time().as_micros(), 1_000_000_001);
    }

    #[test]
    fn slow_oscillator_runs_behind() {
        let mut cfg = OscillatorConfig::ideal();
        cfg.initial_offset_ppb = -50_000;
        let mut o = Oscillator::new(cfg, SimTime::ZERO);
        o.advance(secs(100), 25.0);
        assert_eq!(o.local_time().as_micros(), 100_000_000 - 5_000);
        assert!(o.error_vs(SimTime::from_secs(100)) < 0);
    }

    #[test]
    fn temperature_parabola_peaks_at_turnover() {
        let o = Oscillator::new(OscillatorConfig::tcxo(0), SimTime::ZERO);
        assert_eq!(o.temperature_ppb(25.0), 0);
        assert_eq!(o.temperature_ppb(35.0), -3_500);
        assert_eq!(o.temperature_ppb(15.0), -3_500);
        assert!(o.temperature_ppb(-55.0) < o.temperature_ppb(-20.0));
        assert!(o.temperature_ppb(70.0) < o.temperature_ppb(40.0));
    }

    #[test]
    fn temperature_changes_drift_rate() {
        let mut hot = Oscillator::new(OscillatorConfig::tcxo(0), SimTime::ZERO);
        let mut cold = Oscillator::new(OscillatorConfig::tcxo(0), SimTime::ZERO);
        let mut ref_ = Oscillator::new(OscillatorConfig::tcxo(0), SimTime::ZERO);
        for _ in 0..3_600 {
            hot.advance(secs(1), 75.0);
            cold.advance(secs(1), 25.0);
            ref_.advance(secs(1), 25.0);
        }
        assert_eq!(cold.local_time(), ref_.local_time());
        assert!(hot.local_time() < cold.local_time());
    }

    #[test]
    fn aging_grows_and_saturates() {
        let mut o = Oscillator::new(OscillatorConfig::tcxo(0), SimTime::ZERO);
        assert_eq!(o.aging_ppb(), 0);
        o.advance_unpowered(SimDuration::from_days(365));
        let a1 = o.aging_ppb();
        assert!((999..=1_001).contains(&a1));
        o.advance_unpowered(SimDuration::from_days(365 * 20));
        assert_eq!(o.aging_ppb(), 5_000);
        assert_eq!(o.age(), SimDuration::from_days(365 * 21));
    }

    #[test]
    fn aged_unit_has_drifted_more() {
        let mut cfg = OscillatorConfig::tcxo(0);
        cfg.initial_age = SimDuration::from_days(365 * 10);
        let old = Oscillator::new(cfg, SimTime::ZERO);
        let new = Oscillator::new(OscillatorConfig::tcxo(0), SimTime::ZERO);
        assert!(old.frequency_error_ppb(25.0) > new.frequency_error_ppb(25.0));
    }

    #[test]
    fn two_boxes_drift_apart_and_resync_converges() {
        let mut a = Oscillator::new(OscillatorConfig::tcxo(2_000), SimTime::ZERO);
        let mut b = Oscillator::new(OscillatorConfig::tcxo(-3_000), SimTime::ZERO);
        for _ in 0..86_400 {
            a.advance(secs(1), 40.0);
            b.advance(secs(1), 40.0);
        }
        let split = a.local_time().signed_micros_since(b.local_time());
        assert!(split > 100_000);
        let reference = SimTime::from_secs(86_400);
        a.resync(reference);
        b.resync(reference);
        assert_eq!(a.local_time(), b.local_time());
        assert_eq!(a.error_vs(reference), 0);
        assert_eq!(a.sync_count(), 1);
        assert!(b.last_sync_step_us() > 0);
        assert!(a.last_sync_step_us().abs() > 0);
    }

    #[test]
    fn resync_clears_remainder() {
        let mut cfg = OscillatorConfig::ideal();
        cfg.initial_offset_ppb = 500_000_000;
        let mut o = Oscillator::new(cfg, SimTime::ZERO);
        o.advance(SimDuration::from_micros(1), 25.0);
        o.resync(SimTime::from_secs(5));
        o.advance(SimDuration::from_micros(1), 25.0);
        assert_eq!(o.local_time().as_micros(), 5_000_001);
    }

    #[test]
    fn determinism_same_inputs_same_time() {
        let run = || {
            let mut o = Oscillator::new(OscillatorConfig::tcxo(777), SimTime::ZERO);
            for i in 0..10_000u64 {
                o.advance(SimDuration::from_micros(1_000 + i), 20.0 + (i % 50) as f64);
            }
            o.local_time()
        };
        assert_eq!(run(), run());
    }

    #[test]
    fn unpowered_oscillator_does_not_advance_local_time() {
        let mut o = Oscillator::new(OscillatorConfig::tcxo(0), SimTime::from_secs(3));
        o.advance_unpowered(secs(100));
        assert_eq!(o.local_time(), SimTime::from_secs(3));
    }
}
