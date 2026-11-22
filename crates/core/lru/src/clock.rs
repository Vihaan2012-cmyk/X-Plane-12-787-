#[derive(Clone, Debug)]
pub struct ClockConfig {
    pub offset_ppm: f64,
    pub turnover_temp_c: f64,
    pub temp_coeff_ppm_per_c2: f64,
    pub aging_ppm_per_year: f64,
}

impl Default for ClockConfig {
    fn default() -> Self {
        Self {
            offset_ppm: 0.0,
            turnover_temp_c: 25.0,
            temp_coeff_ppm_per_c2: -0.034,
            aging_ppm_per_year: 1.0,
        }
    }
}

const HOURS_PER_YEAR: f64 = 8766.0;

#[derive(Clone, Debug)]
pub struct BoxClock {
    cfg: ClockConfig,
    local_us: f64,
    last_true_us: Option<u64>,
    age_hours: f64,
    synced: bool,
    stuck: bool,
    last_ppm: f64,
}

impl BoxClock {
    pub fn new(cfg: ClockConfig) -> Self {
        Self {
            cfg,
            local_us: 0.0,
            last_true_us: None,
            age_hours: 0.0,
            synced: false,
            stuck: false,
            last_ppm: 0.0,
        }
    }

    pub fn rate_error_ppm(&self, temp_c: f64) -> f64 {
        let dt = temp_c - self.cfg.turnover_temp_c;
        self.cfg.offset_ppm
            + self.cfg.temp_coeff_ppm_per_c2 * dt * dt
            + self.cfg.aging_ppm_per_year * self.age_hours / HOURS_PER_YEAR
    }

    pub fn set_age_hours(&mut self, hours: f64) {
        self.age_hours = hours.max(0.0);
    }

    pub fn age_hours(&self) -> f64 {
        self.age_hours
    }

    pub fn set_stuck(&mut self, stuck: bool) {
        self.stuck = stuck;
    }

    pub fn power_up(&mut self, now_us: u64) {
        self.local_us = 0.0;
        self.last_true_us = Some(now_us);
        self.synced = false;
    }

    pub fn step(&mut self, now_us: u64, temp_c: f64) {
        let Some(last) = self.last_true_us else {
            self.last_true_us = Some(now_us);
            return;
        };
        let dt = now_us.saturating_sub(last) as f64;
        self.last_true_us = Some(now_us);
        self.age_hours += dt / 3.6e9;
        self.last_ppm = self.rate_error_ppm(temp_c);
        if !self.stuck {
            self.local_us += dt * (1.0 + self.last_ppm * 1.0e-6);
        }
    }

    pub fn last_rate_ppm(&self) -> f64 {
        self.last_ppm
    }

    pub fn local_time_us(&self) -> u64 {
        self.local_us.max(0.0) as u64
    }

    pub fn error_us(&self, true_elapsed_us: u64) -> f64 {
        self.local_us - true_elapsed_us as f64
    }

    pub fn resync(&mut self, reference_us: u64) {
        self.local_us = reference_us as f64;
        self.synced = true;
    }

    pub fn is_synced(&self) -> bool {
        self.synced
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum OverflowAction {
    Wrap,
    FailBox,
    ResetBox,
}

#[derive(Clone, Debug)]
pub struct PoweredCounter {
    width_bits: u32,
    tick_us: u64,
    ticks: u64,
    remainder_us: u64,
    overflows: u32,
}

impl PoweredCounter {
    pub fn new(width_bits: u32, tick_us: u64) -> Self {
        Self {
            width_bits: width_bits.clamp(1, 64),
            tick_us: tick_us.max(1),
            ticks: 0,
            remainder_us: 0,
            overflows: 0,
        }
    }

    fn mask(&self) -> u64 {
        if self.width_bits >= 64 {
            u64::MAX
        } else {
            (1u64 << self.width_bits) - 1
        }
    }

    pub fn advance(&mut self, dt_us: u64) -> u32 {
        let total = self.remainder_us as u128 + dt_us as u128;
        let new_ticks = total / self.tick_us as u128;
        self.remainder_us = (total % self.tick_us as u128) as u64;
        let before = self.ticks as u128;
        let after = before + new_ticks;
        let modulus = self.mask() as u128 + 1;
        let wraps = (after / modulus) as u32;
        self.ticks = (after % modulus) as u64;
        self.overflows += wraps;
        wraps
    }

    pub fn value(&self) -> u64 {
        self.ticks
    }

    pub fn overflows(&self) -> u32 {
        self.overflows
    }

    pub fn width_bits(&self) -> u32 {
        self.width_bits
    }

    pub fn tick_us(&self) -> u64 {
        self.tick_us
    }

    pub fn time_to_overflow_us(&self) -> u64 {
        let modulus = self.mask() as u128 + 1;
        let left = modulus - self.ticks as u128;
        let us = left * self.tick_us as u128 - self.remainder_us as u128;
        us.min(u64::MAX as u128) as u64
    }

    pub fn reset(&mut self) {
        self.ticks = 0;
        self.remainder_us = 0;
    }

    pub fn set_ticks(&mut self, ticks: u64) {
        self.ticks = ticks & self.mask();
    }
}
