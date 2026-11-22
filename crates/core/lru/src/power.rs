#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PowerKind {
    Dc28,
    Ac115,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct PowerInput {
    pub kind: PowerKind,
    pub voltage_v: f64,
    pub frequency_hz: f64,
    pub available: bool,
}

impl PowerInput {
    pub fn dc28(voltage_v: f64) -> Self {
        Self {
            kind: PowerKind::Dc28,
            voltage_v,
            frequency_hz: 0.0,
            available: true,
        }
    }

    pub fn ac115(voltage_v: f64, frequency_hz: f64) -> Self {
        Self {
            kind: PowerKind::Ac115,
            voltage_v,
            frequency_hz,
            available: true,
        }
    }

    pub fn dead(kind: PowerKind) -> Self {
        Self {
            kind,
            voltage_v: 0.0,
            frequency_hz: 0.0,
            available: false,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum InputState {
    Ok,
    UnderVoltage,
    FrequencyOutOfRange,
    Dead,
}

#[derive(Clone, Debug)]
pub struct PowerConfig {
    pub dc_under_voltage_v: f64,
    pub ac_under_voltage_v: f64,
    pub ac_min_frequency_hz: f64,
    pub ac_max_frequency_hz: f64,
    pub capacitance_f: f64,
    pub full_voltage_v: f64,
    pub min_voltage_v: f64,
    pub efficiency: f64,
    pub rated_life_hours: f64,
    pub reference_temp_c: f64,
    pub recharge_us: u64,
}

impl Default for PowerConfig {
    fn default() -> Self {
        Self {
            dc_under_voltage_v: 18.0,
            ac_under_voltage_v: 90.0,
            ac_min_frequency_hz: 360.0,
            ac_max_frequency_hz: 800.0,
            capacitance_f: 0.01,
            full_voltage_v: 28.0,
            min_voltage_v: 12.0,
            efficiency: 0.85,
            rated_life_hours: 50_000.0,
            reference_temp_c: 40.0,
            recharge_us: 500_000,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PowerStatus {
    Good,
    HoldUp,
    Lost,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PowerEvent {
    Gap { start_us: u64, duration_us: u64 },
    PowerLost { at_us: u64 },
    FrequencyExcursion { at_us: u64 },
    UnderVoltage { at_us: u64 },
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct PowerStats {
    pub dips: u32,
    pub resets: u32,
    pub longest_gap_us: u64,
    pub frequency_excursions: u32,
    pub undervoltage_events: u32,
}

#[derive(Clone, Debug)]
pub struct PowerStage {
    cfg: PowerConfig,
    age_equiv_hours: f64,
    health: f64,
    charge: f64,
    last_us: Option<u64>,
    gap_start_us: Option<u64>,
    gap_holdup_us: u64,
    status: PowerStatus,
    bus_voltage_v: f64,
    stats: PowerStats,
    events: Vec<PowerEvent>,
    prev_freq_bad: bool,
    prev_uv: bool,
}

impl PowerStage {
    pub fn new(cfg: PowerConfig) -> Self {
        Self {
            cfg,
            age_equiv_hours: 0.0,
            health: 1.0,
            charge: 0.0,
            last_us: None,
            gap_start_us: None,
            gap_holdup_us: 0,
            status: PowerStatus::Lost,
            bus_voltage_v: 0.0,
            stats: PowerStats::default(),
            events: Vec::new(),
            prev_freq_bad: false,
            prev_uv: false,
        }
    }

    pub fn config(&self) -> &PowerConfig {
        &self.cfg
    }

    pub fn status(&self) -> PowerStatus {
        self.status
    }

    pub fn bus_voltage_v(&self) -> f64 {
        self.bus_voltage_v
    }

    pub fn stats(&self) -> &PowerStats {
        &self.stats
    }

    pub fn events(&self) -> &[PowerEvent] {
        &self.events
    }

    pub fn clear_events(&mut self) {
        self.events.clear();
    }

    pub fn charge(&self) -> f64 {
        self.charge
    }

    pub fn age_equivalent_hours(&self) -> f64 {
        self.age_equiv_hours
    }

    pub fn set_age_equivalent_hours(&mut self, hours: f64) {
        self.age_equiv_hours = hours.max(0.0);
    }

    pub fn set_health(&mut self, health: f64) {
        self.health = health.clamp(0.0, 1.0);
    }

    pub fn health(&self) -> f64 {
        self.health
    }

    pub fn capacity_fraction(&self) -> f64 {
        let ratio = self.age_equiv_hours / self.cfg.rated_life_hours;
        (1.0 - 0.2 * ratio).max(0.3) * self.health
    }

    pub fn aging_acceleration(&self, temp_c: f64) -> f64 {
        2f64.powf((temp_c - self.cfg.reference_temp_c) / 10.0)
    }

    pub fn full_holdup_us(&self, load_w: f64, temp_c: f64) -> u64 {
        if load_w <= 0.0 {
            return u64::MAX / 4;
        }
        let cold = if temp_c < 0.0 {
            (1.0 + temp_c * 0.01).max(0.5)
        } else {
            1.0
        };
        let c = &self.cfg;
        let energy = 0.5
            * c.capacitance_f
            * (c.full_voltage_v * c.full_voltage_v - c.min_voltage_v * c.min_voltage_v)
            * c.efficiency
            * self.capacity_fraction()
            * cold;
        (energy / load_w * 1.0e6).max(0.0) as u64
    }

    pub fn input_state(&self, input: &PowerInput) -> InputState {
        if !input.available {
            return InputState::Dead;
        }
        match input.kind {
            PowerKind::Dc28 => {
                if input.voltage_v < self.cfg.dc_under_voltage_v {
                    InputState::UnderVoltage
                } else {
                    InputState::Ok
                }
            }
            PowerKind::Ac115 => {
                if input.voltage_v < self.cfg.ac_under_voltage_v {
                    InputState::UnderVoltage
                } else if input.frequency_hz < self.cfg.ac_min_frequency_hz
                    || input.frequency_hz > self.cfg.ac_max_frequency_hz
                {
                    InputState::FrequencyOutOfRange
                } else {
                    InputState::Ok
                }
            }
        }
    }

    pub fn step(&mut self, now_us: u64, inputs: &[PowerInput], temp_c: f64, load_w: f64) -> PowerStatus {
        let dt = self.last_us.map_or(0, |l| now_us.saturating_sub(l));
        self.last_us = Some(now_us);
        if self.status != PowerStatus::Lost {
            self.age_equiv_hours += dt as f64 / 3.6e9 * self.aging_acceleration(temp_c);
        }
        let mut best: Option<f64> = None;
        let mut freq_bad = false;
        let mut uv = false;
        for input in inputs {
            match self.input_state(input) {
                InputState::Ok => {
                    best = Some(best.map_or(input.voltage_v, |b: f64| b.max(input.voltage_v)));
                }
                InputState::FrequencyOutOfRange => freq_bad = true,
                InputState::UnderVoltage => uv = true,
                InputState::Dead => {}
            }
        }
        if freq_bad && !self.prev_freq_bad {
            self.stats.frequency_excursions += 1;
            self.events.push(PowerEvent::FrequencyExcursion { at_us: now_us });
        }
        if uv && !self.prev_uv {
            self.stats.undervoltage_events += 1;
            self.events.push(PowerEvent::UnderVoltage { at_us: now_us });
        }
        self.prev_freq_bad = freq_bad;
        self.prev_uv = uv;
        match best {
            Some(v) => {
                self.bus_voltage_v = v;
                if let Some(start) = self.gap_start_us.take() {
                    let duration = now_us - start;
                    self.stats.longest_gap_us = self.stats.longest_gap_us.max(duration);
                    if self.status != PowerStatus::Lost {
                        self.stats.dips += 1;
                    }
                    self.events.push(PowerEvent::Gap {
                        start_us: start,
                        duration_us: duration,
                    });
                    self.charge = self.charge.min(1.0);
                }
                if self.status == PowerStatus::Lost {
                    self.charge = 0.0;
                } else if self.charge < 1.0 {
                    let rate = if self.cfg.recharge_us == 0 {
                        1.0
                    } else {
                        dt as f64 / self.cfg.recharge_us as f64
                    };
                    self.charge = (self.charge + rate).min(1.0);
                }
                if self.status == PowerStatus::Lost && self.cfg.recharge_us == 0 {
                    self.charge = 1.0;
                }
                self.status = PowerStatus::Good;
            }
            None => {
                self.bus_voltage_v = 0.0;
                if self.status == PowerStatus::Lost {
                    return self.status;
                }
                if self.gap_start_us.is_none() {
                    self.gap_start_us = Some(now_us);
                    let full = self.full_holdup_us(load_w, temp_c);
                    self.gap_holdup_us = (full as f64 * self.charge) as u64;
                }
                let start = self.gap_start_us.unwrap_or(now_us);
                let elapsed = now_us - start;
                if elapsed > self.gap_holdup_us {
                    self.status = PowerStatus::Lost;
                    self.stats.resets += 1;
                    self.stats.longest_gap_us = self.stats.longest_gap_us.max(elapsed);
                    self.events.push(PowerEvent::PowerLost {
                        at_us: start + self.gap_holdup_us,
                    });
                    self.charge = 0.0;
                } else {
                    self.status = PowerStatus::HoldUp;
                    let remaining = 1.0 - elapsed as f64 / self.gap_holdup_us.max(1) as f64;
                    self.charge = self.charge.min(remaining.max(0.0));
                }
            }
        }
        self.status
    }

    pub fn holdup_remaining_us(&self, now_us: u64) -> Option<u64> {
        self.gap_start_us
            .map(|s| (s + self.gap_holdup_us).saturating_sub(now_us))
    }
}
