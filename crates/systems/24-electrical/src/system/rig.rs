use super::{BUS_LIVE_V, Electrical};
use crate::sources::APPROX_MOTORING_EFFICIENCY;
use core::f64::consts::TAU;

const FT_LB_NM: f64 = 1.355_818;
const RPM_TO_RAD_S: f64 = TAU / 60.0;
pub(crate) const VFSG_START_TORQUE_FTLB: f64 = 300.0;
pub(crate) const CMSC_OVERLOAD_W: f64 = 150_000.0;
pub(crate) const CMSC_RATING_W: f64 = 105_000.0;
pub(crate) const CMSC_MES_LIMIT_S: f64 = 180.0;
const VFSG_SCO_RPM: f64 = 6780.0;
pub(crate) const VFSG_MOTOR_TORQUE_FTLB: f64 = 150.0;
pub(crate) const VFSG_DUTY_COOLDOWN_S: f64 = 30.0 * 60.0;
pub(crate) const VFSG_COOLDOWN_TORQUE_FRACTION: f64 = 0.5;
pub(crate) const START_EXT_MIN_FWD_SOURCES: usize = 2;
pub(crate) const GEC1_CLOSE_HZ: f64 = 585.0;
pub(crate) const GEC1_OPEN_HZ: f64 = 600.0;
pub(crate) const GEC2_CLOSE_HZ: f64 = 825.0;
pub(crate) const GEC2_OPEN_HZ: f64 = 1050.0;
const GNR_SC_CLOSE_HZ: f64 = 825.0;
const GNR_SC_OPEN_HZ: f64 = 1050.0;
const GEC3_CLOSE_HZ: f64 = 1055.0;
const GEC3_OPEN_HZ: f64 = 1020.0;
pub(crate) const ENGINE_IDLE_RPM: f64 = 7400.0;
const APPROX_ENGINE_AT_IDLE_RPM: f64 = 7300.0;
const APPROX_ENGINE_GROUND_IDLE_RPM: f64 = 7550.0;
const APU_OPERATIONAL_RPM: f64 = 12_000.0;
const APU_ASG_CUTOFF_RPM: f64 = 6122.0;
pub(crate) const APU_READY_FRACTION: f64 = 0.95;
pub(crate) const APU_READY_HOLD_S: f64 = 2.0;
const APPROX_SPU_PBIT_S: f64 = 0.04;
const SPU_OUTPUT_W: f64 = 13_500.0;
pub(crate) const APU_BATT_ATTEMPT_S: f64 = 45.0;
const APU_AC_ATTEMPT_S: f64 = 52.0;
pub(crate) const APU_ATTEMPTS: u8 = 3;
const APPROX_ASG_AC_START_W: f64 = 60_000.0;
const APPROX_ASG_START_TORQUE_NM: f64 = 100.0;
const MIN_OMEGA_RAD_S: f64 = 1.0;
const VFSG_CMSC: [&str; 4] = ["CMSC_CAC_L1", "CMSC_CAC_R1", "CMSC_CAC_L2", "CMSC_CAC_R2"];
const APU_CMSC: &str = "CMSC_CAC_R2";

const APPROX_ENGINE_INERTIA_KGM2: f64 = 68.0;
const APPROX_ENGINE_DRAG0_NM: f64 = 25.0;
const APPROX_ENGINE_DRAG2_NMS2: f64 = 0.000_6;
const APPROX_ENGINE_LIGHTOFF_RPM: f64 = 3000.0;
const APPROX_ENGINE_ASSIST_NM: f64 = 20000.0;
const APPROX_ENGINE_TURBINE_MAX_NM: f64 = 700.0;
const APPROX_ENGINE_ASSIST_RAMP_S: f64 = 10.0;
const APPROX_APU_INERTIA_KGM2: f64 = 5.0;
const APPROX_APU_DRAG0_NM: f64 = 4.0;
const APPROX_APU_DRAG2_NMS2: f64 = 0.000_035_6;
const APPROX_APU_LIGHTOFF_RPM: f64 = 2500.0;
const APPROX_APU_ASSIST_NM: f64 = 1500.0;
const APPROX_APU_TURBINE_MAX_NM: f64 = 140.0;
const APPROX_APU_ASSIST_RAMP_S: f64 = 6.0;

#[derive(Clone, Copy, Debug)]
struct SpoolSpec {
    inertia: f64,
    drag0_nm: f64,
    drag2: f64,
    lightoff_rpm: f64,
    idle_rpm: f64,
    assist_nm: f64,
    max_nm: f64,
    assist_ramp_s: f64,
}

const ENGINE: SpoolSpec = SpoolSpec {
    inertia: APPROX_ENGINE_INERTIA_KGM2,
    drag0_nm: APPROX_ENGINE_DRAG0_NM,
    drag2: APPROX_ENGINE_DRAG2_NMS2,
    lightoff_rpm: APPROX_ENGINE_LIGHTOFF_RPM,
    idle_rpm: APPROX_ENGINE_GROUND_IDLE_RPM,
    assist_nm: APPROX_ENGINE_ASSIST_NM,
    max_nm: APPROX_ENGINE_TURBINE_MAX_NM,
    assist_ramp_s: APPROX_ENGINE_ASSIST_RAMP_S,
};

const APU: SpoolSpec = SpoolSpec {
    inertia: APPROX_APU_INERTIA_KGM2,
    drag0_nm: APPROX_APU_DRAG0_NM,
    drag2: APPROX_APU_DRAG2_NMS2,
    lightoff_rpm: APPROX_APU_LIGHTOFF_RPM,
    idle_rpm: APU_OPERATIONAL_RPM,
    assist_nm: APPROX_APU_ASSIST_NM,
    max_nm: APPROX_APU_TURBINE_MAX_NM,
    assist_ramp_s: APPROX_APU_ASSIST_RAMP_S,
};

#[derive(Clone, Copy, Debug, Default)]
pub(crate) struct Spool {
    pub rpm: f64,
    pub fuel: bool,
    pub lit: bool,
    pub lit_s: f64,
    pub owned: bool,
    pub cranking: bool,
    pub crank_s: f64,
    pub last_crank_s: f64,
}

impl Spool {
    fn drag_nm(&self, spec: &SpoolSpec) -> f64 {
        let w = self.rpm * RPM_TO_RAD_S;
        if self.rpm <= 0.0 {
            0.0
        } else {
            spec.drag0_nm + spec.drag2 * w * w
        }
    }

    fn turbine_nm(&self, spec: &SpoolSpec) -> f64 {
        if !self.lit {
            return 0.0;
        }
        let ramp = (self.lit_s / spec.assist_ramp_s).min(1.0);
        let margin = ((spec.idle_rpm - self.rpm) / spec.idle_rpm).max(-0.05);
        let drag = spec.drag0_nm + spec.drag2 * (self.rpm * RPM_TO_RAD_S).powi(2);
        ramp * (drag + spec.assist_nm * margin).min(spec.max_nm)
    }

    fn step(&mut self, dt: f64, spec: &SpoolSpec, start_nm: f64) {
        if self.lit && !self.fuel {
            self.lit = false;
            self.lit_s = 0.0;
        }
        if !self.lit && self.fuel && self.rpm >= spec.lightoff_rpm {
            self.lit = true;
        }
        if self.lit {
            self.lit_s += dt;
        }
        let net = start_nm + self.turbine_nm(spec) - self.drag_nm(spec);
        let w = self.rpm * RPM_TO_RAD_S + net / spec.inertia * dt;
        self.rpm = (w / RPM_TO_RAD_S).max(0.0);
        if self.cranking {
            self.crank_s += dt;
        }
    }

    pub(crate) fn run_at_idle(&mut self, idle_rpm: f64) {
        self.rpm = idle_rpm;
        self.fuel = true;
        self.lit = true;
        self.lit_s = f64::MAX / 4.0;
        self.owned = true;
        self.cranking = false;
    }

    pub(crate) fn stop(&mut self) {
        self.rpm = 0.0;
        self.fuel = false;
        self.lit = false;
        self.lit_s = 0.0;
        self.owned = false;
        self.cranking = false;
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) enum VfsgMode {
    #[default]
    Stop,
    Start,
    Motor,
    Generator,
}

pub(crate) const VFSG_MODES: [VfsgMode; 4] = [
    VfsgMode::Stop,
    VfsgMode::Start,
    VfsgMode::Motor,
    VfsgMode::Generator,
];

#[derive(Clone, Copy, Debug, Default)]
pub(crate) struct VfsgStart {
    pub gec1: bool,
    pub gec2: bool,
    pub gnr_open: bool,
    pub start_contactor: bool,
    pub gec3: bool,
    pub torque_nm: f64,
    pub mode: VfsgMode,
    pub duty_s: f64,
    pub cooldown_s: f64,
    idle_s: f64,
    demand_w: f64,
}

#[derive(Clone, Copy, Debug, Default)]
pub(crate) struct Rig {
    pub engine: [Spool; 2],
    pub apu: Spool,
    pub vfsg: [VfsgStart; 4],
    pub spu_pbit_s: f64,
    pub spu_path: bool,
    pub apu_attempts_used: u8,
    pub motor_request: [bool; 2],
    pub start_inhibited: [bool; 2],
    apu_demand_w: f64,
    apu_torque_nm: f64,
    apu_ready_s: f64,
}

impl Rig {
    pub(crate) fn new() -> Self {
        Self::default()
    }

    pub(crate) fn spu_closed_cmd(&self) -> bool {
        self.apu.cranking && self.spu_path
    }

    pub(crate) fn spub_closed_cmd(&self) -> bool {
        self.apu.cranking && self.spu_path && self.spu_pbit_s >= APPROX_SPU_PBIT_S
    }

    pub(crate) fn attempt_limit_s(&self) -> f64 {
        if self.spu_path {
            APU_BATT_ATTEMPT_S
        } else {
            APU_AC_ATTEMPT_S
        }
    }

    pub(crate) fn attempts_available(&self) -> f64 {
        f64::from(APU_ATTEMPTS.saturating_sub(self.apu_attempts_used))
    }

    pub(crate) fn start_torque_ftlb(&self) -> f64 {
        self.vfsg
            .iter()
            .map(|v| v.torque_nm / FT_LB_NM)
            .fold(0.0_f64, f64::max)
    }

    pub(crate) fn start_time_s(&self, engine: usize) -> f64 {
        self.engine[engine.min(1)].last_crank_s
    }
}

impl Electrical {
    fn vfsg_pmg_hz(&self, m: usize) -> f64 {
        self.machine_params[m].pmg_freq_hz(self.rig.engine[m / 2].rpm)
    }

    fn update_vfsg_latches(&mut self, m: usize) {
        let k = m / 2;
        let hz = self.vfsg_pmg_hz(m);
        let start = self.crew_in.engine_start_in_progress[k] || self.motoring(k);
        let trip = self.gcu_out[m].trip_latched;
        let gcs = self.panel_out.gen_ctrl_on[m];
        let v = &mut self.rig.vfsg[m];
        v.gec1 = start
            && if v.gec1 {
                hz <= GEC1_OPEN_HZ
            } else {
                hz <= GEC1_CLOSE_HZ
            };
        v.gec2 = start
            && if v.gec2 {
                hz < GEC2_OPEN_HZ
            } else {
                hz <= GEC2_CLOSE_HZ
            };
        v.gnr_open = start
            && if v.gnr_open {
                hz < GNR_SC_OPEN_HZ
            } else {
                hz <= GNR_SC_CLOSE_HZ
            };
        v.start_contactor = start
            && if v.start_contactor {
                hz < GNR_SC_OPEN_HZ
            } else {
                hz <= GNR_SC_CLOSE_HZ
            };
        v.gec3 = !start
            && !trip
            && gcs
            && if v.gec3 {
                hz > GEC3_OPEN_HZ
            } else {
                hz >= GEC3_CLOSE_HZ
            };
    }

    fn vfsg_command(&mut self, m: usize, dt: f64) -> f64 {
        let k = m / 2;
        let omega = (self.rig.engine[k].rpm * RPM_TO_RAD_S).max(MIN_OMEGA_RAD_S);
        let eta = APPROX_MOTORING_EFFICIENCY;
        let active = self.rig.vfsg[m].start_contactor;
        let prev_demand = self.rig.vfsg[m].demand_w;
        let delivered = if prev_demand > 100.0 {
            (self.plant.load_power(VFSG_CMSC[m]) / prev_demand).clamp(0.0, 1.0)
        } else {
            1.0
        };
        let motor = self.motoring(k);
        let cooling = self.rig.vfsg[m].cooldown_s > 0.0;
        let limit_ftlb = if motor {
            VFSG_MOTOR_TORQUE_FTLB
        } else if cooling {
            VFSG_START_TORQUE_FTLB * VFSG_COOLDOWN_TORQUE_FRACTION
        } else {
            VFSG_START_TORQUE_FTLB
        };
        let t_cmd = if active {
            (limit_ftlb * FT_LB_NM).min(CMSC_OVERLOAD_W * eta / omega)
        } else {
            0.0
        };
        self.step_vfsg_duty(m, active && limit_ftlb >= VFSG_START_TORQUE_FTLB, dt);
        let want_w = (t_cmd * omega / eta).min(CMSC_OVERLOAD_W);
        let slew = CMSC_OVERLOAD_W * dt / 0.05;
        let demand = if want_w > prev_demand {
            (prev_demand + slew).min(want_w)
        } else {
            want_w
        };
        if demand != prev_demand {
            self.plant.set_load(VFSG_CMSC[m], demand);
        }
        self.rig.vfsg[m].demand_w = demand;
        let torque = if active { t_cmd * delivered } else { 0.0 };
        self.rig.vfsg[m].torque_nm = t_cmd;
        torque
    }

    fn motoring(&self, k: usize) -> bool {
        self.rig.motor_request[k] && !self.crew_in.engine_start_in_progress[k] && !self.rig.engine[k].lit
    }

    #[cfg(test)]
    pub(crate) fn step_vfsg_duty_for_test(&mut self, m: usize, full_torque: bool, dt: f64) {
        self.step_vfsg_duty(m, full_torque, dt);
    }

    fn step_vfsg_duty(&mut self, m: usize, full_torque: bool, dt: f64) {
        let v = &mut self.rig.vfsg[m];
        v.cooldown_s = (v.cooldown_s - dt).max(0.0);
        if full_torque {
            v.idle_s = 0.0;
            v.duty_s += dt;
            if v.duty_s >= CMSC_MES_LIMIT_S {
                v.duty_s = 0.0;
                v.cooldown_s = VFSG_DUTY_COOLDOWN_S;
            }
        } else {
            v.idle_s += dt;
            if v.idle_s >= VFSG_DUTY_COOLDOWN_S {
                v.duty_s = 0.0;
            }
        }
    }

    fn start_power_ok(&self) -> bool {
        let gen_source = (0..4).any(|m| self.gear_out(&super::gcb_name(m)).contacts_closed)
            || self.bpcu_out.asg_cb_close.iter().any(|b| *b);
        let fwd_ext = ["L EPC", "R EPC"]
            .iter()
            .filter(|n| self.gear_out(n).contacts_closed)
            .count();
        gen_source || fwd_ext == 0 || fwd_ext >= START_EXT_MIN_FWD_SOURCES
    }

    fn update_vfsg_mode(&mut self, m: usize) {
        let k = m / 2;
        let v = self.rig.vfsg[m];
        self.rig.vfsg[m].mode = if self.drive_disconnected[m] {
            VfsgMode::Stop
        } else if v.start_contactor && self.crew_in.engine_start_in_progress[k] {
            VfsgMode::Start
        } else if v.start_contactor && self.motoring(k) {
            VfsgMode::Motor
        } else if v.gec3 || self.gear_out(&super::gcb_name(m)).contacts_closed {
            VfsgMode::Generator
        } else {
            VfsgMode::Stop
        };
    }

    fn step_engine_rig(&mut self, k: usize, dt: f64) {
        if self.crew_in.engine_start_in_progress[k] && !self.rig.engine[k].cranking && !self.start_power_ok()
        {
            self.crew_in.engine_start_in_progress[k] = false;
            self.rig.start_inhibited[k] = true;
        } else if self.crew_in.engine_start_in_progress[k] {
            self.rig.start_inhibited[k] = false;
        }
        for m in [2 * k, 2 * k + 1] {
            self.update_vfsg_latches(m);
            self.update_vfsg_mode(m);
        }
        let start = self.crew_in.engine_start_in_progress[k] || self.motoring(k);
        if !start && !self.rig.engine[k].owned {
            for m in [2 * k, 2 * k + 1] {
                self.rig.vfsg[m].torque_nm = 0.0;
                self.rig.vfsg[m].demand_w = 0.0;
            }
            return;
        }
        let was_start = start;
        let torque: f64 = [2 * k, 2 * k + 1].iter().map(|m| self.vfsg_command(*m, dt)).sum();
        let motor = self.motoring(k);
        let eng = &mut self.rig.engine[k];
        if motor {
            eng.owned = true;
        }
        eng.cranking = start;
        eng.step(dt, &ENGINE, torque);
        if was_start && !motor && (eng.rpm >= VFSG_SCO_RPM || eng.crank_s >= CMSC_MES_LIMIT_S) {
            eng.last_crank_s = eng.crank_s;
            eng.cranking = false;
            eng.crank_s = 0.0;
            self.crew_in.engine_start_in_progress[k] = false;
        }
        let run = eng.lit && eng.rpm >= APPROX_ENGINE_AT_IDLE_RPM;
        self.inputs.engine_rpm[k] = eng.rpm;
        self.inputs.engine_running[k] = run;
        if !eng.cranking && !eng.lit && eng.rpm <= 0.0 {
            eng.owned = false;
        }
    }

    fn step_apu_rig(&mut self, dt: f64) {
        let start_req = self.apu_start_latch_us.is_some() && !self.inputs.apu_running;
        if start_req && !self.rig.apu.cranking {
            self.rig.apu.cranking = true;
            self.rig.apu.crank_s = 0.0;
            self.rig.apu.fuel = true;
            self.rig.spu_path = self.plant.bus_v("R2") <= BUS_LIVE_V;
            self.rig.spu_pbit_s = 0.0;
        }
        if self.rig.apu.cranking {
            self.rig.spu_pbit_s += dt;
        } else {
            self.rig.spu_pbit_s = 0.0;
        }
        let eta = APPROX_MOTORING_EFFICIENCY;
        let omega = (self.rig.apu.rpm * RPM_TO_RAD_S).max(MIN_OMEGA_RAD_S);
        let cap_w = if self.rig.spu_path {
            SPU_OUTPUT_W
        } else {
            APPROX_ASG_AC_START_W
        };
        let prev = self.rig.apu_demand_w;
        let delivered = if prev > 100.0 {
            (self.plant.load_power(APU_CMSC) / prev).clamp(0.0, 1.0)
        } else {
            1.0
        };
        let live = !self.rig.spu_path || self.rig.spub_closed_cmd();
        let t_cmd = if self.rig.apu.cranking && live {
            APPROX_ASG_START_TORQUE_NM.min(cap_w * eta / omega)
        } else {
            0.0
        };
        let want_w = (t_cmd * omega / eta).min(cap_w);
        let slew = cap_w * dt / 0.05;
        let demand = if want_w > prev {
            (prev + slew).min(want_w)
        } else {
            want_w
        };
        if demand != prev {
            self.plant.set_load(APU_CMSC, demand);
        }
        self.rig.apu_demand_w = demand;
        let torque = t_cmd * delivered;
        self.rig.apu_torque_nm = torque;
        if !self.rig.apu.cranking && !self.rig.apu.owned {
            return;
        }
        self.rig.apu.step(dt, &APU, torque);
        let limit = self.rig.attempt_limit_s();
        if self.rig.apu.cranking && self.rig.apu.rpm >= APU_ASG_CUTOFF_RPM && self.rig.apu.lit {
            self.rig.apu.last_crank_s = self.rig.apu.crank_s;
            self.rig.apu.cranking = false;
            self.rig.apu.owned = true;
        } else if self.rig.apu.cranking && self.rig.apu.crank_s >= limit {
            self.rig.apu.cranking = false;
            self.rig.apu.fuel = false;
            self.rig.apu.lit = false;
            self.rig.apu_attempts_used = self.rig.apu_attempts_used.saturating_add(1);
            self.apu_start_latch_us = None;
        }
        if self.rig.apu.owned {
            if self.rig.apu.rpm >= APU_READY_FRACTION * APU_OPERATIONAL_RPM {
                self.rig.apu_ready_s += dt;
            } else {
                self.rig.apu_ready_s = 0.0;
            }
            if self.rig.apu_ready_s >= APU_READY_HOLD_S && !self.inputs.apu_running {
                self.inputs.apu_running = true;
                self.rig.apu_attempts_used = 0;
                self.apu_start_latch_us = None;
            }
        }
        self.inputs.apu_rpm = self.rig.apu.rpm;
        if self.rig.apu.owned && !self.rig.apu.lit {
            self.inputs.apu_running = false;
        }
    }

    pub(crate) fn stage_start(&mut self, dt_us: u64) {
        let dt = dt_us as f64 * 1.0e-6;
        for k in 0..2 {
            self.step_engine_rig(k, dt);
        }
        self.step_apu_rig(dt);
    }

    pub(crate) fn rig_set_engine_running(&mut self, k: usize, run: bool) {
        if run {
            self.rig.engine[k].run_at_idle(APPROX_ENGINE_GROUND_IDLE_RPM);
            self.inputs.engine_rpm[k] = APPROX_ENGINE_GROUND_IDLE_RPM;
        } else {
            self.rig.engine[k].stop();
            self.inputs.engine_rpm[k] = 0.0;
        }
        self.inputs.engine_running[k] = run;
    }

    pub(crate) fn rig_set_apu_running(&mut self, run: bool) {
        if run {
            self.rig.apu.run_at_idle(APU_OPERATIONAL_RPM);
            self.inputs.apu_rpm = APU_OPERATIONAL_RPM;
        } else {
            self.rig.apu.stop();
            self.inputs.apu_rpm = 0.0;
        }
        self.inputs.apu_running = run;
    }
}
