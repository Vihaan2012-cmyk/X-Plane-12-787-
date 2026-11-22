use crate::rng::{SplitMix64, seed_for};

pub const BOLTZMANN_EV_PER_K: f64 = 8.617_333_262e-5;
pub const US_PER_HOUR: f64 = 3_600_000_000.0;

pub fn arrhenius_factor(temp_k: f64, ref_temp_k: f64, activation_ev: f64) -> f64 {
    if temp_k <= 0.0 || ref_temp_k <= 0.0 {
        return 1.0;
    }
    (activation_ev / BOLTZMANN_EV_PER_K * (1.0 / ref_temp_k - 1.0 / temp_k)).exp()
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct HazardModel {
    pub base_per_hour: f64,
    pub wear_gain: f64,
    pub wear_exponent: f64,
    pub ref_temp_k: f64,
    pub activation_ev: f64,
}

impl HazardModel {
    pub fn new(base_per_hour: f64) -> Self {
        Self {
            base_per_hour,
            wear_gain: 0.0,
            wear_exponent: 1.0,
            ref_temp_k: 298.15,
            activation_ev: 0.0,
        }
    }

    pub fn with_wear(mut self, gain: f64, exponent: f64) -> Self {
        self.wear_gain = gain;
        self.wear_exponent = exponent;
        self
    }

    pub fn with_thermal(mut self, ref_temp_k: f64, activation_ev: f64) -> Self {
        self.ref_temp_k = ref_temp_k;
        self.activation_ev = activation_ev;
        self
    }

    pub fn hazard_per_hour(&self, wear: f64, temp_k: f64) -> f64 {
        let w = wear.clamp(0.0, 1.0);
        let wear_factor = 1.0 + self.wear_gain * w.powf(self.wear_exponent);
        let thermal = arrhenius_factor(temp_k, self.ref_temp_k, self.activation_ev);
        (self.base_per_hour * wear_factor * thermal).max(0.0)
    }

    pub fn failure_probability(&self, wear: f64, temp_k: f64, hours: f64) -> f64 {
        1.0 - (-self.hazard_per_hour(wear, temp_k) * hours.max(0.0)).exp()
    }

    pub fn time_to_failure_hours(
        &self,
        seed: u64,
        wear0: f64,
        wear_rate_per_hour: f64,
        temp_k: f64,
        horizon_hours: f64,
        step_hours: f64,
    ) -> Option<f64> {
        let target = exponential_target(seed);
        let step = step_hours.max(1e-9);
        let mut t = 0.0;
        let mut accumulated = 0.0;
        while t < horizon_hours {
            let dt = step.min(horizon_hours - t);
            let wear_mid = wear0 + wear_rate_per_hour * (t + 0.5 * dt);
            let rate = self.hazard_per_hour(wear_mid, temp_k);
            let add = rate * dt;
            if rate > 0.0 && accumulated + add >= target {
                return Some(t + (target - accumulated) / rate);
            }
            accumulated += add;
            t += dt;
        }
        None
    }

    pub fn time_to_failure_us(
        &self,
        seed: u64,
        wear0: f64,
        wear_rate_per_hour: f64,
        temp_k: f64,
        horizon_hours: f64,
        step_hours: f64,
    ) -> Option<u64> {
        self.time_to_failure_hours(seed, wear0, wear_rate_per_hour, temp_k, horizon_hours, step_hours)
            .map(|h| (h * US_PER_HOUR).round() as u64)
    }
}

pub fn exponential_target(seed: u64) -> f64 {
    let u = SplitMix64::new(seed).next_f64();
    -(1.0 - u).ln()
}

pub fn constant_hazard_failure_hours(seed: u64, hazard_per_hour: f64) -> Option<f64> {
    if hazard_per_hour <= 0.0 {
        return None;
    }
    Some(exponential_target(seed) / hazard_per_hour)
}

pub fn failure_seed(serial: &str, mode: &str) -> u64 {
    seed_for(serial, mode)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn arrhenius_is_one_at_reference() {
        assert!((arrhenius_factor(298.15, 298.15, 0.7) - 1.0).abs() < 1e-12);
    }

    #[test]
    fn arrhenius_rises_with_temperature() {
        let hot = arrhenius_factor(348.15, 298.15, 0.7);
        let cold = arrhenius_factor(248.15, 298.15, 0.7);
        assert!(hot > 1.0 && cold < 1.0);
        assert!(hot > 50.0 && hot < 500.0);
    }

    #[test]
    fn arrhenius_ten_degree_rule_roughly_doubles() {
        let f = arrhenius_factor(308.15, 298.15, 0.5);
        assert!(f > 1.5 && f < 2.5);
    }

    #[test]
    fn arrhenius_guards_nonpositive() {
        assert_eq!(arrhenius_factor(0.0, 298.15, 0.7), 1.0);
        assert_eq!(arrhenius_factor(300.0, -1.0, 0.7), 1.0);
    }

    #[test]
    fn hazard_rises_with_wear_and_heat() {
        let m = HazardModel::new(1e-4)
            .with_wear(9.0, 2.0)
            .with_thermal(298.15, 0.6);
        let base = m.hazard_per_hour(0.0, 298.15);
        assert!((base - 1e-4).abs() < 1e-15);
        assert!((m.hazard_per_hour(1.0, 298.15) - 1e-3).abs() < 1e-12);
        assert!(m.hazard_per_hour(0.5, 298.15) > base);
        assert!(m.hazard_per_hour(0.0, 340.0) > base);
        assert_eq!(m.hazard_per_hour(5.0, 298.15), m.hazard_per_hour(1.0, 298.15));
    }

    #[test]
    fn probability_monotonic() {
        let m = HazardModel::new(1e-3);
        let p1 = m.failure_probability(0.0, 298.15, 100.0);
        let p2 = m.failure_probability(0.0, 298.15, 1000.0);
        assert!(p1 > 0.0 && p2 > p1 && p2 < 1.0);
        assert_eq!(m.failure_probability(0.0, 298.15, -5.0), 0.0);
    }

    #[test]
    fn constant_hazard_deterministic() {
        let a = constant_hazard_failure_hours(5, 1e-3).unwrap();
        let b = constant_hazard_failure_hours(5, 1e-3).unwrap();
        assert_eq!(a.to_bits(), b.to_bits());
        assert!(constant_hazard_failure_hours(5, 0.0).is_none());
    }

    #[test]
    fn constant_hazard_mean_matches() {
        let h = 2e-3;
        let n = 20000;
        let mut sum = 0.0;
        for i in 0..n {
            sum += constant_hazard_failure_hours(seed_for("S", &format!("m{i}")), h).unwrap();
        }
        let mean = sum / f64::from(n);
        assert!((mean - 1.0 / h).abs() / (1.0 / h) < 0.05);
    }

    #[test]
    fn time_to_failure_matches_constant_hazard() {
        let m = HazardModel::new(1e-3);
        let seed = failure_seed("SN1", "open");
        let expected = constant_hazard_failure_hours(seed, 1e-3).unwrap();
        let got = m
            .time_to_failure_hours(seed, 0.0, 0.0, 298.15, 1e6, 10.0)
            .unwrap();
        assert!((got - expected).abs() < 1e-6 * expected.max(1.0));
    }

    #[test]
    fn wear_growth_shortens_life() {
        let m = HazardModel::new(1e-4).with_wear(50.0, 2.0);
        let seed = failure_seed("SN1", "open");
        let still = m.time_to_failure_hours(seed, 0.0, 0.0, 298.15, 1e7, 5.0).unwrap();
        let wearing = m
            .time_to_failure_hours(seed, 0.0, 0.001, 298.15, 1e7, 5.0)
            .unwrap();
        assert!(wearing < still);
    }

    #[test]
    fn heat_shortens_life() {
        let m = HazardModel::new(1e-4).with_thermal(298.15, 0.7);
        let seed = 77;
        let cool = m
            .time_to_failure_hours(seed, 0.0, 0.0, 298.15, 1e8, 10.0)
            .unwrap();
        let hot = m
            .time_to_failure_hours(seed, 0.0, 0.0, 358.15, 1e8, 10.0)
            .unwrap();
        assert!(hot < cool);
    }

    #[test]
    fn horizon_returns_none() {
        let m = HazardModel::new(1e-9);
        assert!(m.time_to_failure_hours(1, 0.0, 0.0, 298.15, 10.0, 1.0).is_none());
        let z = HazardModel::new(0.0);
        assert!(z.time_to_failure_hours(1, 0.0, 0.0, 298.15, 1e5, 10.0).is_none());
    }

    #[test]
    fn microsecond_conversion_deterministic() {
        let m = HazardModel::new(1e-2);
        let a = m.time_to_failure_us(3, 0.0, 0.0, 298.15, 1e5, 1.0).unwrap();
        let b = m.time_to_failure_us(3, 0.0, 0.0, 298.15, 1e5, 1.0).unwrap();
        assert_eq!(a, b);
        let h = m.time_to_failure_hours(3, 0.0, 0.0, 298.15, 1e5, 1.0).unwrap();
        assert_eq!(a, (h * US_PER_HOUR).round() as u64);
    }

    #[test]
    fn serial_changes_seed() {
        assert_ne!(failure_seed("A", "m"), failure_seed("B", "m"));
    }
}
