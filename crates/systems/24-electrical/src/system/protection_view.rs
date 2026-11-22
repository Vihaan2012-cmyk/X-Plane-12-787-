use super::{Electrical, MACHINES};
use crate::control::bpcu::BTB_NAMES;
use crate::inspector::ProtectionRow;
use crate::protection::atru_prot::ATRU_NAMES;
use crate::protection::tru_oc::TRU_NAMES;
use crate::sources::gcu::GcuFault;

const GCU_FAULTS: [(GcuFault, &str); 14] = [
    (GcuFault::OverVoltage, "over voltage"),
    (GcuFault::UnderVoltage, "under voltage"),
    (GcuFault::OverFreq, "over frequency"),
    (GcuFault::UnderFreq, "under frequency"),
    (GcuFault::Differential, "differential"),
    (GcuFault::OverCurrent, "over current"),
    (GcuFault::OpenPhase, "open phase"),
    (GcuFault::PhaseSequence, "phase sequence"),
    (GcuFault::PoweredTimeOverflow, "powered time overflow"),
    (GcuFault::GroundFault, "feeder ground fault"),
    (GcuFault::DcContent, "DC content"),
    (GcuFault::ShortedRotatingDiode, "shorted rotating diode"),
    (GcuFault::UnbalancedFeeder, "unbalanced parallel feeder"),
    (GcuFault::FieldDriver, "field driver"),
];

fn row(group: &str, name: &str, tripped: bool, state: String, detail: String) -> ProtectionRow {
    ProtectionRow {
        group: group.to_string(),
        name: name.to_string(),
        state,
        tripped,
        detail,
    }
}

fn flag(tripped: bool) -> String {
    if tripped { "TRIPPED" } else { "normal" }.to_string()
}

impl Electrical {
    pub fn protection_rows(&self) -> Vec<ProtectionRow> {
        let mut v = Vec::new();
        for (m, name) in MACHINES.iter().enumerate() {
            let mask = self.gcus[m].latched_mask();
            let names: Vec<&str> = GCU_FAULTS
                .iter()
                .filter(|(f, _)| mask & f.mask() != 0)
                .map(|(_, n)| *n)
                .collect();
            let state = if names.is_empty() {
                if self.gcu_out[m].vr_enabled {
                    "regulating"
                } else {
                    "field off"
                }
                .to_string()
            } else {
                names.join(", ")
            };
            let detail = format!(
                "generator mode {}  exciter {:.2} A  established {}",
                self.gcus[m].generator_mode(),
                self.gcu_out[m].exciter_current_a,
                self.gcu_out[m].voltage_established
            );
            v.push(row("GCU", &format!("GCU {name}"), mask != 0, state, detail));
        }
        let o = &self.bpcu_out;
        for (k, n) in BTB_NAMES.iter().enumerate() {
            let t = o.btb_fail_to_close[k];
            v.push(row(
                "BPCU fail to close",
                n,
                t,
                if t { "LOCKED OUT".into() } else { "normal".into() },
                "aux disagree 150 ms".into(),
            ));
        }
        for (k, n) in ["L BSB", "R BSB"].iter().enumerate() {
            let t = o.bsb_fail_to_close[k];
            v.push(row(
                "BPCU fail to close",
                n,
                t,
                if t { "LOCKED OUT".into() } else { "normal".into() },
                "aux disagree 190 ms".into(),
            ));
        }
        for (k, n) in ["LTB", "RTB"].iter().enumerate() {
            let t = self.tie_diff[k].tripped();
            v.push(row(
                "Zone differential",
                &format!("tie {n}"),
                t,
                flag(t),
                format!("leak {:.1} A", self.zone_leak_a(n)),
            ));
        }
        for (k, n) in ["ATU_L", "ATU_R"].iter().enumerate() {
            let t = self.atu_diff[k].tripped();
            v.push(row(
                "Zone differential",
                n,
                t,
                flag(t),
                format!("leak {:.1} A", self.zone_leak_a(n)),
            ));
        }
        for (k, n) in TRU_NAMES.iter().enumerate() {
            let t = self.tru_oc[k].tripped();
            v.push(row(
                "TRU overcurrent",
                n,
                t,
                flag(t),
                format!("{:.0} A out", self.tru_output_a(k)),
            ));
        }
        for (k, n) in ATRU_NAMES.iter().enumerate() {
            let p = &self.atru_prot[k];
            let mut why = Vec::new();
            if p.overload() {
                why.push("overload");
            }
            if p.overvoltage() {
                why.push("output over voltage");
            }
            if p.freq_out() {
                why.push("input frequency");
            }
            let (w, volts) = self.atru_reading(n);
            let state = if why.is_empty() {
                "normal".to_string()
            } else {
                why.join(", ")
            };
            v.push(row(
                "ATRU",
                n,
                p.shut_down(),
                state,
                format!("{:.0} kW  {:.0} V", w / 1000.0, volts),
            ));
        }
        for (e, n) in ["L engine", "R engine"].iter().enumerate() {
            let kw = self.hpx_kw(e);
            let lim = super::hpx_limit_kw(self.inputs.engine_rpm[e], self.inputs.engine_running[e]);
            let over = lim > 0.0 && kw > lim;
            v.push(row(
                "Load management",
                &format!("HPx {n}"),
                over,
                format!("{kw:.0} of {lim:.0} kW"),
                String::new(),
            ));
        }
        let g = self.lm_out.hpx_groups_shed;
        v.push(row(
            "Load management",
            "HPx groups shed",
            g > 0,
            g.to_string(),
            "repeat within 20 min adds a group".into(),
        ));
        for (k, n) in ["L EPC", "R EPC", "LAEPC"].iter().enumerate() {
            let s = &self.epc_sup[k];
            let mut why = Vec::new();
            if s.chatter_lockout {
                why.push("chatter lockout");
            }
            if s.failed_open_check {
                why.push("current after open");
            }
            if s.aft_fail_to_close {
                why.push("no current after close");
            }
            let bad = !why.is_empty();
            let state = if bad { why.join(", ") } else { "normal".to_string() };
            v.push(row("External power", n, bad, state, String::new()));
        }
        let b = &self.batt_mon_out;
        for (n, t) in [
            ("discharge inhibit", b.discharge_inhibit),
            ("fault sensed", b.fault_sensed),
            ("HBB under voltage", b.hbb_uv_fault),
            ("HBB over voltage", b.hbb_ov_fault),
        ] {
            v.push(row(
                "Main battery",
                n,
                t,
                if t { "SET".into() } else { "clear".into() },
                String::new(),
            ));
        }
        v.push(row(
            "Main battery",
            "towing charge light",
            false,
            format!("{:?}", b.tow_light).to_uppercase(),
            format!("{} min minimum", b.tow_light.minimum_minutes()),
        ));
        v
    }
}
