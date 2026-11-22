use super::{BUS_LIVE_V, BUS_NAMES, Electrical};
use crate::control::Feed;
use crate::control::bpcu::Src;

#[derive(Clone, Debug, PartialEq)]
pub enum SignalValue {
    Bool(bool),
    Num(f64),
    Text(String),
}

fn bus_alias(name: &str) -> Option<&'static str> {
    Some(match name {
        "L1_235" => "L1",
        "L2_235" => "L2",
        "R1_235" => "R1",
        "R2_235" => "R2",
        "L_115" => "AC115_L",
        "R_115" => "AC115_R",
        "backup" => "BB",
        "c1_tru" => "TRU_C1",
        "c2_tru" => "TRU_C2",
        "hbb_main" => "HBB_MAIN",
        "ltb" => "LTB",
        "rtb" => "RTB",
        "dc28_l" => "DC28_L",
        "dc28_r" => "DC28_R",
        _ => return None,
    })
}

fn gen_idx(s: &str) -> Option<usize> {
    ["l1", "l2", "r1", "r2"].iter().position(|n| *n == s)
}

fn src_text(bus: usize, f: Feed) -> String {
    match f {
        Feed::None => "none".to_string(),
        Feed::Src(Src::Vfsg(k)) => {
            let k = k as usize % 4;
            if k == bus {
                format!("{}_VFSG", BUS_NAMES[k])
            } else if k / 2 == bus / 2 {
                "ring_tie".to_string()
            } else {
                format!("ring_tie_from_{}_vfsg", ["left", "right"][k / 2])
            }
        }
        Feed::Src(Src::Asg(k)) => format!("{}_ASG", ["L", "R"][k as usize % 2]),
        Feed::Src(Src::Ext(k)) => format!("EXT_{}", ["FWD_L", "FWD_R"][k as usize % 2]),
    }
}

impl Electrical {
    pub fn value(&self, name: &str) -> Option<SignalValue> {
        self.value_core(name)
            .or_else(|| self.bound_misc(name))
            .or_else(|| self.bound_loads(name))
            .or_else(|| self.bound_params(name))
            .or_else(|| self.thermal_value(name))
            .or_else(|| self.fault_value(name))
    }

    fn value_core(&self, name: &str) -> Option<SignalValue> {
        use SignalValue::{Bool, Num, Text};
        let parts: Vec<&str> = name.split('.').collect();
        if parts.first() != Some(&"elec") {
            return None;
        }
        match parts.as_slice() {
            ["elec", "mode"] => Some(Text(self.mode().to_string())),
            ["elec", "bus", alias, what] if matches!(*alias, "c1_tru" | "c2_tru") => {
                let v = self.tru_input_v(*alias == "c1_tru");
                match *what {
                    "energized" => Some(Bool(v > BUS_LIVE_V)),
                    "voltage" => Some(Num(v)),
                    _ => None,
                }
            }
            ["elec", "bus", alias, what] => {
                let bus = bus_alias(alias)?;
                let v = self.plant.bus_v(bus);
                match *what {
                    "energized" => Some(Bool(v > 18.0)),
                    "voltage" => Some(Num(v)),
                    "freq_hz" => Some(Num(self.plant.bus_hz(bus))),
                    "source" => {
                        let k = BUS_NAMES.iter().position(|b| *b == bus)?;
                        Some(Text(src_text(k, self.feed_of(k))))
                    }
                    _ => None,
                }
            }
            ["elec", "hbb_main", "voltage"] => Some(Num(self.plant.bus_v("HBB_MAIN"))),
            ["elec", "other_ac_buses", "energized_count"] => {
                let n = BUS_NAMES
                    .iter()
                    .filter(|b| self.plant.bus_v(b) > BUS_LIVE_V)
                    .count();
                Some(Num(n as f64))
            }
            ["elec", "epc", which, what] => {
                let (k, name) = match *which {
                    "fwd_l" => (0, "L EPC"),
                    "fwd_r" => (1, "R EPC"),
                    "aft" => (2, "LAEPC"),
                    _ => return None,
                };
                match *what {
                    "closed" => Some(Bool(self.contactor_closed(name)?)),
                    "locked_out" => Some(Bool(self.ext_out[k].tripped)),
                    "avail" => Some(Bool(self.bpcu_out.ext_avail[k])),
                    "chatter_lockout" => Some(Bool(self.epc_sup[k].chatter_lockout)),
                    "fail_to_open" => Some(Bool(self.epc_sup[k].failed_open_check)),
                    "fail_to_close" => Some(Bool(self.epc_sup[k].aft_fail_to_close)),
                    "on" => Some(Bool(self.bpcu_out.ext_on[k])),
                    _ => None,
                }
            }
            ["elec", "gcb", which, "closed"] => {
                let k = gen_idx(which)?;
                Some(Bool(self.contactor_closed(&format!("{} GCB", BUS_NAMES[k]))?))
            }
            ["elec", "mbr", "closed"] => Some(Bool(self.contactor_closed("MBR")?)),
            ["elec", "cit", "closed"] => Some(Bool(self.contactor_closed("CIT")?)),
            ["elec", "rat", "deployed"] => Some(Bool(self.rat.deployed())),
            ["elec", "rat", "deploy_requested"] => Some(Bool(self.bpcu_out.rat_deploy)),
            ["elec", "asg", which, "gen_on_bus"] => {
                let k = if *which == "l" { 0 } else { 1 };
                Some(Bool(self.contactor_closed(["L APB", "R APB"][k])?))
            }
            ["elec", "vfsg", which, what] => {
                let k = gen_idx(which)?;
                match *what {
                    "mode" => Some(Text(self.vfsg_mode(k).to_string())),
                    "freq_hz" => Some(Num(self.plant.machine_hz(k))),
                    "load_kva" => {
                        let (v, i) = self.plant.machine_sense(k);
                        Some(Num(3.0 * v * i / 1000.0))
                    }
                    _ => None,
                }
            }
            ["elec", "gcu", which, "fault_failsafe"] => {
                let k = gen_idx(which)?;
                Some(Bool(self.gcu_out[k].trip_latched))
            }
            ["elec", "bpcu", "fail_to_close", name] => {
                let n = name.replace('_', " ").to_uppercase();
                if let Some(e) = crate::control::bpcu::BTB_NAMES.iter().position(|b| *b == n) {
                    return Some(Bool(self.bpcu_out.btb_fail_to_close[e]));
                }
                let k = ["L BSB", "R BSB"].iter().position(|b| *b == n)?;
                Some(Bool(self.bpcu_out.bsb_fail_to_close[k]))
            }
            ["elec", "battery", "main", "monitor", what] => {
                let o = &self.batt_mon_out;
                match *what {
                    "discharge_inhibit" => Some(Bool(o.discharge_inhibit)),
                    "fault_sensed" => Some(Bool(o.fault_sensed)),
                    "hbb_uv_fault" => Some(Bool(o.hbb_uv_fault)),
                    "hbb_ov_fault" => Some(Bool(o.hbb_ov_fault)),
                    "fully_charged" => Some(Bool(o.fully_charged)),
                    "tow_light" => Some(Text(format!("{:?}", o.tow_light).to_uppercase())),
                    "tow_minutes" => Some(Num(o.tow_light.minimum_minutes())),
                    _ => None,
                }
            }
            ["elec", "atru", which, what] if which.len() == 2 => {
                let k = ["l1", "l2", "r1", "r2"].iter().position(|n| n == which)?;
                let p = &self.atru_prot[k];
                let (w, v) = self.atru_reading(crate::protection::atru_prot::ATRU_NAMES[k]);
                match *what {
                    "overload" => Some(Bool(p.overload())),
                    "overvoltage" => Some(Bool(p.overvoltage())),
                    "freq_out" => Some(Bool(p.freq_out())),
                    "load_kw" => Some(Num(w / 1000.0)),
                    "out_of_tolerance" => Some(Bool(
                        v > 1.0 && crate::protection::atru_prot::out_of_tolerance(v / 2.0),
                    )),
                    _ => None,
                }
            }
            ["elec", "tie", which, what] => {
                let k = ["l", "r"].iter().position(|n| n == which)?;
                match *what {
                    "diff_tripped" => Some(Bool(self.tie_diff[k].tripped())),
                    "leak_a" => Some(Num(self.zone_leak_a(["LTB", "RTB"][k]))),
                    _ => None,
                }
            }
            ["elec", "atu", which, what] => {
                let k = ["l", "r"].iter().position(|n| n == which)?;
                match *what {
                    "diff_tripped" => Some(Bool(self.atu_diff[k].tripped())),
                    "leak_a" => Some(Num(self.zone_leak_a(["ATU_L", "ATU_R"][k]))),
                    _ => None,
                }
            }
            ["elec", "tru", which, what] => {
                let k = ["l", "r", "c1", "c2"].iter().position(|n| n == which)?;
                match *what {
                    "oc_tripped" => Some(Bool(self.tru_oc[k].tripped())),
                    "output_a" => Some(Num(self.tru_output_a(k))),
                    _ => None,
                }
            }
            ["elec", "load_shed", "active"] => Some(Bool(self.lm_out.shed_count > 0)),
            ["elec", "pecs", which, "coolant_c"] => {
                let k = ["fwd", "aft"].iter().position(|n| n == which)?;
                Some(Num(self.pecs_c[k]))
            }
            ["elec", "load_shed", "pecs_thermal"] => Some(Num(self.lm_out.pecs_thermal_shed as f64)),
            ["elec", "load_shed", "emp_cmsc_stopped"] => Some(Bool(self.lm_out.emp_cmsc_stopped)),
            ["elec", "load_shed", "hpx_groups"] => Some(Num(self.lm_out.hpx_groups_shed as f64)),
            ["elec", "hpx", which, what] => {
                let e = ["l", "r"].iter().position(|n| n == which)?;
                match *what {
                    "kw" => Some(Num(self.hpx_kw(e))),
                    "limit_kw" => Some(Num(super::hpx_limit_kw(
                        self.inputs.engine_rpm[e],
                        self.inputs.engine_running[e],
                    ))),
                    _ => None,
                }
            }
            ["elec", "load_shed", "count"] => Some(Num(self.lm_out.shed_count as f64)),
            ["elec", "ccs", "powered"] => {
                Some(Bool(self.bpcu_out.ccs_powered && self.plant.battery_v(0) > 18.0))
            }
            ["elec", "cbic", "mode"] => Some(Text(format!("{:?}", self.cbic.app.mode()).to_lowercase())),
            ["elec", "msg", id] => {
                let found = self.eicas_messages().iter().any(|n| n.replace(' ', "_") == *id);
                Some(Bool(found))
            }
            ["elec", "battery", which, what] => {
                let b = if *which == "main" { 0 } else { 1 };
                match *what {
                    "voltage" => Some(Num(self.plant.battery_v(b))),
                    "current" => Some(Num(self.plant.battery_a(b))),
                    "temp_c" => Some(Num(self.plant.battery_temp_c(b))),
                    "bmu_disconnect" => Some(Bool(self.bmu_out[b].disconnect)),
                    "diode_forward_v" => Some(Num(self
                        .diode
                        .forward_v(self.plant.battery_v(b), self.plant.battery_a(b).abs()))),
                    "charge_a" => Some(Num(self.charger_out[b].charge_current_a)),
                    _ => None,
                }
            }
            ["elec", "ext", which, what] => {
                let k = match *which {
                    "fwd_l" => 0,
                    "fwd_r" => 1,
                    "aft" => 2,
                    _ => return None,
                };
                match *what {
                    "avail" => Some(Bool(self.ext_out[k].avail)),
                    "quality_good" => Some(Bool(self.ext_out[k].power_quality_good)),
                    "tripped" => Some(Bool(self.ext_out[k].tripped)),
                    "switch_on" => Some(Bool(self.ext_switch_on[k])),
                    _ => None,
                }
            }
            ["elec", "contactor", cname] => Some(Bool(self.contactor_closed(&cname.replace('_', " "))?)),
            _ => None,
        }
    }

    fn vfsg_mode(&self, k: usize) -> &'static str {
        if self.gcu_out[k].trip_latched {
            "tripped"
        } else if self.crew_in.engine_start_in_progress[k / 2] {
            "start"
        } else if self.rig.vfsg[k].mode == super::rig::VfsgMode::Motor {
            "motor"
        } else if self
            .contactor_closed(&format!("{} GCB", BUS_NAMES[k]))
            .unwrap_or(false)
        {
            "generator"
        } else if self.gcu_out[k].voltage_established {
            "ready"
        } else {
            "off"
        }
    }

    pub fn signal(&self, name: &str) -> Option<f64> {
        match self.value(name)? {
            SignalValue::Num(n) => Some(n),
            SignalValue::Bool(b) => Some(if b { 1.0 } else { 0.0 }),
            SignalValue::Text(_) => None,
        }
    }

    pub fn flag(&self, name: &str) -> Option<bool> {
        match self.value(name)? {
            SignalValue::Bool(b) => Some(b),
            SignalValue::Num(n) => Some(n > 0.5),
            SignalValue::Text(_) => None,
        }
    }

    pub fn text(&self, name: &str) -> Option<String> {
        match self.value(name)? {
            SignalValue::Text(t) => Some(t),
            SignalValue::Bool(b) => Some(b.to_string()),
            SignalValue::Num(n) => Some(n.to_string()),
        }
    }

    pub fn snapshot(&self) -> Vec<(String, SignalValue)> {
        let mut out = Vec::new();
        let mut push = |k: String| {
            if let Some(v) = self.value(&k) {
                out.push((k, v));
            }
        };
        push("elec.mode".to_string());
        for b in [
            "L1_235", "L2_235", "R1_235", "R2_235", "L_115", "R_115", "backup", "hbb_main",
        ] {
            for w in ["energized", "voltage"] {
                push(format!("elec.bus.{b}.{w}"));
            }
        }
        for g in ["l1", "l2", "r1", "r2"] {
            push(format!("elec.gcb.{g}.closed"));
            push(format!("elec.vfsg.{g}.mode"));
            push(format!("elec.vfsg.{g}.freq_hz"));
        }
        for e in ["fwd_l", "fwd_r", "aft"] {
            push(format!("elec.epc.{e}.closed"));
            push(format!("elec.ext.{e}.avail"));
        }
        for k in [
            "elec.mbr.closed",
            "elec.rat.deployed",
            "elec.rat.deploy_requested",
            "elec.load_shed.active",
            "elec.battery.main.voltage",
            "elec.battery.main.current",
            "elec.battery.apu.voltage",
        ] {
            push(k.to_string());
        }
        out
    }
}
