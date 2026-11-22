use crate::target::{Target, unbound};
use crate::value::Value;
use std::path::Path;
use sys_ccs::{Ccs, CcsInputs, CcsValue};
use sys_electrical::system::{Electrical, SignalValue};

pub const CCS_BUSES: [&str; 5] = ["CIB", "FIB", "DC28_L", "DC28_R", "HBB_MAIN"];
pub const RDC_BUSES: [&str; 4] = ["CIB", "FIB", "DC28_L", "DC28_R"];
pub const APPROX_CCS_SEED: u64 = 1;
pub const COUPLING_STEP_US: u64 = 100_000;

pub struct AircraftTarget {
    elec: Electrical,
    ccs: Ccs,
    applied: Vec<(&'static str, f64)>,
}

impl AircraftTarget {
    pub fn new(data_dir: &Path) -> Result<AircraftTarget, String> {
        let ccs_dir = data_dir
            .parent()
            .map(|p| p.join("42-ccs"))
            .ok_or("no aircraft data root")?;
        let mut elec = Electrical::new(data_dir)?;
        elec.set_external_load(sys_electrical::net::CCS_STANDIN_LOAD, 0.0);
        let mut ccs = Ccs::new(&ccs_dir, APPROX_CCS_SEED)?;
        let names: Vec<String> = sys_electrical::crew::eicas::message_catalogue()
            .into_iter()
            .map(|d| d.name)
            .collect();
        ccs.set_elec_icd(&names);
        Ok(AircraftTarget {
            elec,
            ccs,
            applied: Vec::new(),
        })
    }

    fn ccs_inputs(&self) -> CcsInputs {
        let mut i = CcsInputs::default();
        for b in CCS_BUSES.iter().chain(RDC_BUSES.iter()) {
            i.bus_v.insert(b.to_string(), self.elec.bus_volts(b));
        }
        i.on_ground = self.elec.inputs().on_ground;
        i.fuel_run = [self.elec.fuel_run(0), self.elec.fuel_run(1)];
        i.ambient_c = self.elec.inputs().ambient_c;
        i
    }

    pub fn ccs(&self) -> &Ccs {
        &self.ccs
    }

    pub fn elec(&self) -> &Electrical {
        &self.elec
    }

    fn couple(&mut self, dt_us: u64) {
        let active = self.elec.eicas_messages();
        let payload = self.ccs.encode_elec(&active);
        self.ccs.bpcu_payload = [payload.clone(), payload];
        let inputs = self.ccs_inputs();
        self.ccs.step(dt_us, &inputs);
        let loads = self.ccs.loads_w();
        if loads != self.applied {
            for (name, w) in &loads {
                self.elec.set_external_load(name, *w);
            }
            self.applied = loads;
        }
    }
}

impl Target for AircraftTarget {
    fn name(&self) -> &str {
        "aircraft"
    }

    fn apply(
        &mut self,
        actor: &str,
        action: &str,
        target: &str,
        value: Option<&Value>,
    ) -> Result<(), String> {
        if actor == "crew"
            && let Some(side) = match target {
                "CCR_RESET_L" => Some(0),
                "CCR_RESET_R" => Some(1),
                _ => None,
            }
        {
            self.ccs.press_reset(side);
            return Ok(());
        }
        if actor == "sim" && target == "ccs_powered_days" {
            let d = value
                .map(|v| v.to_string())
                .and_then(|t| t.parse::<f64>().ok())
                .ok_or("days")?;
            self.ccs.set_powered_days(d);
            return Ok(());
        }
        if actor == "maint"
            && let Some(g) = target.strip_prefix("DATA_LOAD_")
        {
            let on = value
                .map(|v| v.to_string())
                .is_none_or(|t| t != "false" && t != "OFF");
            let sw = self.ccs.sw.iter_mut().find(|s| s.name == g).ok_or("gpm")?;
            sw.data_load = on;
            return Ok(());
        }
        if actor == "fault" {
            let on = action == "arm";
            let mode = value.map(|v| v.to_string()).unwrap_or_else(|| "failed".into());
            if sys_ccs::faults::part_type_of(&self.ccs, target).is_some() {
                return sys_ccs::faults::arm(&mut self.ccs, target, &mode, on).map_err(unbound);
            }
            if let Some(rest) = target.strip_prefix("PART ") {
                let (g, part) = rest.split_once(' ').ok_or("PART <gpm> <partition>")?;
                if on {
                    self.ccs.part_faults.insert((g.to_string(), part.to_string()));
                } else {
                    self.ccs.part_faults.remove(&(g.to_string(), part.to_string()));
                }
                return Ok(());
            }
            if let Some(rest) = target.strip_prefix("LANE ") {
                let (g, lane) = rest.split_once(' ').ok_or("LANE <gpm> <A|B>")?;
                let k = if lane == "A" { 0 } else { 1 };
                let sw = self.ccs.sw.iter_mut().find(|s| s.name == g).ok_or("gpm")?;
                sw.lane_failed[k] = on;
                return Ok(());
            }
            if let Some(rest) = target.strip_prefix("LINK ") {
                let (a, b) = rest.split_once('|').ok_or("LINK <a>|<b>")?;
                let (ber, open) = if mode == "open" {
                    (0.0, on)
                } else {
                    (if on { 1.0e-5 } else { 0.0 }, false)
                };
                return if self.ccs.set_link_fault(a, b, ber, open) {
                    Ok(())
                } else {
                    Err(unbound(target))
                };
            }

            if let Some(g) = target.strip_prefix("GPM_") {
                return if self.ccs.set_gpm_failed(g, on) {
                    Ok(())
                } else {
                    Err(unbound(target))
                };
            }
            if target.starts_with("RDC ") {
                return if self.ccs.set_rdc_failed(target, on) {
                    Ok(())
                } else {
                    Err(unbound(target))
                };
            }
            if self.ccs.set_device_fault(target, &mode, on) {
                return Ok(());
            }
        }
        let text = value.map(|v| v.to_string());
        let action = if actor == "sim" && action == "initial" {
            "set"
        } else {
            action
        };
        self.elec
            .action(actor, action, target, text.as_deref())
            .map_err(unbound)
    }

    fn read(&self, signal: &str) -> Option<Value> {
        if signal.starts_with("ccs.") {
            return self.ccs.value(signal).map(|v| match v {
                CcsValue::Bool(b) => Value::Bool(b),
                CcsValue::Num(x) => Value::Num(x),
                CcsValue::Text(s) => Value::Str(s),
            });
        }
        let name = signal.strip_prefix("elec.").unwrap_or(signal);
        for n in [signal, name] {
            if let Some(v) = self.elec.value(n) {
                return Some(match v {
                    SignalValue::Bool(b) => Value::Bool(b),
                    SignalValue::Num(x) => Value::Num(x),
                    SignalValue::Text(s) => Value::Str(s),
                });
            }
        }
        None
    }

    fn step(&mut self, dt_us: u64) {
        let mut left = dt_us;
        while left > 0 {
            let h = left.min(COUPLING_STEP_US);
            self.elec.step_idle(h);
            self.couple(h);
            left -= h;
        }
    }

    fn adaptive(&self) -> bool {
        true
    }

    fn set_fine(&mut self, fine: bool) {
        self.elec.set_fine_stepping(fine);
    }
}
