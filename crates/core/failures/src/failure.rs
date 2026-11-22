use crate::registry::Registry;
use crate::rng::hash_at;
use std::collections::BTreeMap;
use std::fmt;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FailureMode {
    pub name: String,
    pub description: String,
    pub observables: Vec<String>,
}

impl FailureMode {
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            description: String::new(),
            observables: Vec::new(),
        }
    }

    pub fn described(mut self, description: &str) -> Self {
        self.description = description.to_string();
        self
    }

    pub fn observable_via(mut self, observable: &str) -> Self {
        self.observables.push(observable.to_string());
        self
    }

    pub fn is_observable(&self) -> bool {
        !self.observables.is_empty()
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum Onset {
    Immediate,
    Ramp { duration_us: u64 },
    Intermittent { seed: u64, period_us: u64, duty: f64 },
    Latent { trigger: String },
}

#[derive(Clone, Debug, PartialEq)]
pub struct Failure {
    pub mode: String,
    pub magnitude: f64,
    pub onset: Onset,
    pub armed_at_us: u64,
    pub triggered_at_us: Option<u64>,
}

impl Failure {
    pub fn magnitude_at(&self, now_us: u64) -> f64 {
        if now_us < self.armed_at_us {
            return 0.0;
        }
        let elapsed = now_us - self.armed_at_us;
        match &self.onset {
            Onset::Immediate => self.magnitude,
            Onset::Ramp { duration_us } => {
                if *duration_us == 0 || elapsed >= *duration_us {
                    self.magnitude
                } else {
                    self.magnitude * elapsed as f64 / *duration_us as f64
                }
            }
            Onset::Intermittent {
                seed,
                period_us,
                duty,
            } => {
                let slot = elapsed / (*period_us).max(1);
                if hash_at(*seed, slot) < *duty {
                    self.magnitude
                } else {
                    0.0
                }
            }
            Onset::Latent { .. } => match self.triggered_at_us {
                Some(t) if now_us >= t => self.magnitude,
                _ => 0.0,
            },
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum FailureError {
    UnknownMode { part_type: String, mode: String },
}

impl fmt::Display for FailureError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            FailureError::UnknownMode { part_type, mode } => {
                write!(f, "part type {part_type} has no failure mode {mode}")
            }
        }
    }
}

impl std::error::Error for FailureError {}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct FailureSet {
    failures: BTreeMap<String, Failure>,
}

impl FailureSet {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn arm(&mut self, mode: &str, magnitude: f64, onset: Onset, now_us: u64) {
        let magnitude = if magnitude.is_nan() {
            0.0
        } else {
            magnitude.clamp(0.0, 1.0)
        };
        self.failures.insert(
            mode.to_string(),
            Failure {
                mode: mode.to_string(),
                magnitude,
                onset,
                armed_at_us: now_us,
                triggered_at_us: None,
            },
        );
    }

    pub fn arm_checked(
        &mut self,
        registry: &Registry,
        part_type: &str,
        mode: &str,
        magnitude: f64,
        onset: Onset,
        now_us: u64,
    ) -> Result<(), FailureError> {
        if !registry.supports(part_type, mode) {
            return Err(FailureError::UnknownMode {
                part_type: part_type.to_string(),
                mode: mode.to_string(),
            });
        }
        self.arm(mode, magnitude, onset, now_us);
        Ok(())
    }

    pub fn arm_immediate(&mut self, mode: &str, magnitude: f64, now_us: u64) {
        self.arm(mode, magnitude, Onset::Immediate, now_us);
    }

    pub fn clear(&mut self, mode: &str) -> bool {
        self.failures.remove(mode).is_some()
    }

    pub fn clear_all(&mut self) {
        self.failures.clear();
    }

    pub fn trigger(&mut self, trigger: &str, now_us: u64) -> usize {
        let mut n = 0;
        for f in self.failures.values_mut() {
            if let Onset::Latent { trigger: t } = &f.onset
                && t == trigger
                && f.triggered_at_us.is_none()
            {
                f.triggered_at_us = Some(now_us);
                n += 1;
            }
        }
        n
    }

    pub fn get(&self, mode: &str) -> Option<&Failure> {
        self.failures.get(mode)
    }

    pub fn is_armed(&self, mode: &str) -> bool {
        self.failures.contains_key(mode)
    }

    pub fn len(&self) -> usize {
        self.failures.len()
    }

    pub fn is_empty(&self) -> bool {
        self.failures.is_empty()
    }

    pub fn armed_modes(&self) -> Vec<&str> {
        self.failures.keys().map(String::as_str).collect()
    }

    pub fn magnitude(&self, mode: &str, now_us: u64) -> f64 {
        self.failures.get(mode).map_or(0.0, |f| f.magnitude_at(now_us))
    }

    pub fn is_active(&self, mode: &str, now_us: u64) -> bool {
        self.magnitude(mode, now_us) > 0.0
    }

    pub fn active_modes(&self, now_us: u64) -> Vec<&str> {
        self.failures
            .values()
            .filter(|f| f.magnitude_at(now_us) > 0.0)
            .map(|f| f.mode.as_str())
            .collect()
    }

    pub fn any_active(&self, now_us: u64) -> bool {
        self.failures.values().any(|f| f.magnitude_at(now_us) > 0.0)
    }

    pub fn worst(&self, now_us: u64) -> f64 {
        self.failures
            .values()
            .map(|f| f.magnitude_at(now_us))
            .fold(0.0, f64::max)
    }

    pub fn combined(&self, now_us: u64) -> f64 {
        let mut ok = 1.0;
        for f in self.failures.values() {
            ok *= 1.0 - f.magnitude_at(now_us);
        }
        1.0 - ok
    }

    pub fn unknown_modes(&self, registry: &Registry, part_type: &str) -> Vec<String> {
        self.failures
            .keys()
            .filter(|m| !registry.supports(part_type, m))
            .cloned()
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn immediate_applies_from_arm_time() {
        let mut s = FailureSet::new();
        s.arm_immediate("open", 0.7, 100);
        assert_eq!(s.magnitude("open", 99), 0.0);
        assert_eq!(s.magnitude("open", 100), 0.7);
        assert_eq!(s.magnitude("other", 100), 0.0);
        assert!(s.is_active("open", 500));
    }

    #[test]
    fn magnitude_clamped() {
        let mut s = FailureSet::new();
        s.arm_immediate("a", 3.0, 0);
        s.arm_immediate("b", -1.0, 0);
        s.arm_immediate("c", f64::NAN, 0);
        assert_eq!(s.magnitude("a", 1), 1.0);
        assert_eq!(s.magnitude("b", 1), 0.0);
        assert_eq!(s.magnitude("c", 1), 0.0);
    }

    #[test]
    fn ramp_is_linear_then_holds() {
        let mut s = FailureSet::new();
        s.arm("drift", 0.8, Onset::Ramp { duration_us: 1000 }, 1000);
        assert_eq!(s.magnitude("drift", 1000), 0.0);
        assert!((s.magnitude("drift", 1500) - 0.4).abs() < 1e-12);
        assert_eq!(s.magnitude("drift", 2000), 0.8);
        assert_eq!(s.magnitude("drift", 9000), 0.8);
    }

    #[test]
    fn ramp_zero_duration_is_immediate() {
        let mut s = FailureSet::new();
        s.arm("x", 0.5, Onset::Ramp { duration_us: 0 }, 0);
        assert_eq!(s.magnitude("x", 0), 0.5);
    }

    #[test]
    fn intermittent_deterministic_and_duty_respected() {
        let onset = Onset::Intermittent {
            seed: 99,
            period_us: 1000,
            duty: 0.3,
        };
        let mut a = FailureSet::new();
        let mut b = FailureSet::new();
        a.arm("chafe", 1.0, onset.clone(), 0);
        b.arm("chafe", 1.0, onset, 0);
        let mut on = 0;
        for slot in 0..10000u64 {
            let t = slot * 1000 + 500;
            assert_eq!(a.magnitude("chafe", t), b.magnitude("chafe", t));
            assert_eq!(a.magnitude("chafe", t), a.magnitude("chafe", slot * 1000));
            if a.is_active("chafe", t) {
                on += 1;
            }
        }
        let frac = f64::from(on) / 10000.0;
        assert!((frac - 0.3).abs() < 0.03);
    }

    #[test]
    fn intermittent_extremes() {
        let mut s = FailureSet::new();
        s.arm(
            "never",
            1.0,
            Onset::Intermittent {
                seed: 1,
                period_us: 10,
                duty: 0.0,
            },
            0,
        );
        s.arm(
            "always",
            1.0,
            Onset::Intermittent {
                seed: 1,
                period_us: 0,
                duty: 1.0,
            },
            0,
        );
        for t in 0..200 {
            assert_eq!(s.magnitude("never", t), 0.0);
            assert_eq!(s.magnitude("always", t), 1.0);
        }
    }

    #[test]
    fn latent_waits_for_trigger() {
        let mut s = FailureSet::new();
        s.arm(
            "stuck",
            0.9,
            Onset::Latent {
                trigger: "cold-soak".to_string(),
            },
            0,
        );
        assert_eq!(s.magnitude("stuck", 5000), 0.0);
        assert_eq!(s.trigger("other", 6000), 0);
        assert_eq!(s.trigger("cold-soak", 7000), 1);
        assert_eq!(s.magnitude("stuck", 6999), 0.0);
        assert_eq!(s.magnitude("stuck", 7000), 0.9);
        assert_eq!(s.trigger("cold-soak", 8000), 0);
        assert_eq!(s.get("stuck").unwrap().triggered_at_us, Some(7000));
    }

    #[test]
    fn clear_and_clear_all() {
        let mut s = FailureSet::new();
        s.arm_immediate("a", 1.0, 0);
        s.arm_immediate("b", 1.0, 0);
        assert_eq!(s.len(), 2);
        assert!(s.clear("a"));
        assert!(!s.clear("a"));
        assert!(!s.is_armed("a"));
        s.clear_all();
        assert!(s.is_empty());
        assert!(!s.any_active(10));
    }

    #[test]
    fn rearm_replaces() {
        let mut s = FailureSet::new();
        s.arm_immediate("a", 0.2, 0);
        s.arm_immediate("a", 0.6, 0);
        assert_eq!(s.len(), 1);
        assert_eq!(s.magnitude("a", 1), 0.6);
    }

    #[test]
    fn aggregates() {
        let mut s = FailureSet::new();
        s.arm_immediate("a", 0.5, 0);
        s.arm_immediate("b", 0.5, 0);
        s.arm("c", 1.0, Onset::Ramp { duration_us: 100 }, 0);
        assert_eq!(s.armed_modes(), vec!["a", "b", "c"]);
        assert_eq!(s.active_modes(0), vec!["a", "b"]);
        assert_eq!(s.worst(0), 0.5);
        assert!((s.combined(0) - 0.75).abs() < 1e-12);
        assert_eq!(s.worst(100), 1.0);
        assert!(s.any_active(0));
    }

    #[test]
    fn registry_checked_arm() {
        let mut r = Registry::new();
        r.add("relay", "stuck-open", &["contact-voltage"]).unwrap();
        let mut s = FailureSet::new();
        s.arm_checked(&r, "relay", "stuck-open", 1.0, Onset::Immediate, 0)
            .unwrap();
        let e = s
            .arm_checked(&r, "relay", "bogus", 1.0, Onset::Immediate, 0)
            .unwrap_err();
        assert!(e.to_string().contains("bogus"));
        assert!(s.unknown_modes(&r, "relay").is_empty());
        s.arm_immediate("ghost", 1.0, 0);
        assert_eq!(s.unknown_modes(&r, "relay"), vec!["ghost".to_string()]);
    }

    #[test]
    fn mode_builder() {
        let m = FailureMode::new("a").described("d");
        assert!(!m.is_observable());
        let m = m.observable_via("volts");
        assert!(m.is_observable());
        assert_eq!(m.description, "d");
    }
}
