pub const CHARGER_CURRENT_LIMIT_A: f64 = 46.0;
pub const CHARGER_OUTPUT_V: f64 = 32.2;
pub const CHARGER_FAIL_DELAY_S: f64 = 1.0;
pub const APPROX_CHARGER_EFFICIENCY: f64 = 0.88;
pub const APPROX_FLOAT_ENTER_A: f64 = 1.5;
pub const APPROX_FLOAT_EXIT_A: f64 = 4.0;
pub const APPROX_INPUT_V_MIN: f64 = 22.0;
pub const CHARGE_TEMP_MAX_C: f64 = 70.0;
pub const CHARGE_TEMP_MIN_C: f64 = -18.0;
pub const FULL_CHARGE_TIME_MAX_US: u64 = 90 * 60 * 1_000_000;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ChargerMode {
    Off,
    ConstantCurrent,
    ConstantVoltage,
    Float,
    Fault,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ChargerParams {
    pub current_limit_a: f64,
    pub output_v: f64,
    pub efficiency: f64,
}

impl Default for ChargerParams {
    fn default() -> Self {
        Self {
            current_limit_a: CHARGER_CURRENT_LIMIT_A,
            output_v: CHARGER_OUTPUT_V,
            efficiency: APPROX_CHARGER_EFFICIENCY,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ChargerInput {
    pub dt_us: u64,
    pub input_v: f64,
    pub enable: bool,
    pub inhibit: bool,
    pub bmu_inhibit: bool,
    pub battery_connected: bool,
    pub battery_emf_v: f64,
    pub battery_r_ohm: f64,
    pub battery_temp_c: f64,
    pub internal_fault: bool,
}

impl Default for ChargerInput {
    fn default() -> Self {
        Self {
            dt_us: 10_000,
            input_v: 28.0,
            enable: true,
            inhibit: false,
            bmu_inhibit: false,
            battery_connected: true,
            battery_emf_v: 0.0,
            battery_r_ohm: 0.05,
            battery_temp_c: 25.0,
            internal_fault: false,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ChargerOutput {
    pub mode: ChargerMode,
    pub charge_current_a: f64,
    pub output_v: f64,
    pub input_power_w: f64,
    pub failed: bool,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct BatteryCharger {
    pub params: ChargerParams,
    mode: ChargerMode,
    t_fail: u64,
    t_charge: u64,
    failed: bool,
}

impl BatteryCharger {
    pub fn new(params: ChargerParams) -> Self {
        Self {
            params,
            mode: ChargerMode::Off,
            t_fail: 0,
            t_charge: 0,
            failed: false,
        }
    }

    pub fn mode(&self) -> ChargerMode {
        self.mode
    }

    pub fn step(&mut self, input: &ChargerInput) -> ChargerOutput {
        let p = self.params;
        if input.internal_fault {
            self.t_fail = self.t_fail.saturating_add(input.dt_us);
            if self.t_fail >= (CHARGER_FAIL_DELAY_S * 1.0e6) as u64 {
                self.failed = true;
            }
        } else {
            self.t_fail = 0;
        }
        let blocked = !input.enable
            || input.inhibit
            || input.bmu_inhibit
            || !input.battery_connected
            || input.input_v < APPROX_INPUT_V_MIN
            || input.battery_temp_c > CHARGE_TEMP_MAX_C
            || input.battery_temp_c < CHARGE_TEMP_MIN_C;
        if self.t_charge >= FULL_CHARGE_TIME_MAX_US {
            self.failed = true;
        }
        if self.failed {
            self.mode = ChargerMode::Fault;
            return ChargerOutput {
                mode: self.mode,
                charge_current_a: 0.0,
                output_v: 0.0,
                input_power_w: 0.0,
                failed: true,
            };
        }
        if blocked {
            self.mode = ChargerMode::Off;
            self.t_charge = 0;
            return ChargerOutput {
                mode: self.mode,
                charge_current_a: 0.0,
                output_v: 0.0,
                input_power_w: 0.0,
                failed: false,
            };
        }
        let r = input.battery_r_ohm.max(1.0e-6);
        let i_cv = ((p.output_v - input.battery_emf_v) / r).max(0.0);
        let (mode, i) = if i_cv >= p.current_limit_a {
            (ChargerMode::ConstantCurrent, p.current_limit_a)
        } else if (self.mode == ChargerMode::Float && i_cv < APPROX_FLOAT_EXIT_A)
            || i_cv < APPROX_FLOAT_ENTER_A
        {
            (ChargerMode::Float, i_cv)
        } else {
            (ChargerMode::ConstantVoltage, i_cv)
        };
        self.mode = mode;
        if mode == ChargerMode::Float {
            self.t_charge = 0;
        } else {
            self.t_charge = self.t_charge.saturating_add(input.dt_us);
        }
        let v_out = (input.battery_emf_v + i * r).min(p.output_v);
        ChargerOutput {
            mode,
            charge_current_a: i,
            output_v: v_out,
            input_power_w: v_out * i / p.efficiency,
            failed: false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::storage::battery::*;

    const NB: [bool; MAX_CELLS] = [false; MAX_CELLS];

    #[test]
    fn cc_then_cv_then_float() {
        let mut b = BatteryPack::new(BatteryParams::new(8, 75.0), 3, 0.2, 25.0);
        let mut c = BatteryCharger::new(ChargerParams::default());
        let mut saw_cc = false;
        let mut saw_cv = false;
        let mut last_i = 0.0;
        let mut cc_end_v = 0.0;
        for _ in 0..4000 {
            let (emf, r) = b.thevenin();
            let o = c.step(&ChargerInput {
                dt_us: 10_000_000,
                battery_emf_v: emf,
                battery_r_ohm: r,
                battery_temp_c: b.temp_c(),
                ..ChargerInput::default()
            });
            match o.mode {
                ChargerMode::ConstantCurrent => {
                    assert!(!saw_cv);
                    assert!((o.charge_current_a - 46.0).abs() < 1e-9);
                    saw_cc = true;
                    cc_end_v = o.output_v;
                }
                ChargerMode::ConstantVoltage => {
                    assert!(saw_cc);
                    assert!(o.charge_current_a <= 46.0);
                    assert!(o.charge_current_a < last_i + 1e-9);
                    saw_cv = true;
                }
                _ => {}
            }
            last_i = o.charge_current_a;
            b.step(10_000_000, -o.charge_current_a, 25.0, NB);
            assert!(o.output_v <= 32.2 + 1e-9);
        }
        assert!(saw_cc && saw_cv);
        assert!(cc_end_v < 32.2);
        assert_eq!(c.mode(), ChargerMode::Float);
        assert!(b.soc_mean() > 0.8);
    }

    #[test]
    fn inhibits_stop_charging() {
        let mut c = BatteryCharger::new(ChargerParams::default());
        let base = ChargerInput {
            battery_emf_v: 28.0,
            ..ChargerInput::default()
        };
        assert!(c.step(&base).charge_current_a > 0.0);
        for i in [
            ChargerInput {
                inhibit: true,
                ..base
            },
            ChargerInput {
                bmu_inhibit: true,
                ..base
            },
            ChargerInput {
                enable: false,
                ..base
            },
            ChargerInput {
                input_v: 10.0,
                ..base
            },
            ChargerInput {
                battery_temp_c: 71.0,
                ..base
            },
            ChargerInput {
                battery_connected: false,
                ..base
            },
        ] {
            let o = c.step(&i);
            assert_eq!(o.mode, ChargerMode::Off);
            assert_eq!(o.charge_current_a, 0.0);
        }
    }

    #[test]
    fn internal_fault_after_1s() {
        let mut c = BatteryCharger::new(ChargerParams::default());
        let i = ChargerInput {
            battery_emf_v: 28.0,
            internal_fault: true,
            ..ChargerInput::default()
        };
        for _ in 0..99 {
            assert!(!c.step(&i).failed);
        }
        let o = c.step(&i);
        assert!(o.failed && o.mode == ChargerMode::Fault && o.charge_current_a == 0.0);
    }

    #[test]
    fn input_power_includes_efficiency() {
        let mut c = BatteryCharger::new(ChargerParams::default());
        let o = c.step(&ChargerInput {
            battery_emf_v: 27.0,
            battery_r_ohm: 0.05,
            ..ChargerInput::default()
        });
        assert!(o.mode == ChargerMode::ConstantCurrent);
        assert!(o.input_power_w > o.output_v * 46.0);
    }
}

#[cfg(test)]
mod timeout_tests {
    use super::*;

    #[test]
    fn charge_not_finished_in_90_minutes_faults_the_charger() {
        let mut c = BatteryCharger::new(ChargerParams::default());
        let input = ChargerInput {
            dt_us: 1_000_000,
            battery_emf_v: 26.0,
            ..ChargerInput::default()
        };
        for _ in 0..(90 * 60) {
            assert!(!c.step(&input).failed);
        }
        let out = c.step(&input);
        assert!(out.failed);
        assert_eq!(out.mode, ChargerMode::Fault);
    }
}
