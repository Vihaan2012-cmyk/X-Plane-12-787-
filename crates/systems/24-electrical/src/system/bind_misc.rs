use super::SignalValue;
use super::SignalValue::{Bool, Num, Text};
use super::{BUS_LIVE_V, BUS_NAMES, Electrical};
use crate::control::bpcu::{BTB_NAMES, BTB_NODES, Src};
use crate::control::{Feed, InstFeed};
use crate::protection::contactor::ContactorConfig;
use crate::sources::{EP_RECOMMENDED_SOURCE_VA, POWERED_TICK_LIMIT, POWERED_TICK_US, RAT_BREAKER_CLOSE_HZ};
use crate::storage::{ChargerMode, MAX_CELLS, ocv_from_soc};

const DC_LIVE_V: f64 = 18.0;
const ATRU_115_INPUT_CLASS_KVA: f64 = 65.0;
const SPU_OUT_VAC: f64 = 115.0;
const SPU_OUT_VDC: f64 = 270.0;

fn extra_bus(alias: &str) -> Option<(&'static str, bool)> {
    Some(match alias {
        "L2_270vdc" => ("HVDC_L2", false),
        "L1_270vdc" => ("HVDC_L1", false),
        "R1_270vdc" => ("HVDC_R1", false),
        "R2_270vdc" => ("HVDC_R2", false),
        "L_tie" => ("LTB", true),
        "R_tie" => ("RTB", true),
        "captain_instr" => ("CIB", false),
        "fo_instr" => ("FIB", false),
        "dc28_l" => ("DC28_L", false),
        "dc28_r" => ("DC28_R", false),
        "hbb_apu" => ("HBB_APU", false),
        _ => return None,
    })
}

fn main_bus_index(alias: &str) -> Option<usize> {
    ["L1_235", "L2_235", "R1_235", "R2_235"]
        .iter()
        .position(|n| *n == alias)
}

fn side(s: &str) -> Option<usize> {
    match s {
        "l" => Some(0),
        "r" => Some(1),
        _ => None,
    }
}

fn gen_of(s: &str) -> Option<usize> {
    ["l1", "l2", "r1", "r2"].iter().position(|n| *n == s)
}

fn inst_text(f: InstFeed) -> &'static str {
    match f {
        InstFeed::None => "none",
        InstFeed::MainDc => "main_dc",
        InstFeed::MainBattery => "main_battery",
        InstFeed::BackupBus => "backup_bus",
    }
}

impl Electrical {
    fn closed(&self, name: &str) -> bool {
        self.contactor_closed(name).unwrap_or(false)
    }

    pub(crate) fn tru_input_v(&self, c1: bool) -> f64 {
        let bb = self.plant.bus_v("BB");
        if !c1 || self.closed("C1 TRU RELAY") {
            bb
        } else if self.closed("C1 TRU ISO R") {
            self.plant.bus_v("L2")
        } else {
            0.0
        }
    }

    fn transfer_gap_s(&self) -> f64 {
        let c = ContactorConfig::default();
        (c.release_us + c.operate_us) as f64 * 1.0e-6
    }

    fn machine_kva(&self, m: usize) -> f64 {
        let (v, i) = self.plant.machine_sense(m);
        3.0 * v * i / 1000.0
    }

    fn battery_soc_pct(&self, b: usize) -> f64 {
        let (emf, _) = self.plant.battery_thevenin(b);
        let mut cells = [0.0; MAX_CELLS];
        let n = self.plant.battery_cells(b, &mut cells).max(1);
        let target = emf / n as f64;
        let (mut lo, mut hi) = (0.0_f64, 1.0_f64);
        for _ in 0..40 {
            let mid = 0.5 * (lo + hi);
            if ocv_from_soc(mid) < target {
                lo = mid;
            } else {
                hi = mid;
            }
        }
        100.0 * 0.5 * (lo + hi)
    }

    fn inst_gap_ms(&self, f: InstFeed) -> f64 {
        match f {
            InstFeed::MainDc => 0.0,
            InstFeed::None => f64::INFINITY,
            _ => self.transfer_gap_s() * 1000.0,
        }
    }

    fn misc_bus(&self, alias: &str, what: &str) -> Option<SignalValue> {
        match what {
            "dropout_s" => {
                let k = main_bus_index(alias)?;
                let own = Feed::Src(Src::Vfsg(k as u8));
                let feed = self.feed_of(k);
                if self.plant.bus_v(BUS_NAMES[k]) <= BUS_LIVE_V {
                    Some(Num(f64::INFINITY))
                } else if feed == own || feed == Feed::None {
                    Some(Num(0.0))
                } else {
                    Some(Num(self.transfer_gap_s()))
                }
            }
            "interrupt_ms" => match alias {
                "captain_instr" => Some(Num(self.inst_gap_ms(self.bpcu_out.capt_inst))),
                "fo_instr" => Some(Num(self.inst_gap_ms(self.bpcu_out.fo_inst))),
                _ => None,
            },
            "separated_from_R_115" => {
                if alias == "L_115" {
                    Some(Bool(!(self.closed("L BTB 115") && self.closed("R BTB 115"))))
                } else {
                    None
                }
            }
            "energized" | "voltage" | "source" => {
                let (bus, ac) = extra_bus(alias)?;
                let v = self.plant.bus_v(bus);
                match what {
                    "energized" => Some(Bool(v > if ac { BUS_LIVE_V } else { DC_LIVE_V })),
                    "voltage" => Some(Num(v)),
                    _ => match alias {
                        "captain_instr" => Some(Text(inst_text(self.bpcu_out.capt_inst).to_string())),
                        "fo_instr" => Some(Text(inst_text(self.bpcu_out.fo_inst).to_string())),
                        _ => None,
                    },
                }
            }
            _ => None,
        }
    }

    pub(crate) fn bound_misc(&self, name: &str) -> Option<SignalValue> {
        let parts: Vec<&str> = name.split('.').collect();
        if parts.first() != Some(&"elec") {
            return None;
        }
        match parts.as_slice() {
            ["elec", "bus", alias, what] => self.misc_bus(alias, what),
            ["elec", "aft_ep", "path"] => Some(Text(
                if self.closed("LAEPC") { "L2_ATRU" } else { "none" }.to_string(),
            )),
            ["elec", "apu_batt", what] => match *what {
                "voltage" => Some(Num(if self.inputs.apu_battery_connected {
                    self.plant.battery_v(1)
                } else {
                    0.0
                })),
                "charging" => Some(Bool(
                    self.inputs.apu_battery_connected
                        && self.charger_out[1].mode != ChargerMode::Off
                        && self.charger_out[1].mode != ChargerMode::Fault,
                )),
                "attempt_limit_s" => Some(Num(self.rig.attempt_limit_s())),
                "attempts_available" => Some(Num(
                    if self.inputs.apu_battery_connected
                        && !self.bmu_out[1].disconnect
                        && self.plant.battery_v(1) > DC_LIVE_V
                    {
                        self.rig.attempts_available()
                    } else {
                        0.0
                    },
                )),
                _ => None,
            },
            ["elec", "apu_ready_to_load"] => Some(Bool(
                self.inputs.apu_running
                    && (self.gcu_out[4].voltage_established || self.gcu_out[5].voltage_established),
            )),
            ["elec", "asg", which, what] => {
                let k = 4 + side(which)?;
                match *what {
                    "freq_hz" => Some(Num(self.plant.machine_hz(k))),
                    "load_kva" => Some(Num(self.machine_kva(k))),
                    "voltage_vll" => Some(Num(self.plant.machine_sense(k).0)),
                    _ => None,
                }
            }
            ["elec", "atru_r2", "output_vdc_each_leg"] => Some(Num(self.plant.bus_v("HVDC_R2") / 2.0)),
            ["elec", "batt_main", what] => match *what {
                "discharge_current" => Some(Num(self.plant.battery_a(0).max(0.0))),
                "soc_pct" => Some(Num(self.battery_soc_pct(0))),
                _ => None,
            },
            ["elec", "bb_iso_rly", "open"] => Some(Bool(!self.closed("BB ISO RLY"))),
            ["elec", "bpcu", which, "standby_mode"] => {
                let k = side(which)?;
                let powered = if k == 0 {
                    self.bpcu.left_powered
                } else {
                    self.bpcu.right_powered
                };
                Some(Bool(powered && self.bpcu_out.all_ac_lost))
            }
            ["elec", "bus_tie", which, "closed"] => {
                let bus = match *which {
                    "btb_l1" => 0,
                    "btb_l2" => 1,
                    "btb_r1" => 2,
                    "btb_r2" => 3,
                    "btb_l3" => return Some(Bool(self.closed(BTB_NAMES[4]))),
                    "btb_r3" => return Some(Bool(self.closed(BTB_NAMES[5]))),
                    _ => return None,
                };
                Some(Bool((0..BTB_NAMES.len()).any(|k| {
                    (BTB_NODES[k].0 == bus || BTB_NODES[k].1 == bus) && self.closed(BTB_NAMES[k])
                })))
            }
            ["elec", "bus_tie_115", "both_closed"] => {
                Some(Bool(self.closed("L BTB 115") && self.closed("R BTB 115")))
            }
            ["elec", "cbic", what] => match *what {
                "battery_ground_open_reset_allowed" => {
                    Some(Bool(self.inputs.on_ground && self.control_volts() > DC_LIVE_V))
                }
                "cmd_rejected_tcb" => Some(Bool(self.faults.cbic_tcb_rejected)),
                "main_bus_elcc_controlled" => Some(Bool(!self.gear.iter().any(|g| g.name == BTB_NAMES[0]))),
                _ => None,
            },
            ["elec", "condition", "apu_started_required"] => Some(Bool(
                self.plant.bus_v(BUS_NAMES[1]) <= BUS_LIVE_V || self.plant.bus_v(BUS_NAMES[3]) <= BUS_LIVE_V,
            )),
            ["elec", "disc_light", which] => {
                let k = gen_of(which.strip_prefix("drive_")?)?;
                Some(Bool(self.lamps.drive[k].lit))
            }
            ["elec", "drive", "reconnect_inflight"] => Some(Text(
                if self.inputs.on_ground {
                    "ground_maintenance"
                } else {
                    "not_possible"
                }
                .to_string(),
            )),
            ["elec", "ext_pwr", which, "kva_limit"] => match *which {
                "fwd_l" | "fwd_r" => Some(Num(EP_RECOMMENDED_SOURCE_VA / 1000.0)),
                "aft" => Some(Num(ATRU_115_INPUT_CLASS_KVA)),
                _ => None,
            },
            ["elec", "gcu", "counter_limit_days"] => {
                Some(Num((POWERED_TICK_LIMIT * POWERED_TICK_US) as f64 / 86_400.0e6))
            }
            ["elec", "gcu", "counter_limit_ticks"] => Some(Num(POWERED_TICK_LIMIT as f64)),
            ["elec", "gcu", "simultaneous_loss_all_four"] => {
                Some(Bool((0..4).all(|k| self.gcu_out[k].trip_latched)))
            }
            ["elec", "gcu", which, "uptime_days"] => {
                let k = gen_of(which)?;
                Some(Num(self.gcus[k].powered_time_us() as f64 / 86_400.0e6))
            }
            ["elec", "gec1", which, "closed"] => Some(Bool(self.rig.vfsg[gen_of(which)?].gec1)),
            ["elec", "gec3", which, "closed"] => Some(Bool(self.rig.vfsg[gen_of(which)?].gec3)),
            ["elec", "gen_neutral_relay", which, "open"] => {
                Some(Bool(self.rig.vfsg[gen_of(which)?].gnr_open))
            }
            ["elec", "ground_handling_loads", "freq_hz_max"] => {
                let epc = ["L EPC", "R EPC", "LAEPC"];
                let mut hz = 0.0_f64;
                for k in 0..3 {
                    if self.inputs.ext[k].connected && self.closed(epc[k]) {
                        hz = hz.max(self.inputs.ext[k].hz);
                    }
                }
                Some(Num(hz))
            }
            ["elec", "mmel", "apu_required_with_vfsg_inop"] => Some(Bool((0..4).any(|k| {
                self.faults.gen_loss[k] || self.drive_disconnected[k] || self.gcu_out[k].trip_latched
            }))),
            ["elec", "rat", "deploy_command_delay_ms"] => Some(Num(self.rat.command_delay_ms())),
            ["elec", "rat_gen", what] => match *what {
                "load_kva" => Some(Num(self.machine_kva(6))),
                "voltage_vll" => Some(Num(self.plant.rat_volts())),
                "min_rcb_close_hz" => Some(Num(RAT_BREAKER_CLOSE_HZ)),
                _ => None,
            },
            ["elec", "rcb", "closed_requires_bb_iso_rly_open"] => {
                Some(Bool(!self.closed("RCB") || !self.closed("BB ISO RLY")))
            }
            ["elec", "shutdown", "active"] => Some(Bool(!self.panel_out.battery_on && self.closed("MBR"))),
            ["elec", "spu", "output_vac"] => Some(Num(if self.closed("SPUB") {
                SPU_OUT_VAC * (self.plant.bus_v("HVDC_R2") / SPU_OUT_VDC).min(1.0)
            } else {
                0.0
            })),
            ["elec", "standby_buses", "paralleled_ms"] => Some(Num(if self.bpcu_out.instr_parallel {
                ContactorConfig::default().operate_us as f64 / 1000.0
            } else {
                0.0
            })),
            ["elec", "start_switch", which] => {
                let k = side(which)?;
                Some(Text(
                    if self.crew_in.engine_start_in_progress[k] {
                        "START"
                    } else {
                        "NORM"
                    }
                    .to_string(),
                ))
            }
            ["elec", "transfer", "no_bus_dropout"] => {
                Some(Bool((0..4).all(|k| self.plant.bus_v(BUS_NAMES[k]) > BUS_LIVE_V)))
            }
            ["elec", "vfsg", "remaining_load_kva_max"] => {
                let mut m = 0.0_f64;
                for k in 0..4 {
                    if self.closed(&format!("{} GCB", BUS_NAMES[k])) {
                        m = m.max(self.machine_kva(k));
                    }
                }
                Some(Num(m))
            }
            ["elec", "vfsg", "start_torque_ftlb"] => Some(Num(self.rig.start_torque_ftlb())),
            ["elec", "vfsg", which, "mode_code"] => {
                let mode = self.rig.vfsg[gen_of(which)?].mode;
                Some(Num(super::rig::VFSG_MODES.iter().position(|m| *m == mode)? as f64))
            }
            ["elec", "vfsg", which, "oil_normal"] => {
                let t = self
                    .plant
                    .net()
                    .machine_oil_temp_c(super::MACHINES[gen_of(which)?])?;
                let (lo, hi) = crate::sources::machine::VFSG_OIL_NORMAL_F;
                let t_f = crate::sources::machine::c_to_f(t);
                Some(Bool(t_f >= lo && t_f <= hi))
            }
            ["elec", "vfsg", which, "cooldown_s"] => Some(Num(self.rig.vfsg[gen_of(which)?].cooldown_s)),
            ["elec", "vfsg", which, "drive_disconnected"] => {
                Some(Bool(self.drive_disconnected[gen_of(which)?]))
            }
            ["elec", "vfsg", which, "drive_solenoid_damaged"] => {
                Some(Bool(self.drive_sol[gen_of(which)?].damaged))
            }
            ["elec", "engine", which, "start_inhibited"] => {
                Some(Bool(self.rig.start_inhibited[side(which)?]))
            }
            ["elec", "vfsg", which, "start_time_s"] => Some(Num(self.rig.start_time_s(side(which)?))),
            ["elec", "gec2", which, "closed"] => Some(Bool(self.rig.vfsg[gen_of(which)?].gec2)),
            ["elec", "start_contactor", which, "closed"] => {
                Some(Bool(self.rig.vfsg[gen_of(which)?].start_contactor))
            }
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::super::Electrical;
    use crate::control::CbState;

    fn sys() -> Electrical {
        let dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../../aircraft/24-electrical");
        Electrical::new(&dir).expect("system builds")
    }

    fn run(e: &mut Electrical, seconds: f64) {
        for _ in 0..(seconds * 100.0) as u64 {
            e.step_idle(10_000);
        }
    }

    fn preset(e: &mut Electrical, key: &str, value: &str) {
        e.action("sim", "set", key, Some(value)).unwrap();
    }

    fn cruise(e: &mut Electrical) {
        for (k, v) in [
            ("airplane_state", "cruise"),
            ("alt_ft", "35000.0"),
            ("battery_switch", "ON"),
            ("engines_running", "[true, true]"),
            ("gen_ctrl_switches", "ON"),
            ("on_ground", "false"),
            ("oat_c", "-50.0"),
        ] {
            preset(e, k, v);
        }
    }

    #[test]
    fn presets_drive_inputs_and_bound_signals() {
        let mut e = sys();
        cruise(&mut e);
        assert!(!e.inputs().on_ground);
        assert_eq!(e.inputs().engine_running, [true, true]);
        assert_eq!(e.inputs().ambient_c, -50.0);
        assert_eq!(e.inputs().alt_ft, 35000.0);
        run(&mut e, 3.0);
        assert_eq!(e.flag("elec.bus.L1_235.energized"), Some(true));
        assert_eq!(e.flag("elec.bus.L_tie.energized"), Some(false));
        assert_eq!(e.flag("elec.bus.captain_instr.energized"), Some(true));
        assert_eq!(e.flag("elec.transfer.no_bus_dropout"), Some(true));
        assert!(e.signal("elec.vfsg.l1.load_kva").is_some());
        let soc = e.signal("elec.batt_main.soc_pct").unwrap();
        assert!(soc > 0.0 && soc <= 100.0);
    }

    #[test]
    fn engine_loss_changes_bus_state_bindings() {
        let mut e = sys();
        cruise(&mut e);
        run(&mut e, 3.0);
        assert_eq!(e.flag("elec.bus.L2_235.energized"), Some(true));
        assert_eq!(e.flag("elec.condition.apu_started_required"), Some(false));
        e.action("fault", "gen_loss", "ALL_VFSG", None).unwrap();
        e.action("fault", "all_engines_flameout", "ALL", None).unwrap();
        run(&mut e, 1.0);
        assert_eq!(e.flag("elec.bus.L2_235.energized"), Some(false));
        assert_eq!(e.flag("elec.condition.apu_started_required"), Some(true));
        assert_eq!(e.flag("elec.mmel.apu_required_with_vfsg_inop"), Some(true));
        assert_eq!(e.flag("elec.gcb.l1.closed"), Some(false));
    }

    #[test]
    fn gcu_uptime_preset_and_power_cycle() {
        let mut e = sys();
        preset(&mut e, "gcu.r2.uptime_days", "100.0");
        let d = e.signal("elec.gcu.r2.uptime_days").unwrap();
        assert!((d - 100.0).abs() < 0.01);
        let limit = e.signal("elec.gcu.counter_limit_days").unwrap();
        assert!((limit - 248.55).abs() < 0.01);
        assert_eq!(e.signal("elec.gcu.counter_limit_ticks"), Some(2_147_483_648.0));
        e.action("crew", "cycle_power", "ALL_GCU", None).unwrap();
        assert!(e.signal("elec.gcu.r2.uptime_days").unwrap() < 1.0);
    }

    #[test]
    fn ground_power_actions_close_epc_and_trip_on_frequency() {
        let mut e = sys();
        preset(&mut e, "battery_switch", "ON");
        e.action(
            "ground",
            "connect",
            "EXT_PWR_FWD_L",
            Some("{ v_ll = 115.0, hz = 400.0, kva = 90.0 }"),
        )
        .unwrap();
        e.action("crew", "press", "EXT_PWR_FWD_L", Some("ON")).unwrap();
        run(&mut e, 8.0);
        assert_eq!(e.flag("elec.epc.fwd_l.closed"), Some(true));
        assert!(e.signal("elec.ground_handling_loads.freq_hz_max").unwrap() < 440.0);
        e.action("ground", "set", "EXT_PWR_FWD_L.hz", Some("430.0"))
            .unwrap();
        run(&mut e, 3.0);
        assert_eq!(e.flag("elec.epc.fwd_l.locked_out"), Some(true));
        assert_eq!(e.flag("elec.epc.fwd_l.closed"), Some(false));
        e.action("ground", "set", "EXT_PWR_FWD_L.v_worst_phase_rms", Some("131.0"))
            .unwrap();
        assert_eq!(e.inputs().ext[0].v_ll, 131.0);
    }

    #[test]
    fn cbic_actions_open_lock_and_reject_tcb() {
        let mut e = sys();
        preset(&mut e, "battery_switch", "ON");
        preset(&mut e, "ext_power_on", "true");
        run(&mut e, 5.0);
        e.action("crew", "open_page", "CBIC", None).unwrap();
        assert!(e.inputs().cbic_session.is_some());
        e.action("crew", "cbic_open", "CE1471831", None).unwrap();
        assert_eq!(
            e.cbic().app.breaker(1471831).map(|b| b.state),
            Some(CbState::Open)
        );
        e.action("crew", "cbic_lock", "CE1471831", None).unwrap();
        assert!(matches!(
            e.cbic().app.breaker(1471831).map(|b| b.state),
            Some(CbState::Locked(_))
        ));
        e.action("crew", "cbic_close", "CE1471831", None).unwrap();
        assert!(matches!(
            e.cbic().app.breaker(1471831).map(|b| b.state),
            Some(CbState::Locked(_))
        ));
        assert_eq!(e.flag("elec.cbic.cmd_rejected_tcb"), Some(false));
        e.action("crew", "cbic_open", "CT3209101", None).unwrap();
        assert_eq!(e.flag("elec.cbic.cmd_rejected_tcb"), Some(true));
        assert_eq!(e.flag("elec.cbic.main_bus_elcc_controlled"), Some(false));
    }

    #[test]
    fn faults_and_switches_are_accepted() {
        let mut e = sys();
        e.action("fault", "bpcu_fail", "L_AND_R", None).unwrap();
        assert_eq!(e.faults_mut().bpcu_fail, [true, true]);
        e.action("fault", "vfsg_fail", "L1", Some("drive_disconnect"))
            .unwrap();
        assert!(e.faults_mut().gen_loss[0]);
        assert_eq!(e.flag("elec.mmel.apu_required_with_vfsg_inop"), Some(true));
        e.action("crew", "rotate", "ENG_START_L", Some("START")).unwrap();
        run(&mut e, 0.1);
        assert_eq!(e.text("elec.start_switch.l").as_deref(), Some("START"));
        assert_eq!(e.flag("elec.gec1.l1.closed"), Some(true));
        assert_eq!(e.flag("elec.gen_neutral_relay.l1.open"), Some(true));
        e.action("crew", "move", "FUEL_CONTROL_L", Some("RUN")).unwrap();
        assert_eq!(e.text("elec.start_switch.l").as_deref(), Some("START"));
        assert!(!e.inputs().engine_running[0]);
        e.action("crew", "select", "APU_SWITCH", Some("START")).unwrap();
        e.action("crew", "start", "APU", None).unwrap();
        e.action("crew", "press", "APU_GEN_L", Some("OFF")).unwrap();
        e.action("crew", "press", "GEN_CTRL_L1", Some("OFF_ON")).unwrap();
        e.action("sim", "set", "load.total_kva_on_ext_pwr_fwd_l", Some("100.0"))
            .unwrap();
        assert_eq!(e.inputs().ext_load_kva[0], 100.0);
        assert!(e.action("sim", "set", "bogus_key", Some("1")).is_err());
    }

    #[test]
    fn drive_disconnect_lights_lamp_and_ring_tie_picks_up_bus() {
        let mut e = sys();
        cruise(&mut e);
        run(&mut e, 3.0);
        assert_eq!(e.flag("elec.disc_light.drive_l1"), Some(false));
        assert_eq!(e.flag("elec.bus_tie.btb_l1.closed"), Some(false));
        e.action("fault", "vfsg_fail", "L1", Some("drive_disconnect"))
            .unwrap();
        run(&mut e, 1.0);
        assert_eq!(e.flag("elec.disc_light.drive_l1"), Some(true));
        assert_eq!(e.flag("elec.bus.L1_235.energized"), Some(true));
        assert_eq!(e.flag("elec.bus_tie.btb_l1.closed"), Some(true));
        assert_eq!(
            e.text("elec.drive.reconnect_inflight").as_deref(),
            Some("not_possible")
        );
        assert!(e.signal("elec.bus.L1_235.dropout_s").unwrap() < 0.5);
    }
}
