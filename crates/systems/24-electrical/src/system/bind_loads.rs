use super::{BUS_LIVE_V, Electrical, SignalValue};
use crate::control::CbState;
use crate::control::DEFAULT_LOADS;
use crate::control::load_mgmt::{EP_5MIN_KVA, EP_5S_KVA, MIN_SHED_US};
use crate::sources;
use std::collections::BTreeMap;
use std::sync::OnceLock;

const BINDINGS_CSV: &str = include_str!("../../../../../aircraft/24-electrical/load-bindings.csv");
const BREAKERS_CSV: &str = include_str!("../../../../../aircraft/24-electrical/breakers.csv");
const DC_LIVE_V: f64 = 18.0;
const AC115_LIVE_V: f64 = 40.0;

struct Row {
    kind: String,
    target: String,
    grade: String,
    source: String,
}

fn table() -> &'static BTreeMap<String, Row> {
    static TABLE: OnceLock<BTreeMap<String, Row>> = OnceLock::new();
    TABLE.get_or_init(|| {
        let mut map = BTreeMap::new();
        for line in BINDINGS_CSV.lines().skip(1) {
            let f: Vec<&str> = line.split(',').collect();
            if f.len() < 5 || f[0].is_empty() {
                continue;
            }
            map.insert(
                f[0].to_string(),
                Row {
                    kind: f[1].to_string(),
                    target: f[2].to_string(),
                    grade: f[3].to_string(),
                    source: f[4].to_string(),
                },
            );
        }
        map
    })
}

fn id_prefix(prefix_type: &str) -> Option<String> {
    for line in BREAKERS_CSV.lines().skip(1) {
        let f: Vec<&str> = line.split(',').collect();
        if f.len() > 2 && f[2] == prefix_type {
            let p: String = f[0].chars().take_while(|c| c.is_ascii_alphabetic()).collect();
            if !p.is_empty() {
                return Some(p);
            }
        }
    }
    None
}

fn live_threshold(bus: &str) -> f64 {
    if bus.starts_with("DC28")
        || bus.starts_with("HBB")
        || bus == "CIB"
        || bus == "FIB"
        || bus.ends_with(" DC")
    {
        DC_LIVE_V
    } else if bus.starts_with("AC115") || bus.ends_with(" AC") {
        AC115_LIVE_V
    } else {
        BUS_LIVE_V
    }
}

fn protection_value(name: &str) -> Option<f64> {
    Some(match name {
        "EP_OF_HZ" => sources::EP_OF_HZ,
        "EP_OF_RESET_HZ" => sources::EP_OF_RESET_HZ,
        "EP_UF_HZ" => sources::EP_UF_HZ,
        "EP_UF_RESET_HZ" => sources::EP_UF_RESET_HZ,
        "EP_UV_V" => sources::EP_UV_V,
        "EP_UV_RESET_V" => sources::EP_UV_RESET_V,
        "EP_UV_DELAY_S" => sources::EP_UV_DELAY_S,
        "EP_OV_V" => sources::EP_OV_V,
        "EP_OV_RESET_V" => sources::EP_OV_RESET_V,
        _ => return None,
    })
}

fn load_index(name: &str) -> Option<usize> {
    DEFAULT_LOADS.iter().position(|d| d.name == name)
}

impl Electrical {
    pub fn binding_grade(&self, name: &str) -> Option<(&'static str, &str)> {
        let row = table().get(name)?;
        let grade = ["VERIFIED", "DERIVED", "APPROX"]
            .into_iter()
            .find(|g| *g == row.grade)?;
        Some((grade, row.source.as_str()))
    }

    fn bind_atom(&self, atom: &str) -> bool {
        if let Some(n) = atom.strip_prefix("shed:") {
            return load_index(n).is_some_and(|k| !self.lm_out.shed[k]);
        }
        if let Some(n) = atom.strip_prefix("sw:") {
            return match n {
                "cabin_util" => self.panel_out.cabin_util_on,
                "ife" => self.panel_out.ife_on,
                "battery" => self.panel_out.battery_on,
                "ground_service" => self.ground_service_switch,
                _ => false,
            };
        }
        if let Some(n) = atom.strip_prefix("mode:") {
            return self.mode() == n;
        }
        if let Some(n) = atom.strip_prefix("start:") {
            return n
                .parse::<usize>()
                .ok()
                .and_then(|k| self.crew_in.engine_start_in_progress.get(k).copied())
                .unwrap_or(false);
        }
        if atom == "elcc_enabled" {
            return self.bpcu_out.sspc_elcc_loads_enabled;
        }
        if atom == "ep_l_only" || atom == "ep_r_only" {
            let (own, other) = if atom == "ep_l_only" {
                ("L EPC", "R EPC")
            } else {
                ("R EPC", "L EPC")
            };
            return self.mode() == "external_power_on"
                && self.contactor_closed(own) == Some(true)
                && self.contactor_closed(other) != Some(true);
        }
        if atom == "apu_start" {
            return self.apu_start_latch_us.is_some();
        }
        match self.contactor_closed(atom) {
            Some(closed) => closed,
            None => self.plant.bus_v(atom) > live_threshold(atom),
        }
    }

    fn bind_expr(&self, expr: &str) -> bool {
        expr.split('&').all(|term| {
            let (neg, body) = match term.strip_prefix('!') {
                Some(b) => (true, b),
                None => (false, term),
            };
            body.split('|').any(|a| self.bind_atom(a)) != neg
        })
    }

    fn bind_load_shed(&self, target: &str) -> Option<SignalValue> {
        use SignalValue::{Bool, Num};
        let out = &self.lm_out;
        if let Some(c) = target.strip_prefix("const:") {
            return c.parse::<f64>().ok().map(Num);
        }
        if let Some(pair) = target.strip_prefix("order:") {
            let (a, b) = pair.split_once('<')?;
            let (ka, kb) = (load_index(a)?, load_index(b)?);
            return Some(Bool(!(out.shed[kb] && !out.shed[ka])));
        }
        Some(match target {
            "active" => Bool(out.shed_count > 0),
            "count" => Num(out.shed_count as f64),
            "interval_s" => Num(MIN_SHED_US as f64 / 1_000_000.0),
            "thr_5min_kva" => Num(EP_5MIN_KVA),
            "thr_5s_kva" => Num(EP_5S_KVA),
            "group" => Bool(out.ext_overload_5s),
            "nonessential" => Bool(
                out.shed_count > 0
                    && out
                        .shed
                        .iter()
                        .zip(DEFAULT_LOADS.iter())
                        .all(|(s, d)| !*s || d.sheddable),
            ),
            "tcb_shed" => Bool(
                out.shed
                    .iter()
                    .zip(DEFAULT_LOADS.iter())
                    .any(|(s, d)| *s && !d.sheddable),
            ),
            _ => return None,
        })
    }

    fn bind_cmsc(&self, chain: &str) -> Option<SignalValue> {
        for seg in chain.split(';') {
            match seg.split_once('=') {
                Some((cond, text)) => {
                    if self.bind_expr(cond) {
                        return Some(SignalValue::Text(text.to_string()));
                    }
                }
                None => return Some(SignalValue::Text(seg.to_string())),
            }
        }
        None
    }

    fn bind_breaker(&self, target: &str) -> Option<SignalValue> {
        use SignalValue::{Bool, Num, Text};
        let (what, rest) = target.split_once(':')?;
        if what == "prefix" {
            return id_prefix(rest).map(Text);
        }
        let (id_text, fallback) = match rest.split_once('=') {
            Some((i, f)) => (i, Some(f)),
            None => (rest, None),
        };
        let id: u32 = id_text.parse().ok()?;
        let reg = self.cbic.app.breaker(id);
        let dev = self.pdu.breaker_truth(id);
        let state = match dev {
            Some((s, _)) => Some(s),
            None => reg.map(|b| b.state),
        };
        let closed = match state {
            Some(s) => s == CbState::Closed,
            None => fallback.is_some_and(|f| self.bind_expr(f)),
        };
        match what {
            "state" => Some(Text(
                match state {
                    Some(CbState::Tripped) => "tripped",
                    Some(CbState::Unknown) => "unknown",
                    _ if closed => "closed",
                    _ => "open",
                }
                .to_string(),
            )),
            "amps" => Some(Num(match (dev, reg) {
                (Some((_, a)), _) if closed => a,
                (None, Some(b)) if closed => b.load_amps,
                _ => 0.0,
            })),
            "locked" => Some(Bool(matches!(state, Some(CbState::Locked(_))))),
            _ => None,
        }
    }

    pub(crate) fn bound_loads(&self, name: &str) -> Option<SignalValue> {
        use SignalValue::{Bool, Num};
        let row = table().get(name)?;
        match row.kind.as_str() {
            "bus" | "sspc" | "contactor" => Some(Bool(self.bind_expr(&row.target))),
            "load_shed" => self.bind_load_shed(&row.target),
            "pecs" => {
                if row.target.contains(';') {
                    let n = row.target.split(';').filter(|g| self.bind_expr(g)).count();
                    Some(Num(n as f64))
                } else {
                    Some(Bool(self.bind_expr(&row.target)))
                }
            }
            "cmsc" => self.bind_cmsc(&row.target),
            "breaker" => self.bind_breaker(&row.target),
            "protection" => protection_value(&row.target).map(Num),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::system::ElectricalInputs;

    fn new_sys() -> Electrical {
        let dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../../aircraft/24-electrical");
        Electrical::new(&dir).expect("system builds")
    }

    fn run(e: &mut Electrical, seconds: f64, inputs: &ElectricalInputs) {
        for _ in 0..(seconds * 100.0) as u64 {
            e.step(10_000, inputs);
        }
    }

    fn flying() -> ElectricalInputs {
        ElectricalInputs {
            engine_rpm: [7600.0; 2],
            engine_running: [true; 2],
            on_ground: false,
            radio_alt_ft: 20000.0,
            ..ElectricalInputs::default()
        }
    }

    #[test]
    fn table_rows_are_well_formed() {
        let t = table();
        assert!(t.len() > 70);
        for (name, row) in t {
            assert!(
                [
                    "bus",
                    "sspc",
                    "contactor",
                    "load_shed",
                    "breaker",
                    "protection",
                    "cmsc",
                    "pecs"
                ]
                .contains(&row.kind.as_str()),
                "{name}"
            );
            assert!(!row.target.is_empty(), "{name}");
            assert!(!row.source.is_empty(), "{name}");
            assert!(
                ["VERIFIED", "DERIVED", "APPROX"].contains(&row.grade.as_str()),
                "{name}"
            );
        }
    }

    #[test]
    fn every_listed_signal_resolves() {
        let mut e = new_sys();
        run(&mut e, 0.5, &ElectricalInputs::default());
        for name in table().keys() {
            assert!(e.value(name).is_some(), "{name}");
            assert!(e.binding_grade(name).is_some(), "{name}");
        }
    }

    #[test]
    fn cold_and_dark_loads_are_off() {
        let mut e = new_sys();
        run(&mut e, 1.0, &ElectricalInputs::default());
        assert_eq!(e.flag("elec.load.captain_inboard_du"), Some(false));
        assert_eq!(e.flag("elec.load.c1_hydraulic_pump"), Some(false));
        assert_eq!(e.flag("elec.pecs.energized"), Some(false));
    }

    #[test]
    fn battery_on_powers_instrument_bus_loads_only() {
        let mut e = new_sys();
        run(&mut e, 0.5, &ElectricalInputs::default());
        assert_eq!(e.flag("elec.load.lower_du"), Some(false));
        e.action("crew", "press", "BATTERY", Some("ON")).unwrap();
        run(&mut e, 3.0, &ElectricalInputs::default());
        assert_eq!(e.flag("elec.load.lower_du"), Some(true));
        assert_eq!(e.flag("elec.load.captain_flight_instruments"), Some(true));
        assert_eq!(e.flag("elec.load.c1_hydraulic_pump"), Some(false));
        assert_eq!(e.flag("elec.load.fo_displays"), Some(false));
        assert_eq!(e.flag("elec.load.only_tcb_instrument_bus_loads"), Some(true));
    }

    #[test]
    fn generator_online_powers_ac_loads_and_pecs() {
        let mut e = new_sys();
        for i in 0..4 {
            e.panel_mut().gen_ctrl[i].set_latched(true);
        }
        e.action("crew", "press", "BATTERY", Some("ON")).unwrap();
        run(&mut e, 20.0, &flying());
        assert_eq!(e.flag("elec.bus.L1_235.energized"), Some(true));
        assert_eq!(e.flag("elec.load.c1_hydraulic_pump"), Some(true));
        assert_eq!(e.flag("elec.pecs.energized"), Some(true));
        assert!(e.signal("elec.pecs.loops_active").unwrap() >= 1.0);
        assert_eq!(e.text("elec.cmsc.l1.mode").as_deref(), Some("CAC"));
    }

    #[test]
    fn loads_follow_a_dead_bus() {
        let mut e = new_sys();
        for i in 0..4 {
            e.panel_mut().gen_ctrl[i].set_latched(true);
        }
        e.action("crew", "press", "BATTERY", Some("ON")).unwrap();
        run(&mut e, 20.0, &flying());
        assert_eq!(e.flag("elec.load.right_aft_fuel_pump"), Some(true));
        e.faults_mut().gen_loss[1] = true;
        run(&mut e, 5.0, &flying());
        let l2 = e.flag("elec.bus.L2_235.energized").unwrap();
        assert_eq!(e.flag("elec.load.right_aft_fuel_pump"), Some(l2));
        assert_eq!(e.flag("elec.load.l2_stabilizer"), Some(l2));
    }

    #[test]
    fn load_shed_values_track_load_manager() {
        let mut e = new_sys();
        run(&mut e, 0.5, &ElectricalInputs::default());
        assert_eq!(e.signal("elec.load_shed.threshold_5min_kva"), Some(EP_5MIN_KVA));
        assert_eq!(
            e.signal("elec.load_shed.large_group_threshold_kva"),
            Some(EP_5S_KVA)
        );
        assert_eq!(
            e.signal("elec.load_shed.step_count"),
            Some(e.load_output().shed_count as f64)
        );
        assert_eq!(e.flag("elec.load.thermal_cb_loads_shed"), Some(false));
    }

    #[test]
    fn breaker_state_reads_device_truth_and_cbic_registry() {
        use crate::control::BreakerKind;
        let mut e = new_sys();
        run(&mut e, 0.5, &ElectricalInputs::default());
        assert_eq!(e.text("elec.cb.CE1471831.state").as_deref(), Some("open"));
        e.cbic_mut()
            .app
            .add_breaker(2421501, "ELCC TEST", BreakerKind::Elcc, CbState::Closed);
        assert_eq!(e.text("elec.cb.CK2421501.state").as_deref(), Some("closed"));
        e.action("crew", "press", "BATTERY", Some("ON")).unwrap();
        run(&mut e, 3.0, &flying());
        assert_eq!(
            e.pdu().channel("CE2471821").map(|c| c.state),
            Some(crate::protection::sspc::SspcState::Closed)
        );
        assert_eq!(e.text("elec.cb.prefix.ECB").as_deref(), Some("CE"));
        assert_eq!(e.text("elec.cb.prefix.TCB").as_deref(), Some("CT"));
        assert_eq!(e.text("elec.cb.prefix.ELCC").as_deref(), Some("CK"));
    }

    #[test]
    fn every_scenario_signal_in_the_load_families_is_bound() {
        let dir =
            std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../../tests/scenarios/24-electrical");
        let mut e = new_sys();
        run(&mut e, 0.5, &ElectricalInputs::default());
        let mut checked = 0;
        for entry in std::fs::read_dir(dir).unwrap() {
            let path = entry.unwrap().path();
            if path.extension().and_then(|x| x.to_str()) != Some("toml") {
                continue;
            }
            for line in std::fs::read_to_string(&path).unwrap().lines() {
                let Some(rest) = line.strip_prefix("signal = \"") else {
                    continue;
                };
                let sig = rest.trim_end_matches('"');
                let family = ["load", "load_shed", "cb", "prot", "pecs", "cmsc"]
                    .iter()
                    .any(|f| sig.starts_with(&format!("elec.{f}.")));
                if family {
                    assert!(e.value(sig).is_some(), "{sig}");
                    checked += 1;
                }
            }
        }
        assert!(checked > 80);
    }

    #[test]
    fn protection_values_come_from_ext_power_monitor_constants() {
        let e = new_sys();
        assert_eq!(e.signal("elec.prot.fwd_ep_of_trip_hz"), Some(429.0));
        assert_eq!(e.signal("elec.prot.fwd_ep_uv_rms"), Some(104.5));
        assert_eq!(e.signal("elec.prot.fwd_ep_uv_delay_s"), Some(9.0));
    }
}
