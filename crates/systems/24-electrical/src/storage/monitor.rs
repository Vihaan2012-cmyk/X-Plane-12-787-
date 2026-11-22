pub const DISCH_HIGH_A: f64 = 30.0;
pub const DISCH_HIGH_US: u64 = 20_000_000;
pub const INHIBIT_LOW_A: f64 = 5.0;
pub const MSG_LOW_A: f64 = 3.0;
pub const DISCH_LOW_US: u64 = 600_000_000;
pub const INHIBIT_HBB_V: f64 = 24.0;
pub const INHIBIT_HBB_US: u64 = 60_000_000;
pub const CLEAR_A: f64 = 5.0;
pub const CLEAR_A_US: u64 = 1_000_000;
pub const CLEAR_HBB_V: f64 = 29.0;
pub const CLEAR_HBB_US: u64 = 1_500_000;
pub const FAULT_SENSE_US: u64 = 15_000_000;
pub const HBB_UV_V: f64 = 24.0;
pub const HBB_UV_US: u64 = 15_000_000;
pub const HBB_OV_V: f64 = 35.0;
pub const HBB_OV_US: u64 = 60_000_000;
pub const FULL_CHARGE_V: f64 = 31.0;
pub const FULL_CHARGE_A: f64 = 5.0;
pub const TOW_HIGH_V: f64 = 30.1;
pub const TOW_LOW_V: f64 = 29.7;
pub const TOW_HIGH_MIN: f64 = 60.0;
pub const TOW_MEDIUM_MIN: f64 = 30.0;
pub const TOW_LOW_MIN: f64 = 15.0;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum TowLight {
    #[default]
    Low,
    Medium,
    High,
}

impl TowLight {
    pub fn from_volts(v: f64) -> TowLight {
        if v > TOW_HIGH_V {
            TowLight::High
        } else if v >= TOW_LOW_V {
            TowLight::Medium
        } else {
            TowLight::Low
        }
    }

    pub fn minimum_minutes(self) -> f64 {
        match self {
            TowLight::High => TOW_HIGH_MIN,
            TowLight::Medium => TOW_MEDIUM_MIN,
            TowLight::Low => TOW_LOW_MIN,
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct MonitorInput {
    pub dt_us: u64,
    pub discharge_a: f64,
    pub battery_v: f64,
    pub hbb_v: f64,
    pub transfer_pending: bool,
    pub switch_cycled_off: bool,
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct MonitorOutput {
    pub discharge_inhibit: bool,
    pub disch_message: bool,
    pub fault_sensed: bool,
    pub hbb_uv_fault: bool,
    pub hbb_ov_fault: bool,
    pub fully_charged: bool,
    pub tow_light: TowLight,
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct BatteryMonitor {
    t_high: u64,
    t_inhibit_low: u64,
    t_msg_low: u64,
    t_hbb_low: u64,
    t_clear_a: u64,
    t_clear_v: u64,
    t_sense: u64,
    t_uv: u64,
    t_ov: u64,
    inhibit: bool,
    sensed: bool,
    uv: bool,
    ov: bool,
}

fn run(timer: &mut u64, cond: bool, dt: u64) -> u64 {
    *timer = if cond { timer.saturating_add(dt) } else { 0 };
    *timer
}

impl BatteryMonitor {
    pub fn step(&mut self, i: &MonitorInput) -> MonitorOutput {
        let dt = i.dt_us;
        let d = i.discharge_a;
        let high = run(&mut self.t_high, d >= DISCH_HIGH_A && !i.transfer_pending, dt) >= DISCH_HIGH_US;
        let inhibit_low =
            run(&mut self.t_inhibit_low, d > INHIBIT_LOW_A && d < DISCH_HIGH_A, dt) >= DISCH_LOW_US;
        let msg_low = run(&mut self.t_msg_low, d > MSG_LOW_A && d < DISCH_HIGH_A, dt) >= DISCH_LOW_US;
        let hbb_low = run(&mut self.t_hbb_low, i.hbb_v < INHIBIT_HBB_V, dt) >= INHIBIT_HBB_US;
        if high || inhibit_low || hbb_low {
            self.inhibit = true;
        }
        let clear_a = run(&mut self.t_clear_a, d < CLEAR_A, dt) >= CLEAR_A_US;
        let clear_v = run(&mut self.t_clear_v, i.hbb_v > CLEAR_HBB_V, dt) >= CLEAR_HBB_US;
        if self.inhibit && clear_a && clear_v && !high && !inhibit_low && !hbb_low {
            self.inhibit = false;
        }
        if run(&mut self.t_sense, self.inhibit, dt) >= FAULT_SENSE_US {
            self.sensed = true;
        }
        if run(&mut self.t_uv, i.hbb_v < HBB_UV_V, dt) >= HBB_UV_US {
            self.uv = true;
        }
        if run(&mut self.t_ov, i.hbb_v > HBB_OV_V, dt) >= HBB_OV_US {
            self.ov = true;
        }
        if i.switch_cycled_off {
            self.sensed = false;
            self.uv = false;
            self.ov = false;
        }
        MonitorOutput {
            discharge_inhibit: self.inhibit,
            disch_message: high || msg_low,
            fault_sensed: self.sensed,
            hbb_uv_fault: self.uv,
            hbb_ov_fault: self.ov,
            fully_charged: i.battery_v >= FULL_CHARGE_V && d < 0.0 && -d < FULL_CHARGE_A,
            tow_light: TowLight::from_volts(i.battery_v),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const DT: u64 = 100_000;

    fn input(d: f64, hbb: f64) -> MonitorInput {
        MonitorInput {
            dt_us: DT,
            discharge_a: d,
            battery_v: 30.0,
            hbb_v: hbb,
            ..MonitorInput::default()
        }
    }

    fn secs(m: &mut BatteryMonitor, i: &MonitorInput, s: f64) -> MonitorOutput {
        let mut o = MonitorOutput::default();
        for _ in 0..(s * 1.0e6 / DT as f64).round() as u64 {
            o = m.step(i);
        }
        o
    }

    #[test]
    fn heavy_discharge_inhibits_after_20s_unless_transfer_pending() {
        let mut m = BatteryMonitor::default();
        assert!(!secs(&mut m, &input(35.0, 27.0), 19.9).discharge_inhibit);
        assert!(secs(&mut m, &input(35.0, 27.0), 0.1).discharge_inhibit);
        let mut m = BatteryMonitor::default();
        let mut i = input(35.0, 27.0);
        i.transfer_pending = true;
        assert!(!secs(&mut m, &i, 60.0).discharge_inhibit);
    }

    #[test]
    fn moderate_discharge_message_at_3a_and_inhibit_at_5a_after_10_min() {
        let mut m = BatteryMonitor::default();
        let o = secs(&mut m, &input(4.0, 27.0), 600.0);
        assert!(o.disch_message && !o.discharge_inhibit);
        let mut m = BatteryMonitor::default();
        let o = secs(&mut m, &input(6.0, 27.0), 600.0);
        assert!(o.disch_message && o.discharge_inhibit);
    }

    #[test]
    fn inhibit_clears_only_when_current_and_hbb_both_recover() {
        let mut m = BatteryMonitor::default();
        secs(&mut m, &input(0.0, 23.0), 60.0);
        assert!(m.inhibit);
        assert!(secs(&mut m, &input(0.0, 28.0), 5.0).discharge_inhibit);
        assert!(secs(&mut m, &input(0.0, 29.5), 1.4).discharge_inhibit);
        assert!(!secs(&mut m, &input(0.0, 29.5), 0.2).discharge_inhibit);
    }

    #[test]
    fn inhibit_is_sensed_as_a_fault_after_15s_and_switch_cycle_resets() {
        let mut m = BatteryMonitor::default();
        secs(&mut m, &input(0.0, 23.0), 60.0);
        assert!(!secs(&mut m, &input(0.0, 23.0), 14.8).fault_sensed);
        assert!(secs(&mut m, &input(0.0, 23.0), 0.2).fault_sensed);
        let mut i = input(0.0, 30.0);
        i.switch_cycled_off = true;
        assert!(!m.step(&i).fault_sensed);
    }

    #[test]
    fn hbb_under_and_over_voltage_faults_use_their_own_timers() {
        let mut m = BatteryMonitor::default();
        assert!(!secs(&mut m, &input(0.0, 23.5), 14.9).hbb_uv_fault);
        assert!(secs(&mut m, &input(0.0, 23.5), 0.1).hbb_uv_fault);
        let mut m = BatteryMonitor::default();
        assert!(!secs(&mut m, &input(0.0, 36.0), 59.9).hbb_ov_fault);
        assert!(secs(&mut m, &input(0.0, 36.0), 0.1).hbb_ov_fault);
    }

    #[test]
    fn full_charge_and_tow_light_follow_terminal_voltage() {
        let mut m = BatteryMonitor::default();
        let mut i = input(-3.0, 31.0);
        i.battery_v = 31.2;
        assert!(m.step(&i).fully_charged);
        i.discharge_a = -10.0;
        assert!(!m.step(&i).fully_charged);
        assert_eq!(TowLight::from_volts(30.2), TowLight::High);
        assert_eq!(TowLight::from_volts(29.9), TowLight::Medium);
        assert_eq!(TowLight::from_volts(29.5), TowLight::Low);
        assert_eq!(TowLight::High.minimum_minutes(), 60.0);
    }
}
