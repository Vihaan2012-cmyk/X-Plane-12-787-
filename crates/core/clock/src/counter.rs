use crate::time::{SimDuration, SimTime};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OverflowMode {
    Wrap,
    Saturate,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CounterConfig {
    pub bits: u8,
    pub tick: SimDuration,
    pub mode: OverflowMode,
}

impl CounterConfig {
    pub fn gcu_248_day() -> Self {
        CounterConfig {
            bits: 31,
            tick: SimDuration::from_millis(10),
            mode: OverflowMode::Wrap,
        }
    }

    pub fn for_overflow_days(bits: u8, days: u64, mode: OverflowMode) -> Option<Self> {
        if bits == 0 || bits > 63 {
            return None;
        }
        let total = u128::from(SimDuration::from_days(days).as_micros());
        let tick = u64::try_from((total + (1u128 << (bits - 1))) >> bits).ok()?;
        if tick == 0 {
            return None;
        }
        Some(CounterConfig {
            bits,
            tick: SimDuration::from_micros(tick),
            mode,
        })
    }

    pub fn ccs_51_day() -> Self {
        Self::for_overflow_days(32, 51, OverflowMode::Wrap).unwrap_or(CounterConfig {
            bits: 32,
            tick: SimDuration::from_micros(1_026),
            mode: OverflowMode::Wrap,
        })
    }
}

#[derive(Debug, Clone)]
pub struct PoweredCounter {
    config: CounterConfig,
    accumulated: u64,
    powered_since: Option<SimTime>,
}

impl PoweredCounter {
    pub fn new(config: CounterConfig) -> Option<Self> {
        if config.bits == 0 || config.bits > 64 || config.tick.is_zero() {
            return None;
        }
        Some(PoweredCounter {
            config,
            accumulated: 0,
            powered_since: None,
        })
    }

    pub fn config(&self) -> CounterConfig {
        self.config
    }

    pub fn is_powered(&self) -> bool {
        self.powered_since.is_some()
    }

    pub fn power_on(&mut self, at: SimTime) {
        if self.powered_since.is_none() {
            self.powered_since = Some(at);
        }
    }

    pub fn power_off(&mut self, at: SimTime) {
        if let Some(since) = self.powered_since.take() {
            self.accumulated = self
                .accumulated
                .saturating_add(at.saturating_duration_since(since).as_micros());
        }
    }

    pub fn reset(&mut self, at: SimTime) {
        self.accumulated = 0;
        if self.powered_since.is_some() {
            self.powered_since = Some(at);
        }
    }

    pub fn restore(&mut self, powered_total: SimDuration) {
        self.accumulated = powered_total.as_micros();
    }

    pub fn powered_time(&self, at: SimTime) -> SimDuration {
        let live = match self.powered_since {
            Some(since) => at.saturating_duration_since(since).as_micros(),
            None => 0,
        };
        SimDuration::from_micros(self.accumulated.saturating_add(live))
    }

    pub fn raw_ticks(&self, at: SimTime) -> u64 {
        self.powered_time(at).as_micros() / self.config.tick.as_micros()
    }

    fn mask(&self) -> u64 {
        if self.config.bits >= 64 {
            u64::MAX
        } else {
            (1u64 << self.config.bits) - 1
        }
    }

    pub fn max_value(&self) -> u64 {
        self.mask()
    }

    pub fn value(&self, at: SimTime) -> u64 {
        let raw = self.raw_ticks(at);
        match self.config.mode {
            OverflowMode::Wrap => raw & self.mask(),
            OverflowMode::Saturate => raw.min(self.mask()),
        }
    }

    pub fn overflow_count(&self, at: SimTime) -> u64 {
        if self.config.bits >= 64 {
            0
        } else {
            self.raw_ticks(at) >> self.config.bits
        }
    }

    pub fn has_overflowed(&self, at: SimTime) -> bool {
        self.overflow_count(at) > 0
    }

    pub fn overflow_period(&self) -> Option<SimDuration> {
        let total = u128::from(self.config.tick.as_micros()) << self.config.bits;
        u64::try_from(total).ok().map(SimDuration::from_micros)
    }

    pub fn time_to_next_overflow(&self, at: SimTime) -> Option<SimDuration> {
        let period = u128::from(self.overflow_period()?.as_micros());
        let used = u128::from(self.powered_time(at).as_micros());
        let next = (used / period + 1) * period;
        u64::try_from(next - used).ok().map(SimDuration::from_micros)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn cfg(bits: u8, mode: OverflowMode) -> CounterConfig {
        CounterConfig {
            bits,
            tick: SimDuration::from_millis(1),
            mode,
        }
    }

    #[test]
    fn counts_only_while_powered() {
        let mut c = PoweredCounter::new(cfg(16, OverflowMode::Wrap)).unwrap();
        assert_eq!(c.value(SimTime::from_secs(100)), 0);
        c.power_on(SimTime::from_secs(10));
        assert_eq!(c.value(SimTime::from_secs(12)), 2_000);
        c.power_off(SimTime::from_secs(13));
        assert_eq!(c.value(SimTime::from_secs(500)), 3_000);
        c.power_on(SimTime::from_secs(600));
        assert_eq!(c.value(SimTime::from_secs(601)), 4_000);
        assert!(c.is_powered());
        assert_eq!(c.powered_time(SimTime::from_secs(601)), SimDuration::from_secs(4));
    }

    #[test]
    fn repeated_power_calls_are_idempotent() {
        let mut c = PoweredCounter::new(cfg(16, OverflowMode::Wrap)).unwrap();
        c.power_on(SimTime::from_secs(1));
        c.power_on(SimTime::from_secs(5));
        c.power_off(SimTime::from_secs(6));
        c.power_off(SimTime::from_secs(60));
        assert_eq!(c.powered_time(SimTime::from_secs(100)), SimDuration::from_secs(5));
    }

    #[test]
    fn wrap_at_bit_width() {
        let mut c = PoweredCounter::new(cfg(8, OverflowMode::Wrap)).unwrap();
        c.power_on(SimTime::ZERO);
        assert_eq!(c.value(SimTime::from_millis(255)), 255);
        assert!(!c.has_overflowed(SimTime::from_millis(255)));
        assert_eq!(c.value(SimTime::from_millis(256)), 0);
        assert!(c.has_overflowed(SimTime::from_millis(256)));
        assert_eq!(c.value(SimTime::from_millis(300)), 44);
        assert_eq!(c.overflow_count(SimTime::from_millis(256 * 3 + 1)), 3);
        assert_eq!(c.max_value(), 255);
    }

    #[test]
    fn saturate_holds_max() {
        let mut c = PoweredCounter::new(cfg(8, OverflowMode::Saturate)).unwrap();
        c.power_on(SimTime::ZERO);
        assert_eq!(c.value(SimTime::from_millis(1_000)), 255);
        assert_eq!(c.overflow_count(SimTime::from_millis(1_000)), 3);
    }

    #[test]
    fn gcu_counter_overflows_at_248_days() {
        let mut c = PoweredCounter::new(CounterConfig::gcu_248_day()).unwrap();
        c.power_on(SimTime::ZERO);
        let day = SimDuration::from_days(1).as_micros();
        assert!(!c.has_overflowed(SimTime::from_micros(248 * day)));
        assert!(c.has_overflowed(SimTime::from_micros(249 * day)));
        let p = c.overflow_period().unwrap();
        assert_eq!(p.as_micros(), (1u64 << 31) * 10_000);
        assert!(p.as_secs() / 86_400 == 248);
    }

    #[test]
    fn ccs_counter_overflows_at_51_days() {
        let mut c = PoweredCounter::new(CounterConfig::ccs_51_day()).unwrap();
        c.power_on(SimTime::ZERO);
        let day = SimDuration::from_days(1).as_micros();
        assert!(!c.has_overflowed(SimTime::from_micros(50 * day)));
        assert!(c.has_overflowed(SimTime::from_micros(52 * day)));
        let days = c.overflow_period().unwrap().as_secs() as f64 / 86_400.0;
        assert!((days - 51.0).abs() < 0.05);
    }

    #[test]
    fn powered_gap_does_not_count_toward_overflow() {
        let mut c = PoweredCounter::new(CounterConfig::gcu_248_day()).unwrap();
        c.power_on(SimTime::ZERO);
        c.power_off(SimTime::from_micros(200 * SimDuration::from_days(1).as_micros()));
        let later = SimTime::from_micros(1_000 * SimDuration::from_days(1).as_micros());
        assert!(!c.has_overflowed(later));
        c.power_on(later);
        let t = later + SimDuration::from_days(60);
        assert!(c.has_overflowed(t));
    }

    #[test]
    fn sixty_four_bit_never_overflows() {
        let mut c = PoweredCounter::new(cfg(64, OverflowMode::Wrap)).unwrap();
        c.power_on(SimTime::ZERO);
        assert_eq!(c.overflow_count(SimTime::from_secs(1_000_000)), 0);
        assert_eq!(c.max_value(), u64::MAX);
        assert!(c.overflow_period().is_none());
        assert!(c.time_to_next_overflow(SimTime::ZERO).is_none());
    }

    #[test]
    fn time_to_overflow_counts_down() {
        let mut c = PoweredCounter::new(cfg(8, OverflowMode::Wrap)).unwrap();
        c.power_on(SimTime::ZERO);
        assert_eq!(
            c.time_to_next_overflow(SimTime::ZERO),
            Some(SimDuration::from_millis(256))
        );
        assert_eq!(
            c.time_to_next_overflow(SimTime::from_millis(200)),
            Some(SimDuration::from_millis(56))
        );
        assert_eq!(
            c.time_to_next_overflow(SimTime::from_millis(256)),
            Some(SimDuration::from_millis(256))
        );
    }

    #[test]
    fn restore_and_reset() {
        let mut c = PoweredCounter::new(cfg(16, OverflowMode::Wrap)).unwrap();
        c.restore(SimDuration::from_secs(40));
        c.power_on(SimTime::from_secs(5));
        assert_eq!(c.value(SimTime::from_secs(6)), 41_000);
        c.reset(SimTime::from_secs(6));
        assert_eq!(c.value(SimTime::from_secs(7)), 1_000);
        c.power_off(SimTime::from_secs(7));
        c.reset(SimTime::from_secs(8));
        assert_eq!(c.value(SimTime::from_secs(9)), 0);
    }

    #[test]
    fn invalid_configs_rejected() {
        assert!(PoweredCounter::new(cfg(0, OverflowMode::Wrap)).is_none());
        assert!(PoweredCounter::new(cfg(65, OverflowMode::Wrap)).is_none());
        let zero_tick = CounterConfig {
            bits: 8,
            tick: SimDuration::ZERO,
            mode: OverflowMode::Wrap,
        };
        assert!(PoweredCounter::new(zero_tick).is_none());
        assert!(CounterConfig::for_overflow_days(0, 1, OverflowMode::Wrap).is_none());
        assert!(CounterConfig::for_overflow_days(63, 1, OverflowMode::Wrap).is_none());
    }
}
