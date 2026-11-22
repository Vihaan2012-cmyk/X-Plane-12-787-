use core::fmt;
use core::ops::{Add, AddAssign, Sub};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct SimTime(u64);

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct SimDuration(u64);

pub(crate) const MICROS_PER_SEC: u64 = 1_000_000;

fn secs_f64_to_micros(s: f64) -> Option<u64> {
    if !s.is_finite() || s < 0.0 {
        return None;
    }
    let us = (s * 1.0e6).round();
    if us >= 18_446_744_073_709_551_615.0 {
        None
    } else {
        Some(us as u64)
    }
}

impl SimDuration {
    pub const ZERO: SimDuration = SimDuration(0);
    pub const MAX: SimDuration = SimDuration(u64::MAX);

    pub const fn from_micros(us: u64) -> Self {
        SimDuration(us)
    }

    pub const fn from_millis(ms: u64) -> Self {
        SimDuration(ms.saturating_mul(1_000))
    }

    pub const fn from_secs(s: u64) -> Self {
        SimDuration(s.saturating_mul(MICROS_PER_SEC))
    }

    pub const fn from_minutes(m: u64) -> Self {
        SimDuration::from_secs(m.saturating_mul(60))
    }

    pub const fn from_hours(h: u64) -> Self {
        SimDuration::from_secs(h.saturating_mul(3_600))
    }

    pub const fn from_days(d: u64) -> Self {
        SimDuration::from_secs(d.saturating_mul(86_400))
    }

    pub fn from_secs_f64(s: f64) -> Option<Self> {
        secs_f64_to_micros(s).map(SimDuration)
    }

    pub const fn as_micros(self) -> u64 {
        self.0
    }

    pub const fn as_millis(self) -> u64 {
        self.0 / 1_000
    }

    pub const fn as_secs(self) -> u64 {
        self.0 / MICROS_PER_SEC
    }

    pub const fn subsec_micros(self) -> u32 {
        (self.0 % MICROS_PER_SEC) as u32
    }

    pub fn as_secs_f64(self) -> f64 {
        self.0 as f64 / 1.0e6
    }

    pub const fn is_zero(self) -> bool {
        self.0 == 0
    }

    pub const fn checked_add(self, rhs: SimDuration) -> Option<SimDuration> {
        match self.0.checked_add(rhs.0) {
            Some(v) => Some(SimDuration(v)),
            None => None,
        }
    }

    pub const fn checked_sub(self, rhs: SimDuration) -> Option<SimDuration> {
        match self.0.checked_sub(rhs.0) {
            Some(v) => Some(SimDuration(v)),
            None => None,
        }
    }

    pub const fn checked_mul(self, n: u64) -> Option<SimDuration> {
        match self.0.checked_mul(n) {
            Some(v) => Some(SimDuration(v)),
            None => None,
        }
    }

    pub const fn checked_div(self, n: u64) -> Option<SimDuration> {
        match self.0.checked_div(n) {
            Some(v) => Some(SimDuration(v)),
            None => None,
        }
    }

    pub const fn checked_rem(self, rhs: SimDuration) -> Option<SimDuration> {
        match self.0.checked_rem(rhs.0) {
            Some(v) => Some(SimDuration(v)),
            None => None,
        }
    }

    pub const fn saturating_add(self, rhs: SimDuration) -> SimDuration {
        SimDuration(self.0.saturating_add(rhs.0))
    }

    pub const fn saturating_sub(self, rhs: SimDuration) -> SimDuration {
        SimDuration(self.0.saturating_sub(rhs.0))
    }

    pub const fn saturating_mul(self, n: u64) -> SimDuration {
        SimDuration(self.0.saturating_mul(n))
    }

    pub const fn div_duration(self, rhs: SimDuration) -> Option<u64> {
        self.0.checked_div(rhs.0)
    }
}

impl SimTime {
    pub const ZERO: SimTime = SimTime(0);
    pub const MAX: SimTime = SimTime(u64::MAX);

    pub const fn from_micros(us: u64) -> Self {
        SimTime(us)
    }

    pub const fn from_millis(ms: u64) -> Self {
        SimTime(ms.saturating_mul(1_000))
    }

    pub const fn from_secs(s: u64) -> Self {
        SimTime(s.saturating_mul(MICROS_PER_SEC))
    }

    pub fn from_secs_f64(s: f64) -> Option<Self> {
        secs_f64_to_micros(s).map(SimTime)
    }

    pub const fn as_micros(self) -> u64 {
        self.0
    }

    pub fn as_secs_f64(self) -> f64 {
        self.0 as f64 / 1.0e6
    }

    pub const fn since_start(self) -> SimDuration {
        SimDuration(self.0)
    }

    pub const fn checked_add(self, d: SimDuration) -> Option<SimTime> {
        match self.0.checked_add(d.0) {
            Some(v) => Some(SimTime(v)),
            None => None,
        }
    }

    pub const fn checked_sub(self, d: SimDuration) -> Option<SimTime> {
        match self.0.checked_sub(d.0) {
            Some(v) => Some(SimTime(v)),
            None => None,
        }
    }

    pub const fn checked_duration_since(self, earlier: SimTime) -> Option<SimDuration> {
        match self.0.checked_sub(earlier.0) {
            Some(v) => Some(SimDuration(v)),
            None => None,
        }
    }

    pub const fn saturating_add(self, d: SimDuration) -> SimTime {
        SimTime(self.0.saturating_add(d.0))
    }

    pub const fn saturating_sub(self, d: SimDuration) -> SimTime {
        SimTime(self.0.saturating_sub(d.0))
    }

    pub const fn saturating_duration_since(self, earlier: SimTime) -> SimDuration {
        SimDuration(self.0.saturating_sub(earlier.0))
    }

    pub fn signed_micros_since(self, other: SimTime) -> i128 {
        i128::from(self.0) - i128::from(other.0)
    }
}

impl Add<SimDuration> for SimTime {
    type Output = SimTime;
    fn add(self, rhs: SimDuration) -> SimTime {
        SimTime(self.0.saturating_add(rhs.0))
    }
}

impl AddAssign<SimDuration> for SimTime {
    fn add_assign(&mut self, rhs: SimDuration) {
        *self = *self + rhs;
    }
}

impl Sub<SimTime> for SimTime {
    type Output = SimDuration;
    fn sub(self, rhs: SimTime) -> SimDuration {
        self.saturating_duration_since(rhs)
    }
}

impl Add for SimDuration {
    type Output = SimDuration;
    fn add(self, rhs: SimDuration) -> SimDuration {
        self.saturating_add(rhs)
    }
}

impl AddAssign for SimDuration {
    fn add_assign(&mut self, rhs: SimDuration) {
        *self = *self + rhs;
    }
}

impl Sub for SimDuration {
    type Output = SimDuration;
    fn sub(self, rhs: SimDuration) -> SimDuration {
        self.saturating_sub(rhs)
    }
}

fn fmt_micros(f: &mut fmt::Formatter<'_>, us: u64) -> fmt::Result {
    write!(f, "{}.{:06}s", us / MICROS_PER_SEC, us % MICROS_PER_SEC)
}

impl fmt::Display for SimTime {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "T+")?;
        fmt_micros(f, self.0)
    }
}

impl fmt::Display for SimDuration {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt_micros(f, self.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn constructors_are_exact() {
        assert_eq!(SimDuration::from_secs(3).as_micros(), 3_000_000);
        assert_eq!(SimDuration::from_millis(7).as_micros(), 7_000);
        assert_eq!(SimDuration::from_days(1).as_secs(), 86_400);
        assert_eq!(SimDuration::from_hours(2).as_secs(), 7_200);
        assert_eq!(SimDuration::from_minutes(2).as_secs(), 120);
        assert_eq!(SimTime::from_millis(5).as_micros(), 5_000);
    }

    #[test]
    fn seconds_round_trip() {
        let d = SimDuration::from_secs_f64(1.234_567).unwrap();
        assert_eq!(d.as_micros(), 1_234_567);
        assert_eq!(d.as_secs_f64(), 1.234_567);
        assert_eq!(SimTime::from_secs_f64(0.5).unwrap().as_micros(), 500_000);
    }

    #[test]
    fn invalid_seconds_rejected() {
        assert!(SimDuration::from_secs_f64(-1.0).is_none());
        assert!(SimDuration::from_secs_f64(f64::NAN).is_none());
        assert!(SimDuration::from_secs_f64(f64::INFINITY).is_none());
        assert!(SimDuration::from_secs_f64(1.0e300).is_none());
    }

    #[test]
    fn checked_ops_detect_overflow() {
        assert!(SimTime::MAX.checked_add(SimDuration::from_micros(1)).is_none());
        assert!(SimTime::ZERO.checked_sub(SimDuration::from_micros(1)).is_none());
        assert!(SimDuration::MAX.checked_mul(2).is_none());
        assert!(SimDuration::from_micros(1).checked_div(0).is_none());
        assert!(
            SimDuration::from_micros(1)
                .checked_rem(SimDuration::ZERO)
                .is_none()
        );
        assert!(
            SimDuration::ZERO
                .checked_sub(SimDuration::from_micros(1))
                .is_none()
        );
        assert_eq!(
            SimTime::from_micros(10).checked_add(SimDuration::from_micros(5)),
            Some(SimTime::from_micros(15))
        );
        assert_eq!(
            SimTime::from_micros(10).checked_duration_since(SimTime::from_micros(4)),
            Some(SimDuration::from_micros(6))
        );
        assert_eq!(
            SimTime::from_micros(4).checked_duration_since(SimTime::from_micros(10)),
            None
        );
    }

    #[test]
    fn saturating_ops_clamp() {
        assert_eq!(SimTime::MAX + SimDuration::from_micros(9), SimTime::MAX);
        assert_eq!(
            SimTime::from_micros(3) - SimTime::from_micros(9),
            SimDuration::ZERO
        );
        assert_eq!(SimDuration::MAX.saturating_mul(3), SimDuration::MAX);
        assert_eq!(
            SimTime::ZERO.saturating_sub(SimDuration::from_micros(1)),
            SimTime::ZERO
        );
    }

    #[test]
    fn display_format() {
        assert_eq!(SimTime::from_micros(12_345_678).to_string(), "T+12.345678s");
        assert_eq!(SimDuration::from_micros(5).to_string(), "0.000005s");
    }

    #[test]
    fn signed_difference() {
        assert_eq!(
            SimTime::from_micros(3).signed_micros_since(SimTime::from_micros(10)),
            -7
        );
    }

    #[test]
    fn duration_division() {
        let a = SimDuration::from_millis(10);
        assert_eq!(a.div_duration(SimDuration::from_millis(3)), Some(3));
        assert_eq!(a.div_duration(SimDuration::ZERO), None);
        assert_eq!(a.subsec_micros(), 10_000);
        assert!(!a.is_zero());
    }
}
