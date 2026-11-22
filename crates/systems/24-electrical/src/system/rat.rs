use crate::sources::{Gcu, GcuInput, GcuOutput, GcuParams, MachineParams, rat_breaker_close_permit};

pub const RAT_DEPLOY_DELAY_US: u64 = 150_000;
pub const RAT_GOVERNED_RPM: f64 = 8000.0;
pub const APPROX_RAT_SPINUP_TAU_S: f64 = 2.5;
pub const APPROX_RAT_SPINDOWN_TAU_S: f64 = 6.0;
pub const RAT_AT_SPEED_FRACTION: f64 = 0.95;
pub const RAT_STOWED_RPM: f64 = 50.0;
pub const RAT_PMG_MIN_V: f64 = 20.0;

#[derive(Clone, Copy, Debug, Default)]
pub struct RatInput {
    pub dt_us: u64,
    pub now_us: u64,
    pub request: bool,
    pub on_ground: bool,
    pub host_rpm: f64,
    pub terminal_v: f64,
    pub current_a: f64,
    pub rcb_closed: bool,
    pub gen_lost: bool,
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct RatOutput {
    pub rpm: f64,
    pub deployed: bool,
    pub gen_ok: bool,
    pub field_on: bool,
    pub exciter_a: f64,
    pub rcb_permit: bool,
    pub elec_freq_hz: f64,
}

#[derive(Clone, Debug)]
pub struct RatUnit {
    params: MachineParams,
    gcu: Gcu,
    gcu_out: GcuOutput,
    request_since_us: Option<u64>,
    deployed: bool,
    rpm: f64,
    out: RatOutput,
}

impl Default for RatUnit {
    fn default() -> Self {
        Self::new()
    }
}

impl RatUnit {
    pub fn new() -> Self {
        Self {
            params: MachineParams::rat(),
            gcu: Gcu::new(GcuParams::rat()),
            gcu_out: GcuOutput::default(),
            request_since_us: None,
            deployed: false,
            rpm: 0.0,
            out: RatOutput::default(),
        }
    }

    pub fn deployed(&self) -> bool {
        self.deployed
    }

    pub fn output(&self) -> RatOutput {
        self.out
    }

    pub fn command_delay_ms(&self) -> f64 {
        if self.request_since_us.is_some() {
            RAT_DEPLOY_DELAY_US as f64 / 1000.0
        } else {
            0.0
        }
    }

    pub fn transient(&self) -> bool {
        if self.request_since_us.is_some() && !self.deployed {
            return true;
        }
        if !self.deployed && self.rpm <= RAT_STOWED_RPM {
            return false;
        }
        let target = RAT_GOVERNED_RPM * RAT_AT_SPEED_FRACTION;
        self.rpm < target || !self.gcu_out.voltage_established
    }

    pub fn step(&mut self, i: &RatInput) -> RatOutput {
        let dt = i.dt_us as f64 * 1.0e-6;
        if i.request && !i.on_ground {
            let since = *self.request_since_us.get_or_insert(i.now_us);
            if i.now_us.saturating_sub(since) >= RAT_DEPLOY_DELAY_US {
                self.deployed = true;
            }
        } else if !i.request {
            self.request_since_us = None;
        }
        if i.on_ground && self.deployed && self.rpm <= RAT_STOWED_RPM {
            self.deployed = false;
            self.request_since_us = None;
        }
        let target = if !self.deployed || i.on_ground {
            0.0
        } else if i.host_rpm > 0.0 {
            i.host_rpm
        } else {
            RAT_GOVERNED_RPM
        };
        let tau = if target > self.rpm {
            APPROX_RAT_SPINUP_TAU_S
        } else {
            APPROX_RAT_SPINDOWN_TAU_S
        };
        self.rpm += (target - self.rpm) * (1.0 - (-dt / tau).exp());
        if self.rpm < 1.0 && target == 0.0 {
            self.rpm = 0.0;
        }
        let pmg_v = self.rpm * self.params.pmg_v_per_rpm;
        let at_speed = self.rpm >= RAT_GOVERNED_RPM * RAT_AT_SPEED_FRACTION;
        let gcu_in = GcuInput {
            dt_us: i.dt_us,
            pmg_available: pmg_v > RAT_PMG_MIN_V && !i.gen_lost,
            pmg_freq_hz: self.params.pmg_freq_hz(self.rpm),
            gcs_on: true,
            reset_cmd: false,
            phase_v: [i.terminal_v; 3],
            gen_i: [i.current_a; 3],
            feeder_i: [if i.rcb_closed { i.current_a } else { 0.0 }; 3],
            gcb_closed: i.rcb_closed,
            engine_start_mode: false,
            idle_valid: true,
            engine_at_idle: at_speed,
            phase_seq_abc: true,
            ..GcuInput::default()
        };
        self.gcu_out = self.gcu.step(&gcu_in);
        let freq = self.params.electrical_freq_hz(self.rpm);
        let field_on = !self.gcu_out.field_off && self.gcu_out.exciter_current_a > 0.0 && !i.gen_lost;
        let permit = rat_breaker_close_permit(freq, &self.gcu_out) && !self.gcu_out.trip_latched;
        self.out = RatOutput {
            rpm: self.rpm,
            deployed: self.deployed,
            gen_ok: permit,
            field_on,
            exciter_a: self.gcu_out.exciter_current_a,
            rcb_permit: permit,
            elec_freq_hz: freq,
        };
        self.out
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn run(unit: &mut RatUnit, secs: f64, request: bool) {
        let steps = (secs * 1000.0) as u64;
        let mut now = 0;
        for _ in 0..steps {
            now += 1000;
            unit.step(&RatInput {
                dt_us: 1000,
                now_us: now,
                request,
                terminal_v: 230.0,
                ..RatInput::default()
            });
        }
    }

    #[test]
    fn deploys_within_200_ms_of_request() {
        let mut u = RatUnit::new();
        run(&mut u, 0.19, true);
        assert!(u.deployed());
        let mut v = RatUnit::new();
        run(&mut v, 0.1, true);
        assert!(!v.deployed());
    }

    #[test]
    fn spins_to_governed_speed_in_about_ten_seconds() {
        let mut u = RatUnit::new();
        run(&mut u, 10.0, true);
        assert!(u.output().rpm > 0.97 * RAT_GOVERNED_RPM);
    }

    #[test]
    fn no_request_stays_stowed() {
        let mut u = RatUnit::new();
        run(&mut u, 5.0, false);
        assert!(!u.deployed());
        assert_eq!(u.output().rpm, 0.0);
        assert!(!u.output().rcb_permit);
    }
}
