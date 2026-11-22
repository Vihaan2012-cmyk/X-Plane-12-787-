#![allow(clippy::needless_range_loop)]

mod actions;
mod bind_loads;
mod bind_misc;
mod bind_params;
mod breaker_link;
mod build;
mod causal;
mod drive_disc;
mod fault_link;
mod inputs;
mod interlock;
mod pdu_link;
mod plant;
mod protection_view;
mod rat;
mod regulation;
mod rig;
mod signals;
mod stepping;
#[cfg(test)]
mod tests;
#[cfg(test)]
mod thermal_fault_tests;
mod thermal_link;
mod view;

pub use inputs::{CrewCommand, ElectricalInputs, ExtSupply, Faults};
pub use pdu_link::{ChannelRow, LinkChannel, MapSummary, PduLink, RatingSource, RpduRow};
pub use signals::SignalValue;

use crate::control::bpcu::Src;
use crate::control::{
    BpcuInput, BpcuOutput, BpcuPair, Cbic, CbicCluster, CbicInput, Feed, GroundConfig, LmInput, LmOutput,
    LoadManager,
};
use crate::crew::{
    ApuSelector, CrewInputs, EicasElec, FlightPhase, OverheadPanel, PanelLamps, PanelOutputs, SynopticPage,
    build_synoptic, classify_phase,
};
use crate::protection::contactor::{Contactor, ContactorInput, ContactorOutput};
use crate::sources::{
    ExtPowerInput, ExtPowerMonitor, ExtPowerOutput, Gcu, GcuInput, GcuOutput, MachineParams, Receptacle,
};
use crate::storage::{
    BatteryCharger, BatteryDiodeModule, BatteryMonitoringUnit, BmuInput, BmuOutput, ChargerInput,
    ChargerOutput, MAX_CELLS,
};
use plant::Plant;

const OPEN_DIODE_DC_FACTOR: f64 = 4.05;
const CONTROL_POWER_MIN_V: f64 = 18.0;
const APPROX_VFSG_EFFICIENCY: f64 = 0.92;
const APPROX_HPX_IDLE_KW: f64 = 220.0;
const APPROX_HPX_MAX_KW: f64 = 650.0;
const APPROX_HPX_MAX_RPM: f64 = 15_900.0;
const APPROX_HPX_IDLE_RPM: f64 = 7_550.0;

pub(crate) fn hpx_limit_kw(rpm: f64, running: bool) -> f64 {
    if !running {
        return 0.0;
    }
    let f = ((rpm - APPROX_HPX_IDLE_RPM) / (APPROX_HPX_MAX_RPM - APPROX_HPX_IDLE_RPM)).clamp(0.0, 1.0);
    APPROX_HPX_IDLE_KW + f * (APPROX_HPX_MAX_KW - APPROX_HPX_IDLE_KW)
}
const ATRU_EFFICIENCY_DERIVED: f64 = 0.97;
const PECS_SUPPLY_C: f64 = (85.0 - 32.0) / 1.8;
const ATRU_MIN_COOLANT_KG_PER_S: f64 = 6.8 / 60.0;
const COOLANT_CP_J_PER_KG_K: f64 = 3_600.0;
const APPROX_PECS_STILL_W_PER_K: f64 = 2.0;
const APPROX_PECS_HEAT_CAP_J_PER_K: f64 = 60_000.0;
use rat::{RatInput, RatUnit};

pub const SUBSTEP_US: u64 = 1_000;
pub const COIL_NOMINAL_V: f64 = 28.0;
pub const BUS_LIVE_V: f64 = 100.0;
pub const RAT_SPINUP_RPM: f64 = 3000.0;
pub const MACHINES: [&str; 6] = ["VFSG_L1", "VFSG_L2", "VFSG_R1", "VFSG_R2", "ASG_L", "ASG_R"];

pub(crate) struct Switchgear {
    pub name: String,
    pub upstream: String,
    pub unit: Contactor,
    pub cmd: bool,
    pub out: ContactorOutput,
    pub forced_open: bool,
}

pub struct Electrical {
    pub(crate) plant: Plant,
    pub(crate) parallel_blocks: Vec<String>,
    pub(crate) tru_oc: [crate::protection::tru_oc::TruOc; 4],
    pub(crate) exciter_feedback_a: [f64; 6],
    pub(crate) atru_prot: [crate::protection::atru_prot::AtruProt; 4],
    pub(crate) epc_sup: [crate::protection::epc_supervisor::EpcSupervisor; 3],
    pub(crate) pecs_c: [f64; 2],
    pub(crate) managed_w: [[f64; 4]; 14],
    pub(crate) circuit_prot: Vec<breaker_link::CircuitProt>,
    pub(crate) batt_mon: crate::storage::monitor::BatteryMonitor,
    pub(crate) batt_mon_out: crate::storage::monitor::MonitorOutput,
    pub(crate) batt_switch_prev: bool,
    pub(crate) tie_diff: [crate::protection::zone_diff::ZoneDiff; 2],
    pub(crate) atu_diff: [crate::protection::zone_diff::ZoneDiff; 2],
    pub(crate) now_us: u64,
    pub(crate) panel: OverheadPanel,
    pub(crate) lamps: PanelLamps,
    pub(crate) panel_out: PanelOutputs,
    pub(crate) eicas: EicasElec,
    pub(crate) crew_in: CrewInputs,
    pub(crate) synoptic: SynopticPage,
    pub(crate) bpcu: BpcuPair,
    pub(crate) bpcu_out: BpcuOutput,
    pub(crate) rat: RatUnit,
    pub(crate) gcus: Vec<Gcu>,
    pub(crate) gcu_out: Vec<GcuOutput>,
    pub(crate) ext_mon: Vec<ExtPowerMonitor>,
    pub(crate) ext_out: Vec<ExtPowerOutput>,
    pub(crate) ext_switch_on: [bool; 3],
    pub(crate) gear: Vec<Switchgear>,
    pub(crate) bmu: [BatteryMonitoringUnit; 2],
    pub(crate) bmu_out: [BmuOutput; 2],
    pub(crate) chargers: [BatteryCharger; 2],
    pub(crate) charger_out: [ChargerOutput; 2],
    pub(crate) diode: BatteryDiodeModule,
    pub(crate) lm: LoadManager,
    pub(crate) lm_out: LmOutput,
    pub(crate) cbic: CbicCluster,
    pub(crate) inputs: ElectricalInputs,
    pub(crate) faults: Faults,
    pub(crate) apu_batt_switch_on: bool,
    pub(crate) ground_service_switch: bool,
    pub(crate) apu_start_latch_us: Option<u64>,
    pub(crate) drive_disconnected: [bool; 4],
    pub(crate) drive_sol: [drive_disc::DriveSolenoid; 4],
    pub(crate) machine_params: Vec<MachineParams>,
    pub(crate) last_dt_us: u64,
    pub(crate) pace: stepping::Pace,
    pub(crate) rig: rig::Rig,
    pub(crate) fault_link: fault_link::FaultLink,
    pub(crate) thermal: Option<thermal_link::ThermalLink>,
    pub(crate) pdu: pdu_link::PduLink,
}

impl Electrical {
    pub fn new(data_dir: &std::path::Path) -> Result<Electrical, String> {
        build::build(data_dir)
    }

    pub fn now_us(&self) -> u64 {
        self.now_us
    }

    pub fn inputs(&self) -> &ElectricalInputs {
        &self.inputs
    }

    pub fn faults_mut(&mut self) -> &mut Faults {
        &mut self.faults
    }

    pub fn panel(&self) -> &OverheadPanel {
        &self.panel
    }

    pub fn panel_mut(&mut self) -> &mut OverheadPanel {
        &mut self.panel
    }

    pub fn lamps(&self) -> &PanelLamps {
        &self.lamps
    }

    pub fn synoptic(&self) -> &SynopticPage {
        &self.synoptic
    }

    pub fn eicas(&self) -> &EicasElec {
        &self.eicas
    }

    pub fn circuit_names(&self) -> Vec<String> {
        self.plant
            .net()
            .circuits()
            .iter()
            .map(|(n, _)| n.clone())
            .collect()
    }

    pub fn parallel_blocks(&self) -> &[String] {
        &self.parallel_blocks
    }

    pub fn eicas_messages(&self) -> Vec<String> {
        self.eicas.names()
    }

    pub fn bpcu_output(&self) -> &BpcuOutput {
        &self.bpcu_out
    }

    pub fn load_output(&self) -> &LmOutput {
        &self.lm_out
    }

    pub fn cbic(&self) -> &CbicCluster {
        &self.cbic
    }

    pub fn cbic_mut(&mut self) -> &mut CbicCluster {
        &mut self.cbic
    }

    pub fn gcu_output(&self, machine: usize) -> Option<&GcuOutput> {
        self.gcu_out.get(machine)
    }

    pub fn ext_output(&self, receptacle: usize) -> Option<&ExtPowerOutput> {
        self.ext_out.get(receptacle)
    }

    pub fn contactor_closed(&self, name: &str) -> Option<bool> {
        self.gear
            .iter()
            .find(|g| g.name == name)
            .map(|g| g.out.contacts_closed)
    }

    pub fn contactor_names(&self) -> Vec<String> {
        self.gear.iter().map(|g| g.name.clone()).collect()
    }

    pub fn bus_volts(&self, bus: &str) -> f64 {
        self.plant.bus_v(bus)
    }

    pub fn set_external_load(&mut self, name: &str, watts: f64) -> bool {
        let base = self.fault_link.load_base.get(name).copied();
        let ok = match base {
            Some(_) => {
                self.fault_link.load_base.insert(name.to_string(), watts);
                true
            }
            None => self.plant.net_mut().set_load(name, watts),
        };
        self.pace.quiet = 0;
        ok
    }

    pub fn fuel_run(&self, engine: usize) -> bool {
        self.rig.engine.get(engine).is_some_and(|e| e.fuel)
    }

    fn apply_crew(&mut self, cmds: &[CrewCommand]) {
        for c in cmds {
            self.apply_command(*c);
        }
    }

    pub(crate) fn substep(&mut self, dt_us: u64) {
        self.now_us += dt_us;
        self.last_dt_us = dt_us;
        self.stage_fault_pre();
        self.release_due_drive_switches();
        self.stage_panel();
        self.step_drive_disconnects(dt_us);
        self.stage_sensing(dt_us);
        self.stage_contactors(dt_us);
        self.stage_start(dt_us);
        self.stage_machines(dt_us);
        self.stage_fault_post(dt_us);
        self.stage_solve(dt_us);
        self.stage_pdu(dt_us);
        self.stage_feedback(dt_us);
        self.stage_heat(dt_us);
    }

    fn engines_running(&self) -> u8 {
        self.inputs.engine_running.iter().filter(|r| **r).count() as u8
    }

    fn stage_panel(&mut self) {
        let (out, lamps) = self.panel.update(&self.crew_in);
        let (out, lamps) = self.causal_panel(out, lamps);
        for (i, pulse) in out.ext_fwd_pulse.iter().enumerate() {
            if *pulse {
                self.ext_switch_on[i] = !self.ext_switch_on[i];
            }
        }
        if out.ext_aft_pulse {
            self.ext_switch_on[2] = !self.ext_switch_on[2];
        }
        if out.apu_selector == ApuSelector::Start {
            self.apu_start_latch_us.get_or_insert(self.now_us);
            self.panel.release_apu_selector();
        }
        self.panel_out = out;
        self.lamps = lamps;
        self.ground_service_switch = out.ground_service_on;
        self.apu_batt_switch_on = out.battery_on;
    }

    fn stage_sensing(&mut self, dt_us: u64) {
        let on_ground = self.inputs.on_ground;
        for k in 0..3 {
            let supply = self.inputs.ext[k];
            let input = self.ext_input(k, dt_us);
            self.ext_out[k] = self.ext_mon[k].step(&input);
            if !supply.connected {
                self.ext_switch_on[k] = false;
            }
        }
        for m in 0..6 {
            self.step_gcu(m, dt_us);
        }
        let batt_v = self.plant.battery_v(0);
        let bpcu_in = self.bpcu_input(on_ground);
        self.bpcu.left_powered = batt_v > 18.0 && !self.faults.bpcu_fail[0];
        self.bpcu.right_powered = batt_v > 18.0 && !self.faults.bpcu_fail[1];
        self.bpcu_out = self.bpcu.step(&bpcu_in);
        self.step_load_manager(on_ground);
        self.step_cbic(on_ground);
    }

    fn ext_input(&self, k: usize, dt_us: u64) -> ExtPowerInput {
        let epc_names = ["L EPC", "R EPC", "LAEPC"];
        let supply = self.inputs.ext[k];
        let i_a = self.plant.contactor_a(epc_names[k]);
        ExtPowerInput {
            dt_us,
            connected: supply.connected,
            pin_ef_ok: self.causal_ext_pin_ef(k, supply.pin_ef_ok),
            phase_v: self.causal_ext_phase_v(k, supply.v_ll),
            phase_i: [i_a; 3],
            freq_hz: supply.hz,
            phase_seq_abc: supply.phase_seq_abc,
            engine_start_mode_active: self.crew_in.engine_start_in_progress.iter().any(|b| *b),
            atru_saturation: false,
        }
    }

    fn step_gcu(&mut self, m: usize, dt_us: u64) {
        let input = self.gcu_input(m, dt_us);
        self.gcu_out[m] = self.gcus[m].step(&input);
    }

    fn gcu_input(&self, m: usize, dt_us: u64) -> GcuInput {
        let rpm = self.shaft_rpm(m);
        let params = &self.machine_params[m];
        let pmg_hz = params.pmg_freq_hz(rpm);
        let pmg_v = rpm * params.pmg_v_per_rpm;
        let (terminal, _) = self.plant.machine_sense(m);
        let (phase_v, gen_i, _) = self.plant.machine_phases(m);
        let v = self.por_voltage(m);
        let breaker = if m < 4 { gcb_name(m) } else { apb_name(m - 4) };
        let gcb_closed = self.gear_out(&breaker).contacts_closed;
        let ctrl_on = if m < 4 {
            self.panel_out.gen_ctrl_on[m]
        } else {
            self.panel_out.apu_gen_on[m - 4]
        };
        let lost = if m < 4 {
            self.faults.gen_loss[m] || self.drive_disconnected[m]
        } else {
            self.faults.asg_loss[m - 4]
        };
        let engine = m / 2;
        let rpm_run = if m < 4 {
            self.inputs.engine_running[engine]
        } else {
            self.inputs.apu_running
        };
        GcuInput {
            dt_us,
            pmg_available: pmg_v > 20.0 && !lost,
            pmg_freq_hz: pmg_hz,
            gcs_on: ctrl_on,
            reset_cmd: if m < 4 {
                self.panel_out.gen_ctrl_cycled[m]
            } else {
                false
            },
            phase_v: phase_v.map(|p| if terminal > 1.0 { v * p / terminal } else { v }),
            gen_i,
            feeder_i: if gcb_closed {
                let mut f = self.plant.contactor_phase_a(&breaker);
                f[0] += self.fault_link.ct_offset_a[m];
                f
            } else {
                [0.0; 3]
            },
            exciter_feedback_a: Some(self.exciter_feedback_a[m]),
            gcb_closed,
            residual_a: self.plant.feeder_zone_residual_a(m, &breaker),
            dc_content_a: self.dc_content_a(m),
            feeder_unbalance_a: self.feeder_unbalance_a(m, &breaker),
            engine_start_mode: if m < 4 {
                self.crew_in.engine_start_in_progress[m / 2]
            } else {
                self.rig.apu.cranking
            },
            idle_valid: true,
            engine_at_idle: rpm_run,
            phase_seq_abc: true,
        }
    }

    fn shaft_rpm(&self, m: usize) -> f64 {
        if m < 4 {
            self.inputs.engine_rpm[m / 2]
        } else {
            self.inputs.apu_rpm
        }
    }

    fn gear_out(&self, name: &str) -> ContactorOutput {
        self.gear
            .iter()
            .find(|g| g.name == name)
            .map(|g| g.out)
            .unwrap_or(ContactorOutput {
                contacts_closed: false,
                armature_closed: false,
                aux_no: false,
                aux_nc: true,
                resistance_ohm: 0.0,
                welded: false,
                bouncing: false,
            })
    }

    fn bpcu_input(&self, on_ground: bool) -> BpcuInput {
        let mut i = BpcuInput::cold_and_dark(self.now_us);
        i.on_ground = on_ground;
        for k in 0..4 {
            let g = &self.gcu_out[k];
            i.gen_ok[k] = self.gcus[k].generator_mode()
                && !self.crew_in.engine_start_in_progress[k / 2]
                && g.voltage_established
                && !g.trip_latched
                && !g.field_off;
            i.gen_ctrl_on[k] = self.panel_out.gen_ctrl_on[k];
            i.gcb_aux_closed[k] = self.gear_out(&gcb_name(k)).aux_no;
            i.bus_volts[k] = self.plant.bus_v(BUS_NAMES[k]);
        }
        for k in 0..2 {
            let g = &self.gcu_out[4 + k];
            i.asg_ok[k] = self.inputs.apu_running && g.voltage_established && !g.trip_latched && !g.field_off;
            i.asg_ctrl_on[k] = self.panel_out.apu_gen_on[k];
            i.asg_cb_aux_closed[k] = self.gear_out(&apb_name(k)).aux_no;
            i.atu_ok[k] = self.plant.bus_v(["ATU_L", "ATU_R"][k]) > 90.0;
            i.tie_fault[k] = self.tie_diff[k].tripped();
            i.bsb_aux_closed[k] = self.gear_out(["L BSB", "R BSB"][k]).aux_no;
            i.btb115_aux_closed[k] = self.gear_out(["L BTB 115", "R BTB 115"][k]).aux_no;
            i.dc_main_ok[k] = self.plant.bus_v(["DC28_L", "DC28_R"][k]) > 20.0;
            i.instr_bus_v[k] = self.plant.bus_v(["CIB", "FIB"][k]);
            i.instr_tru_a[k] = self.tru_output_a(2 + k);
        }
        let epc = ["L EPC", "R EPC", "LAEPC"];
        for k in 0..3 {
            i.ext_plugged[k] = self.inputs.ext[k].connected;
            i.ext_quality_ok[k] = self.ext_out[k].power_quality_good && !self.ext_out[k].tripped;
            i.alt_ft = self.inputs.alt_ft;
            i.ext_hz[k] = if self.inputs.ext[k].connected {
                self.inputs.ext[k].hz
            } else {
                0.0
            };
            i.ext_switch_on[k] = self.ext_switch_on[k];
            i.epc_aux_closed[k] = self.gear_out(epc[k]).aux_no;
        }
        for (k, name) in crate::control::bpcu::BTB_NAMES.iter().enumerate() {
            i.btb_aux_closed[k] = self.gear_out(name).aux_no;
        }
        i.batt_switch_on = self.panel_out.battery_on;
        i.apu_batt_switch_on = self.apu_batt_switch_on;
        i.apu_start_requested = self.apu_start_latch_us.is_some();
        i.rat_gen_ok = self.rat.output().gen_ok;
        i.backup_bus_ok = self.plant.bus_v("BB") > 150.0;
        i.ground_service_switch = self.ground_service_switch;
        i.engines_running = self.engines_running();
        i
    }

    fn lm_requests(&self, on_ground: bool) -> [bool; 14] {
        let ac_live = |b: &str| self.plant.bus_v(b) > BUS_LIVE_V;
        let dc_live = |b: &str| self.plant.bus_v(b) > 18.0;
        let ac235 = ["L1", "L2", "R1", "R2"].iter().any(|b| ac_live(b));
        let ac115 = ac_live("AC115_L") || ac_live("AC115_R");
        let dc28 = dc_live("DC28_L") || dc_live("DC28_R");
        let p = &self.panel_out;
        let powered_flight = !on_ground || self.engines_running() > 0;
        let packs = ac235 && powered_flight;
        [
            ac235 && p.cabin_util_on && self.inputs.galley_power,
            ac235 && p.cabin_util_on && self.inputs.galley_power,
            ac115 && p.ife_on,
            ac115 && p.cabin_util_on,
            ac235 && self.inputs.cargo_heat,
            packs,
            packs && on_ground,
            ac235 || ac115,
            packs,
            ac235 && self.engines_running() > 0,
            ac235 && powered_flight,
            ac115 && powered_flight,
            dc28,
            dc28,
        ]
    }

    fn ext_load_kva(&self) -> f64 {
        let epc_names = ["L EPC", "R EPC", "LAEPC"];
        (0..3)
            .filter(|&k| self.bpcu_out.ext_on[k])
            .map(|k| {
                let host = self.inputs.ext_load_kva[k];
                if host > 0.0 {
                    host
                } else {
                    3.0 * self.plant.bus_v(["AC115_L", "AC115_R", "AC115_L"][k]).max(0.0)
                        * self.plant.contactor_a(epc_names[k])
                        / 1000.0
                }
            })
            .sum()
    }

    fn lm_input(&self, on_ground: bool) -> LmInput {
        let ext_in_use = self.bpcu_out.ext_on.iter().any(|b| *b);
        LmInput {
            now_us: self.now_us,
            on_ground,
            available_kw: self.bpcu_out.source_kva * 0.9,
            demand_on: self.lm_requests(on_ground),
            engine_start: self.crew_in.engine_start_in_progress.iter().any(|b| *b),
            source_change: self.bpcu_out.source_change,
            ext_in_use,
            ext_kva: self.ext_load_kva(),
            overload_hint_kw: 0.0,
            hpx_kw: [0, 1].map(|e| self.hpx_kw(e)),
            hpx_limit_kw: [0, 1]
                .map(|e| hpx_limit_kw(self.inputs.engine_rpm[e], self.inputs.engine_running[e])),
            pecs_coolant_c: self.pecs_c[0].max(self.pecs_c[1]),
            ext_phase_a: ["L EPC", "R EPC", "LAEPC"]
                .iter()
                .map(|n| {
                    self.plant
                        .contactor_phase_a(n)
                        .iter()
                        .cloned()
                        .fold(0.0, f64::max)
                })
                .fold(0.0, f64::max),
            ext_single_90: self.bpcu_out.ext_on.iter().filter(|b| **b).count() == 1,
        }
    }

    fn apply_managed_loads(&mut self) {
        use crate::control::load_mgmt::{CAC_LOAD_INDEX, DEFAULT_LOADS, HYD_LOAD_INDEX};
        let req = self.lm_requests(self.inputs.on_ground);
        let start_busy = self.crew_in.engine_start_in_progress.iter().any(|b| *b)
            || self.rig.engine.iter().any(|e| e.cranking)
            || self.rig.apu.cranking;
        for (k, d) in DEFAULT_LOADS.iter().enumerate() {
            let on = req[k] && !self.lm_out.shed[k];
            let mut kw = if on { d.kw } else { 0.0 };
            if k == CAC_LOAD_INDEX {
                kw *= self.lm_out.cac_speed;
            }
            let (names, n): (Vec<String>, f64) = match k {
                CAC_LOAD_INDEX => (
                    ["CMSC_CAC_L1", "CMSC_CAC_L2", "CMSC_CAC_R1", "CMSC_CAC_R2"]
                        .iter()
                        .map(|s| s.to_string())
                        .collect(),
                    4.0,
                ),
                HYD_LOAD_INDEX => (
                    ["CMSC_EMP_C1", "CMSC_EMP_C2"]
                        .iter()
                        .map(|s| s.to_string())
                        .collect(),
                    2.0,
                ),
                _ => (
                    ["L", "R"].iter().map(|s| format!("LM_{}_{s}", d.name)).collect(),
                    2.0,
                ),
            };
            if (k == CAC_LOAD_INDEX || k == HYD_LOAD_INDEX) && start_busy {
                self.managed_w[k] = [-1.0; 4];
                continue;
            }
            for (j, name) in names.iter().enumerate() {
                let per = kw * 1000.0 / n;
                let w = if k == CAC_LOAD_INDEX || k == HYD_LOAD_INDEX {
                    per.min(rig::CMSC_RATING_W)
                } else {
                    per
                };
                if (self.managed_w[k][j] - w).abs() > 1.0e-6 {
                    self.managed_w[k][j] = w;
                    self.plant.set_load(name, w);
                }
            }
        }
    }

    pub(crate) fn hpx_kw(&self, engine: usize) -> f64 {
        (0..2)
            .map(|j| {
                let m = engine * 2 + j;
                let (v, i, _) = self.plant.machine_phases(m);
                (0..3).map(|p| v[p] * i[p]).sum::<f64>()
            })
            .sum::<f64>()
            / APPROX_VFSG_EFFICIENCY
            / 1000.0
    }

    fn step_load_manager(&mut self, on_ground: bool) {
        let input = self.lm_input(on_ground);
        self.lm_out = self.lm.step(&input);
        self.apply_managed_loads();
    }

    fn step_cbic(&mut self, on_ground: bool) {
        let input = CbicInput {
            now_us: self.now_us,
            on_ground,
            ground_speed_kt: self.inputs.ground_speed_kt,
            doors_open: self.inputs.doors_open,
            ext_power_avail: self.ext_out.iter().any(|e| e.avail),
            gnd_test_enable: false,
        };
        self.cbic.app.update(&input);
    }

    fn contactor_command(&self, g: &Switchgear) -> bool {
        if g.forced_open {
            return false;
        }
        let o = &self.bpcu_out;
        let name = g.name.as_str();
        let ext_on_mode = o.ground_config == GroundConfig::ExtPowerOn;
        for k in 0..4 {
            if name == gcb_name(k) {
                return o.gcb_close[k] && self.gcu_out[k].gcb_close_permit && !self.gcu_out[k].gcb_trip_cmd;
            }
        }
        for k in 0..2 {
            if name == apb_name(k) {
                let u = &self.gcu_out[4 + k];
                return o.asg_cb_close[k] && u.gcb_close_permit && !u.gcb_trip_cmd;
            }
        }
        for (k, n) in crate::control::bpcu::BTB_NAMES.iter().enumerate() {
            if name == *n {
                return o.btb_close[k];
            }
        }
        match name {
            "L EPC" => o.epc_close[0] && self.ext_out[0].epc_close_permit && !self.epc_sup[0].chatter_lockout,
            "R EPC" => o.epc_close[1] && self.ext_out[1].epc_close_permit && !self.epc_sup[1].chatter_lockout,
            "LAEPC" => {
                o.epc_close[2]
                    && self.ext_out[2].epc_close_permit
                    && !self.epc_sup[2].chatter_lockout
                    && !self.gear_out("L2 ATRUC").contacts_closed
            }
            "L2 ATRUC" => !o.epc_close[2] && self.plant.bus_v("L2") > BUS_LIVE_V && o.sspc_elcc_loads_enabled,
            "L BSB" => {
                !o.bsb_fail_to_close[0]
                    && (o.bsb_close[0] || (ext_on_mode && o.bus_feed[0] == Feed::Src(Src::Ext(0))))
            }
            "R BSB" => {
                !o.bsb_fail_to_close[1]
                    && (o.bsb_close[1] || (ext_on_mode && o.bus_feed[2] == Feed::Src(Src::Ext(1))))
            }
            "L BTB 115" => o.btb115_close[0],
            "R BTB 115" => o.btb115_close[1],
            "RCB" => {
                o.rat_cb_close && self.rat.output().rcb_permit && !self.gear_out("BB ISO RLY").contacts_closed
            }
            "MBR" => o.main_batt_relay_close && !self.hbb_fault_opens_mbr(),
            "BB ISO RLY" => !o.bb_iso_open,
            "C1 TRU ISO R" => !o.c1_tru_iso_open,
            "C1 TRU RELAY" => o.c1_tru_iso_open && self.plant.bus_v("BB") > BUS_LIVE_V,
            "CIT" | "FIT" => o.instr_parallel || o.capt_inst == crate::control::bpcu::InstFeed::MainBattery,
            n if n.starts_with("AC FEED RPDU ") => {
                let left = n["AC FEED RPDU ".len()..]
                    .trim()
                    .parse::<u32>()
                    .map(|v| v % 2 == 1)
                    .unwrap_or(true);
                self.plant.bus_v(if left { "AC115_L" } else { "AC115_R" }) > BUS_LIVE_V
                    && o.sspc_elcc_loads_enabled
            }
            "SPUB" => self.rig.spub_closed_cmd(),
            "SPUC" | "SPU contactor" => self.rig.spu_closed_cmd(),
            "L ATUC" if self.atu_diff[0].tripped() => false,
            "R ATUC" if self.atu_diff[1].tripped() => false,
            "L ATUC" => {
                (ext_on_mode && o.bus_feed[0] == Feed::Src(Src::Ext(0)))
                    || self.plant.bus_v("L1") > BUS_LIVE_V
            }
            "R ATUC" => {
                (ext_on_mode && o.bus_feed[2] == Feed::Src(Src::Ext(1)))
                    || self.plant.bus_v("R1") > BUS_LIVE_V
            }
            _ => self.plant.bus_v(&g.upstream) > BUS_LIVE_V && o.sspc_elcc_loads_enabled,
        }
    }

    fn control_volts(&self) -> f64 {
        self.plant
            .battery_v(0)
            .max(self.plant.bus_v("HBB_MAIN"))
            .max(self.plant.bus_v("DC28_L"))
    }

    fn stage_contactors(&mut self, dt_us: u64) {
        let cv = self.control_volts();
        let mut cmds: Vec<bool> = self.gear.iter().map(|g| self.contactor_command(g)).collect();
        self.parallel_blocks = self.interlock(&mut cmds);
        let mut closed: Vec<(String, bool)> = Vec::new();
        for (g, cmd) in self.gear.iter_mut().zip(cmds) {
            g.cmd = cmd;
            let input = ContactorInput {
                dt_us,
                coil_v: if cmd { cv.min(COIL_NOMINAL_V) } else { 0.0 },
                current_a: self.plant.contactor_a(&g.name),
            };
            let line_v = self.plant.bus_v(&g.upstream);
            g.out = g.unit.step(&input, line_v);
            closed.push((g.name.clone(), g.out.contacts_closed));
        }
        for (name, c) in closed {
            self.plant.set_contactor(&name, c);
        }
    }

    fn stage_machines(&mut self, dt_us: u64) {
        for m in 0..6 {
            let rpm = self.shaft_rpm(m);
            self.plant.set_machine_rpm(m, rpm);
            self.drive_machine_exciter(m);
        }
        let (rat_v, rat_a) = self.plant.machine_sense(6);
        let rat_out = self.rat.step(&RatInput {
            dt_us,
            now_us: self.now_us,
            request: self.bpcu_out.rat_deploy || self.inputs.rat_deployed,
            on_ground: self.inputs.on_ground,
            host_rpm: self.inputs.rat_rpm,
            terminal_v: rat_v,
            current_a: rat_a,
            rcb_closed: self.gear_out("RCB").contacts_closed,
            gen_lost: false,
        });
        self.plant.set_machine_rpm(6, rat_out.rpm);
        self.drive_rat_exciter(rat_out.exciter_a, rat_out.field_on);
        for k in 0..3 {
            self.plant.set_ext(k, self.inputs.ext[k]);
        }
        let shed = self.lm_out.shed;
        self.plant.set_load_shed(&shed);
    }

    fn stage_solve(&mut self, dt_us: u64) {
        self.plant.step(dt_us, self.inputs.ambient_c);
    }

    fn stage_feedback(&mut self, dt_us: u64) {
        self.step_tru_oc(dt_us);
        self.step_zone_diffs(dt_us);
        self.step_atru_prot(dt_us);
        self.step_epc_supervisors(dt_us);
        self.step_pecs(dt_us);
        self.step_circuit_protection(dt_us);
        self.step_storage(dt_us);
        self.step_battery_monitor(dt_us);
        self.refresh_crew_inputs();
        self.eicas.update(&self.crew_in);
        let gen_valid = [true; 4];
        let bus_valid = [true; 4];
        self.synoptic = build_synoptic(&self.crew_in, gen_valid, bus_valid);
    }

    pub(crate) fn feeder_unbalance_a(&self, m: usize, breaker: &str) -> f64 {
        if m >= 4 {
            return 0.0;
        }
        let part = format!("W:{breaker}");
        let k = crate::faults::resistance_multiplier(self.fault_link.book.magnitude(
            &part,
            "high-resistance",
            self.now_us,
        ));
        if k <= 1.0 {
            return 0.0;
        }
        let i = self
            .plant
            .contactor_phase_a(breaker)
            .iter()
            .cloned()
            .fold(0.0, f64::max);
        i * (k - 1.0) / (k + 1.0)
    }

    pub(crate) fn dc_content_a(&self, m: usize) -> [f64; 3] {
        let v_ll = 3.0_f64.sqrt() * crate::sources::gcu::GcuParams::vfsg().regulated_v;
        let mut dc = 0.0;
        for (k, bus) in [
            (0, Some(0)),
            (1, Some(2)),
            (2, self.gear_out("C1 TRU ISO R").contacts_closed.then_some(1)),
        ] {
            let Some(b) = bus else {
                continue;
            };
            let name = crate::protection::tru_oc::TRU_NAMES[k];
            let mag = self.fault_link.book.magnitude(name, "diode-open", self.now_us);
            if mag <= 0.0 {
                continue;
            }
            let src = match self.feed_of(b) {
                Feed::Src(Src::Vfsg(g)) => g as usize,
                Feed::Src(Src::Asg(g)) => 4 + g as usize,
                _ => continue,
            };
            if src != m {
                continue;
            }
            let watts = self
                .plant
                .snapshot()
                .and_then(|s| s.converter(name))
                .map(|c| c.output_w)
                .unwrap_or(0.0);
            dc += mag.min(1.0) * watts / (OPEN_DIODE_DC_FACTOR * v_ll);
        }
        [dc, 0.0, 0.0]
    }

    pub(crate) fn tru_feed_v(&self, k: usize) -> f64 {
        match k {
            0 if self.gear_out("L TRU RLY").contacts_closed => self.plant.bus_v("L1"),
            1 if self.gear_out("R TRU RLY").contacts_closed => self.plant.bus_v("R1"),
            2 => self.tru_input_v(true),
            3 => self.tru_input_v(false),
            _ => 0.0,
        }
    }

    pub(crate) fn tru_output_a(&self, k: usize) -> f64 {
        self.plant
            .snapshot()
            .and_then(|s| s.converter(crate::protection::tru_oc::TRU_NAMES[k]))
            .map(|c| c.output_a)
            .unwrap_or(0.0)
    }

    pub(crate) fn zone_leak_a(&self, zone: &str) -> f64 {
        self.plant.snapshot().map(|s| s.leak_a(zone)).unwrap_or(0.0)
    }

    fn step_zone_diffs(&mut self, dt_us: u64) {
        use crate::protection::zone_diff::*;
        if self.control_volts() < CONTROL_POWER_MIN_V {
            self.tie_diff = [ZoneDiff::default(); 2];
            self.atu_diff = [ZoneDiff::default(); 2];
            return;
        }
        for k in 0..2 {
            let tie = self.zone_leak_a(["LTB", "RTB"][k]);
            self.tie_diff[k].step(tie, TIE_DIFF_A, TIE_DIFF_HW_US + TIE_DIFF_SW_US, dt_us);
            let atu = self.zone_leak_a(["ATU_L", "ATU_R"][k]);
            self.atu_diff[k].step(atu, ATU_DIFF_A, ATU_DIFF_US, dt_us);
        }
    }

    pub(crate) fn atru_reading(&self, name: &str) -> (f64, f64) {
        self.plant
            .snapshot()
            .and_then(|s| s.converter(name))
            .map(|c| (c.output_w, c.output_v))
            .unwrap_or((0.0, 0.0))
    }

    fn step_atru_prot(&mut self, dt_us: u64) {
        use crate::protection::atru_prot::*;
        let spu_start = self.gear_out("SPUB").contacts_closed;
        for k in 0..4 {
            let tag = ATRU_FEED_BUS[k];
            let on_115 = k == 1 && self.gear_out("LAEPC").contacts_closed;
            let (input_v, input_hz) = if self.gear_out(&format!("{tag} ATRUC")).contacts_closed {
                (self.plant.bus_v(tag), self.plant.bus_hz(tag))
            } else if on_115 {
                let e = &self.inputs.ext[2];
                (e.v_ll, e.hz)
            } else {
                (0.0, 0.0)
            };
            let (mut w, v) = self.atru_reading(ATRU_NAMES[k]);
            if k == 1 {
                w += self.atru_reading("ATRU_L2_115").0;
            }
            let input = AtruInput {
                dt_us,
                input_v: if k == 3 && spu_start { 0.0 } else { input_v },
                input_hz,
                on_115_input: on_115,
                load_kva: w / ATRU_EFFICIENCY_DERIVED / 1000.0,
                out_half_v: v / 2.0,
            };
            self.atru_prot[k].step(&input);
        }
    }

    fn step_pecs(&mut self, dt_us: u64) {
        let dt = dt_us as f64 * 1.0e-6;
        let ambient = self.inputs.ambient_c;
        let pumped = |b: &str| self.plant.bus_v(b) > BUS_LIVE_V;
        for k in 0..2 {
            let atrus = if k == 0 {
                ["ATRU_L1", "ATRU_R1"]
            } else {
                ["ATRU_L2", "ATRU_R2"]
            };
            let q: f64 = atrus
                .iter()
                .map(|n| self.atru_reading(n).0 * (1.0 / ATRU_EFFICIENCY_DERIVED - 1.0))
                .sum();
            let powered = if k == 0 {
                pumped("L1") || pumped("L2")
            } else {
                pumped("R1") || pumped("R2")
            };
            let part = ["PECS_1", "PECS_2"][k];
            let loss = self
                .fault_link
                .book
                .magnitude(part, "pump-loss", self.now_us)
                .min(1.0);
            let rise = self
                .fault_link
                .book
                .magnitude(part, "coolant-temp-rise", self.now_us)
                .min(1.0);
            let flow = if powered {
                (1.0 - loss) * (1.0 - 0.5 * rise)
            } else {
                0.0
            };
            let m_dot = ATRU_MIN_COOLANT_KG_PER_S * atrus.len() as f64 * flow;
            let supply = if flow > 0.0 { PECS_SUPPLY_C } else { ambient };
            let t = self.pecs_c[k];
            let t = if t.is_nan() { supply } else { t };
            let carried = m_dot * COOLANT_CP_J_PER_KG_K * (t - supply);
            let still = APPROX_PECS_STILL_W_PER_K * (t - ambient);
            self.pecs_c[k] = t + (q - carried - still) / APPROX_PECS_HEAT_CAP_J_PER_K * dt;
        }
    }

    fn step_epc_supervisors(&mut self, dt_us: u64) {
        for (k, name) in ["L EPC", "R EPC", "LAEPC"].iter().enumerate() {
            let commanded = self.gear.iter().find(|g| g.name == *name).is_some_and(|g| g.cmd);
            let sense = crate::protection::epc_supervisor::EpcSense {
                now_us: self.now_us,
                dt_us,
                plugged: self.inputs.ext[k].connected,
                commanded,
                aux_closed: self.gear_out(name).aux_no,
                current_a: self
                    .plant
                    .contactor_phase_a(name)
                    .iter()
                    .cloned()
                    .fold(0.0, f64::max),
                voltage_v: if self.inputs.ext[k].connected {
                    let v = self.causal_ext_phase_v(k, self.inputs.ext[k].v_ll);
                    (v[0] + v[1] + v[2]) / 3.0
                } else {
                    0.0
                },
                aft: k == 2,
            };
            self.epc_sup[k].step(&sense);
            if self.epc_sup[k].fwd_off_cmd {
                self.ext_switch_on[k] = false;
            }
        }
    }

    fn step_battery_monitor(&mut self, dt_us: u64) {
        let on = self.panel_out.battery_on;
        let cycled_off = self.batt_switch_prev && !on;
        self.batt_switch_prev = on;
        let input = crate::storage::monitor::MonitorInput {
            dt_us,
            discharge_a: self.plant.battery_a(0),
            battery_v: self.plant.battery_v(0),
            hbb_v: self.plant.bus_v("HBB_MAIN"),
            transfer_pending: self.bpcu_out.transfer_pending,
            switch_cycled_off: cycled_off,
        };
        self.batt_mon_out = self.batt_mon.step(&input);
    }

    pub(crate) fn hbb_fault_opens_mbr(&self) -> bool {
        let o = &self.batt_mon_out;
        self.inputs.on_ground && !self.bpcu_out.rat_deploy && (o.hbb_uv_fault || o.hbb_ov_fault)
    }

    fn step_tru_oc(&mut self, dt_us: u64) {
        for k in 0..4 {
            let amps = self.tru_output_a(k);
            let feed = self.tru_feed_v(k);
            self.tru_oc[k].step(amps, feed, dt_us);
        }
    }

    fn step_storage(&mut self, dt_us: u64) {
        for b in 0..2 {
            let mut cells = [0.0; MAX_CELLS];
            let n = self.plant.battery_cells(b, &mut cells);
            let (bmu_fail, volt_off, temp_bias) = self.causal_bmu(b);
            let (chg_fault, chg_inhibit, chg_scale) = self.causal_charger(b);
            for v in cells.iter_mut().take(n) {
                *v += volt_off;
            }
            let input = BmuInput {
                dt_us,
                cells: n,
                cell_v: cells,
                temp_c: self.plant.battery_temp_c(b) + temp_bias,
                current_a: self.plant.battery_a(b),
                reset: false,
                bmu_internal_fail: bmu_fail,
            };
            self.bmu_out[b] = self.bmu[b].step(&input);
            let (emf, r) = self.plant.battery_thevenin(b);
            let ci = ChargerInput {
                dt_us,
                input_v: if b == 0 {
                    self.plant.bus_v("DC28_L")
                } else {
                    self.plant.bus_v("HBB_APU")
                },
                enable: true,
                inhibit: chg_inhibit,
                bmu_inhibit: self.bmu_out[b].charge_inhibit,
                battery_connected: !self.bmu_out[b].disconnect,
                battery_emf_v: emf,
                battery_r_ohm: r,
                battery_temp_c: self.plant.battery_temp_c(b),
                internal_fault: chg_fault,
            };
            self.charger_out[b] = self.chargers[b].step(&ci);
            self.plant
                .set_charger(b, self.charger_out[b].charge_current_a * chg_scale);
        }
    }

    pub(crate) fn refresh_crew_inputs(&mut self) {
        let on_ground = self.inputs.on_ground;
        let phase: FlightPhase = classify_phase(
            self.inputs.ground_speed_kt,
            self.inputs.radio_alt_ft,
            on_ground,
            false,
        );
        let mut c = std::mem::take(&mut self.crew_in);
        c.now_us = self.now_us;
        c.phase = phase;
        c.on_ground = on_ground;
        c.engine_running = self.inputs.engine_running;
        c.fuel_run = self.inputs.engine_running;
        c.battery_switch_on = self.panel_out.battery_on;
        c.apu_gen_switch_on = self.panel_out.apu_gen_on;
        c.gen_ctrl_on = self.panel_out.gen_ctrl_on;
        c.gen_ctrl_cycled = self.panel_out.gen_ctrl_cycled;
        c.cabin_util_on = self.panel_out.cabin_util_on;
        c.ife_on = self.panel_out.ife_on;
        for k in 0..4 {
            c.gcb_closed[k] = self.gear_out(&gcb_name(k)).contacts_closed;
            let gen_fault =
                self.drive_disconnected[k] || self.gcu_out[k].trip_latched || self.faults.gen_loss[k];
            let fault_open =
                gen_fault && self.inputs.engine_running[k / 2] && !c.engine_start_in_progress[k / 2];
            c.gcb_prevented[k] =
                !self.gcu_out[k].gcb_close_permit && self.panel_out.gen_ctrl_on[k] && !fault_open;
            c.drive_disconnected[k] = self.drive_disconnected[k];
            c.protective_trip[k] = self.gcu_out[k].trip_latched;
            c.pmg_lost[k] = false;
            c.vfsg_oil_high[k] = self.plant.net().machine_oil_temp_c(MACHINES[k]).is_some_and(|t| {
                t > crate::sources::machine::f_to_c(crate::sources::machine::VFSG_OIL_NORMAL_F.1)
            });
            let v = self.plant.bus_v(BUS_NAMES[k]);
            c.ac_bus_volts[k] = v;
            c.ac_bus_energized[k] = v > BUS_LIVE_V;
            let (gv, _) = self.plant.machine_sense(k);
            c.gen_volts[k] = gv;
            c.gen_hz[k] = self.plant.machine_hz(k);
        }
        for k in 0..2 {
            c.apb_closed[k] = self.gear_out(&apb_name(k)).contacts_closed;
            c.apb_cmd_closed[k] = self.bpcu_out.asg_cb_close[k];
            c.apu_gen_available[k] = self.gcu_out[4 + k].voltage_established;
            let (gv, _) = self.plant.machine_sense(4 + k);
            c.apu_gen_volts[k] = gv;
            c.apu_gen_hz[k] = self.plant.machine_hz(4 + k);
            c.ac115_volts[k] = self.plant.bus_v(["AC115_L", "AC115_R"][k]);
            c.dc28_volts[k] = self.plant.bus_v(["DC28_L", "DC28_R"][k]);
            c.ext_fwd_inserted[k] = self.inputs.ext[k].connected;
            c.ext_fwd_quality_ok[k] = self.ext_out[k].power_quality_good;
            c.ext_fwd_epc_closed[k] = self.gear_out(["L EPC", "R EPC"][k]).contacts_closed;
        }
        c.apu_running = self.inputs.apu_running;
        c.ext_aft_inserted = self.inputs.ext[2].connected;
        c.ext_aft_quality_ok = self.ext_out[2].power_quality_good;
        c.ext_aft_epc_closed = self.gear_out("LAEPC").contacts_closed;
        c.apu_start_in_progress = self.apu_start_latch_us.is_some() && !self.inputs.apu_running;
        c.main_batt_volts = self.plant.battery_v(0);
        c.main_batt_current_a = self.plant.battery_a(0);
        c.apu_batt_volts = self.plant.battery_v(1);
        c.hot_batt_volts = self.plant.bus_v("HBB_MAIN");
        c.main_batt_disch_msg = self.batt_mon_out.disch_message;
        c.hot_batt_bus_energized = c.hot_batt_volts > 22.0;
        c.rat_deployed = self.rat.deployed();
        c.ground_handling_mode = self.bpcu_out.ground_config == GroundConfig::GroundHandling;
        c.ground_service_on = self.ground_service_switch;
        c.ac_source_online = self.bpcu_out.bus_powered.iter().any(|b| *b);
        c.backup_volts = self.plant.bus_v("BB");
        c.flt_inst_capt_volts = self.plant.bus_v("CIB");
        c.flt_inst_fo_volts = self.plant.bus_v("FIB");
        c.bpcu_inop = [!self.bpcu.left_powered, !self.bpcu.right_powered];
        c.load_shed = self
            .lm_out
            .shed
            .iter()
            .enumerate()
            .filter(|(_, s)| **s)
            .map(|(k, _)| crate::control::DEFAULT_LOADS[k].name.to_string())
            .collect();
        self.causal_crew_inputs(&mut c);
        self.crew_in = c;
    }

    pub(crate) fn feed_of(&self, bus: usize) -> Feed {
        self.bpcu_out.bus_feed[bus]
    }

    pub fn mode(&self) -> &'static str {
        let any_ac = (0..4).any(|k| self.plant.bus_v(BUS_NAMES[k]) > BUS_LIVE_V);
        let batt = self.plant.battery_v(0) > 18.0 && self.panel_out.battery_on;
        match self.bpcu_out.ground_config {
            GroundConfig::ExtPowerOn => "external_power_on",
            GroundConfig::GroundHandling => "ground_handling",
            _ => {
                if any_ac {
                    "generator"
                } else if self.inputs.on_ground {
                    if batt {
                        "on_ground_battery_only"
                    } else {
                        "cold_dark"
                    }
                } else if self.bpcu_out.rat_cb_close && self.plant.bus_v("BB") > BUS_LIVE_V {
                    "in_air_rat_only"
                } else if batt {
                    "in_air_battery_only"
                } else {
                    "cold_dark"
                }
            }
        }
    }
}

pub const BUS_NAMES: [&str; 4] = ["L1", "L2", "R1", "R2"];

pub(crate) fn gcb_name(k: usize) -> String {
    format!("{} GCB", BUS_NAMES[k])
}

pub(crate) fn apb_name(k: usize) -> String {
    ["L APB", "R APB"][k].to_string()
}

pub(crate) fn receptacle(k: usize) -> Receptacle {
    [Receptacle::LeftFwd, Receptacle::RightFwd, Receptacle::Aft][k]
}

pub(crate) fn new_cbic() -> CbicCluster {
    CbicCluster::new(Cbic::new(), 1)
}
