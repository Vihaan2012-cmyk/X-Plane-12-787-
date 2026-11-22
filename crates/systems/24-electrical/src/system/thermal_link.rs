use super::fault_link::norm;
use super::{Electrical, MACHINES, SignalValue, Switchgear};
use crate::faults;
use crate::heat::{ElecThermal, ModuleKind, ZONES, Zone};
use crate::net::ElectricalNet;
use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;

pub(crate) const HEAT_TICK_US: u64 = 1_000_000;
const CONTACT_R_BASE_OHM: f64 = 1.0e-4;
const FEEDER_LENGTH_M: f64 = 1.0;
const MODULE_FLOW_KG_S: f64 = 0.005;
const COIL_HEAT_W: f64 = 8.0;
const RESISTANCE_PUSH_RATIO: f64 = 0.02;
const CELL_MASS_KG: f64 = 0.9;
const MACHINE_IDS: [&str; 7] = [
    "VFSG_L1", "VFSG_L2", "VFSG_R1", "VFSG_R2", "ASG_L", "ASG_R", "RAT",
];
const BATTERY_IDS: [&str; 2] = ["BATT_MAIN", "BATT_APU"];
const GROUND_FAULT_V: f64 = 115.0;
const GROUND_FAULT_MAX_W: f64 = 3000.0;
const ARC_HEAT_W: f64 = 150.0;
const WINDING_SHORT_W: f64 = 4000.0;
const RUNAWAY_HEAT_W: f64 = 600.0;
const MODULE_OVERTEMP_W: f64 = 300.0;

struct SspcPart {
    id: String,
    wire: String,
    rpdu: Option<String>,
    spdu: Option<String>,
}

struct GearPart {
    name: String,
    wire: String,
    panel: Option<String>,
    applied_ohm: f64,
}

pub(crate) struct ThermalLink {
    pub(crate) heat: ElecThermal,
    gear: Vec<GearPart>,
    sspcs: Vec<SspcPart>,
    modules: BTreeSet<String>,
    keys: BTreeMap<String, String>,
    acc_us: u64,
    soak_k: Option<f64>,
}

fn register(keys: &mut BTreeMap<String, String>, id: &str) {
    keys.insert(norm(id), id.to_string());
}

fn zone_for_name(name: &str) -> Zone {
    let aft = name.starts_with("R ")
        || name.starts_with("R1")
        || name.starts_with("R2")
        || name.contains(" R ")
        || name.starts_with("ASG")
        || name.ends_with("_R")
        || name.contains("APU")
        || name.starts_with("LA");
    if aft { Zone::EeBayAft } else { Zone::EeBayFwd }
}

fn zone_for_number(n: u32) -> Zone {
    [Zone::Cabin, Zone::Crown, Zone::EeBayFwd][(n % 3) as usize]
}

fn rating_for(name: &str, listed: Option<f64>) -> f64 {
    if let Some(r) = listed {
        return r;
    }
    if name.contains("GCB") || name.contains("APB") {
        300.0
    } else if name.contains("BTB 115") {
        150.0
    } else if name.contains("BTB") || name.contains("EPC") || name.contains("BSB") || name.contains("ATUC") {
        250.0
    } else if name.contains("RCB") {
        150.0
    } else if name.contains("ATRUC") || name.contains("TRU") || name.contains("MBR") {
        100.0
    } else {
        50.0
    }
}

fn parse_rating(s: &str) -> Option<f64> {
    let t: String = s.chars().filter(|c| c.is_ascii_digit() || *c == '.').collect();
    t.parse::<f64>().ok().filter(|v| *v > 0.0)
}

fn rpdu_in(text: &str) -> Option<String> {
    let i = text.find("RPDU ")?;
    let digits: String = text[i + 5..].chars().take(2).collect();
    if digits.len() == 2 && digits.chars().all(|c| c.is_ascii_digit()) {
        Some(format!("RPDU {digits}"))
    } else {
        None
    }
}

fn csv_rows(path: &Path) -> Vec<Vec<String>> {
    let text = std::fs::read_to_string(path).unwrap_or_default();
    text.lines()
        .skip(1)
        .map(|l| l.split(',').map(|s| s.to_string()).collect())
        .collect()
}

fn zone_temp_k(zone: Zone, oat_k: f64, engines: bool) -> f64 {
    let base = zone.default_k();
    match zone {
        Zone::EeBayFwd | Zone::EeBayAft => base + 0.25 * (oat_k - 288.15),
        Zone::Cabin => base,
        Zone::Crown => 0.5 * (zone.default_k() + oat_k) + 5.0,
        Zone::WheelWell => oat_k + 8.0,
        Zone::Nacelle => oat_k + if engines { 30.0 } else { 5.0 },
    }
}

impl ThermalLink {
    pub(crate) fn build(data_dir: &Path, gear: &[Switchgear], net: &ElectricalNet) -> ThermalLink {
        let mut heat = ElecThermal::new();
        let mut keys: BTreeMap<String, String> = BTreeMap::new();
        let mut modules: BTreeSet<String> = BTreeSet::new();
        let mut parts: Vec<GearPart> = Vec::new();
        let contact_rows = csv_rows(&data_dir.join("contactors.csv"));
        let mut listed: BTreeMap<String, (Option<String>, Option<f64>)> = BTreeMap::new();
        for f in &contact_rows {
            if f.len() < 7 || f[1].is_empty() || f[1].starts_with('"') {
                continue;
            }
            let panel = if f[4].is_empty() { None } else { Some(f[4].clone()) };
            listed.entry(f[1].clone()).or_insert((panel, parse_rating(&f[6])));
        }
        for g in gear {
            let (panel, rating) = listed.get(&g.name).cloned().unwrap_or((None, None));
            let r = rating_for(&g.name, rating);
            let zone = zone_for_name(&g.name);
            let wire = format!("W:{}", g.name);
            if heat.add_contactor(&g.name, zone, r).is_err() {
                continue;
            }
            let _ = heat.add_wire(&wire, zone, r * 1.25, FEEDER_LENGTH_M);
            register(&mut keys, &g.name);
            register(&mut keys, &wire);
            parts.push(GearPart {
                name: g.name.clone(),
                wire,
                panel,
                applied_ohm: CONTACT_R_BASE_OHM,
            });
        }
        let mut panels: BTreeSet<String> = BTreeSet::new();
        for p in &parts {
            if let Some(panel) = &p.panel {
                panels.insert(format!("PNL {panel}"));
            }
        }
        for (k, id) in panels.iter().enumerate() {
            let zone = zone_for_number(k as u32);
            if heat.add_module(id, zone, ModuleKind::PanelBox, 20.0).is_ok() {
                modules.insert(id.clone());
                register(&mut keys, id);
            }
        }
        let rpdu_rows = csv_rows(&data_dir.join("rpdus.csv"));
        for f in &rpdu_rows {
            let Some(id) = f.first().filter(|s| s.starts_with("RPDU ")) else {
                continue;
            };
            let n: u32 = id[5..].trim().parse().unwrap_or(0);
            if heat
                .add_module(id, zone_for_number(n), ModuleKind::Rpdu, 25.0)
                .is_ok()
            {
                modules.insert(id.clone());
                register(&mut keys, id);
            }
        }
        let breaker_rows = csv_rows(&data_dir.join("breakers.csv"));
        let mut breaker: BTreeMap<String, (Option<String>, Option<String>, f64)> = BTreeMap::new();
        for f in &breaker_rows {
            if f.len() < 8 {
                continue;
            }
            let spdu = if f[4].contains("SPDU") {
                Some(f[4].clone())
            } else {
                None
            };
            let rating = parse_rating(&f[7]).unwrap_or(10.0);
            breaker.insert(f[0].clone(), (rpdu_in(&f[1]), spdu, rating));
        }
        let mut sspcs = Vec::new();
        for id in net.sspc_ids() {
            let (rpdu, spdu, rating) = if let Some(rest) = id.strip_prefix("RPDU") {
                let n: String = rest.trim().chars().take(2).collect();
                (Some(format!("RPDU {n}")), None, 10.0)
            } else {
                breaker.get(id).cloned().unwrap_or((None, None, 10.0))
            };
            let n: u32 = rpdu
                .as_deref()
                .and_then(|r| r.get(5..))
                .and_then(|d| d.parse().ok())
                .unwrap_or(0);
            let zone = zone_for_number(n);
            let wire = format!("W:{id}");
            if heat.add_wire(&wire, zone, rating * 1.25, FEEDER_LENGTH_M).is_ok() {
                register(&mut keys, &wire);
            }
            if let Some(s) = &spdu
                && !modules.contains(s)
            {
                let z = if s.starts_with('L') {
                    Zone::EeBayFwd
                } else {
                    Zone::EeBayAft
                };
                if heat.add_module(s, z, ModuleKind::Spdu, 60.0).is_ok() {
                    modules.insert(s.clone());
                    register(&mut keys, s);
                }
            }
            sspcs.push(SspcPart {
                id: id.clone(),
                wire,
                rpdu,
                spdu,
            });
        }
        for s in ["L SPDU", "R SPDU"] {
            if !modules.contains(s) {
                let z = if s.starts_with('L') {
                    Zone::EeBayFwd
                } else {
                    Zone::EeBayAft
                };
                if heat.add_module(s, z, ModuleKind::Spdu, 60.0).is_ok() {
                    modules.insert(s.to_string());
                    register(&mut keys, s);
                }
            }
        }
        for m in MACHINES {
            let id = format!("GCU_{m}");
            let z = if m.starts_with("VFSG") {
                Zone::EeBayFwd
            } else {
                Zone::EeBayAft
            };
            if heat.add_module(&id, z, ModuleKind::PanelBox, 30.0).is_ok() {
                modules.insert(id.clone());
                register(&mut keys, &id);
            }
        }
        for id in ["BPCU_L", "BPCU_R"] {
            let z = if id.ends_with('L') {
                Zone::EeBayFwd
            } else {
                Zone::EeBayAft
            };
            if heat.add_module(id, z, ModuleKind::PanelBox, 40.0).is_ok() {
                modules.insert(id.to_string());
                register(&mut keys, id);
            }
        }
        for (k, id) in BATTERY_IDS.iter().enumerate() {
            let r = if k == 0 {
                net.battery().params.r0_cell_25c_ohm
            } else {
                net.battery_apu().params.r0_cell_25c_ohm
            };
            let z = if k == 0 { Zone::EeBayFwd } else { Zone::EeBayAft };
            let _ = heat.add_battery(id, z, 8, CELL_MASS_KG, r);
            register(&mut keys, id);
        }
        for (k, id) in MACHINE_IDS.iter().enumerate() {
            let (zone, rated_a, loss) = match k {
                0..=3 => (Zone::Nacelle, 260.0, 8000.0),
                4 | 5 => (Zone::EeBayAft, 260.0, 6000.0),
                _ => (Zone::WheelWell, 100.0, 3000.0),
            };
            let _ = heat.add_generator(id, zone, rated_a, loss);
            register(&mut keys, id);
        }
        for m in &modules {
            heat.set_cooling_flow(m, MODULE_FLOW_KG_S);
        }
        ThermalLink {
            heat,
            gear: parts,
            sspcs,
            modules,
            keys,
            acc_us: 0,
            soak_k: None,
        }
    }

    fn fault_effects(&self, e: &Electrical) -> Effects {
        let now = e.now_us;
        let book = &e.fault_link.book;
        let mut fx = Effects::default();
        let mut parts: Vec<&str> = e.fault_link.active_parts();
        parts.dedup();
        for part in parts {
            let Some(t) = book.part_type(part) else {
                continue;
            };
            match t {
                faults::CONTACTOR => {
                    fx.mult(part, book.contactor(part, now).resistance_mult);
                }
                faults::WIRE_SEGMENT => {
                    let w = book.wire(part, now);
                    fx.mult(part, w.resistance_mult);
                    if w.arcing {
                        fx.extra(part, ARC_HEAT_W);
                    }
                    if let Some(r) = w.ground_fault_ohm {
                        fx.extra(
                            part,
                            (GROUND_FAULT_V * GROUND_FAULT_V / r).min(GROUND_FAULT_MAX_W),
                        );
                    }
                }
                faults::CONNECTOR => {
                    let target = format!("W:{}", part.strip_prefix("CONN:").unwrap_or(part));
                    fx.mult(&target, book.connector(part, now).resistance_mult);
                }
                faults::VFSG | faults::ASG | faults::RAT_GEN => {
                    let m = book.machine(part, now);
                    let mut w = m.friction_loss_w;
                    if m.winding_short {
                        w += WINDING_SHORT_W;
                    }
                    fx.extra(part, w);
                    fx.oil(part, m.oil_flow_scale);
                }
                faults::BATTERY_CELL => {
                    let pack = part.split_once(".c").map_or(part, |p| p.0);
                    let c = book.battery_cell(part, now);
                    let mut w = c.extra_heat_w;
                    if let Some(r) = c.internal_short_ohm {
                        w += 3.6 * 3.6 / r;
                    }
                    fx.extra(pack, w);
                }
                faults::BATTERY_PACK => {
                    fx.extra(
                        part,
                        RUNAWAY_HEAT_W * book.magnitude(part, "thermal-runaway", now),
                    );
                }
                faults::RPDU_MODULE | faults::SPDU_CARD => {
                    let id = match part {
                        "SPDU_L" => "L SPDU",
                        "SPDU_R" => "R SPDU",
                        other => other,
                    };
                    fx.extra(id, MODULE_OVERTEMP_W * book.magnitude(part, "overtemp", now));
                    fx.flow(id, 1.0 - book.magnitude(part, "cooling-loss", now));
                }
                faults::OIL_AIR_VALVE => {
                    let tag = part.strip_prefix("OAV_").unwrap_or(part);
                    let closed = book.magnitude(part, "locked-closed", now);
                    for m in MACHINE_IDS {
                        if m.ends_with(tag)
                            || (tag.len() == 1 && m.starts_with("VFSG_") && m[5..].starts_with(tag))
                        {
                            fx.oil(m, 1.0 - closed);
                        }
                    }
                }
                faults::PECS_PUMP => {
                    let zone = match part {
                        "PECS_1" => Some(Zone::EeBayFwd),
                        "PECS_2" => Some(Zone::EeBayAft),
                        _ => None,
                    };
                    let loss = book.magnitude(part, "pump-loss", now);
                    let rise = book.magnitude(part, "coolant-temp-rise", now);
                    for m in &self.modules {
                        if zone.is_some_and(|z| self.heat.zone_of(m) == Some(z)) {
                            fx.flow(m, (1.0 - loss) * (1.0 - 0.5 * rise));
                        }
                    }
                }
                faults::GCU | faults::BPCU => {
                    let f = book.magnitude(part, "fan-failure", now);
                    fx.flow(part, 1.0 - f);
                }
                _ => {}
            }
        }
        fx
    }

    fn tick(&mut self, e: &mut Electrical, dt_us: u64) {
        let oat_k = e.inputs.ambient_c + 273.15;
        let engines = e.inputs.engine_running.iter().any(|r| *r);
        for z in ZONES {
            let k = self.soak_k.unwrap_or_else(|| zone_temp_k(z, oat_k, engines));
            self.heat.set_zone_temp(z, k);
            match (self.soak_k, z) {
                (Some(s), _) => self.heat.set_cooling_air_k(z, s),
                (None, Zone::Nacelle | Zone::WheelWell | Zone::Crown) => {
                    self.heat.set_cooling_air_k(z, k - 3.0)
                }
                _ => {}
            }
        }
        let fx = self.fault_effects(e);
        let mut module_a: BTreeMap<&str, f64> = BTreeMap::new();
        let mut battery_a = [0.0; 2];
        let mut machine_a = [0.0; 7];
        if let Some(s) = e.plant.snapshot() {
            for c in &s.contactors {
                self.heat.set_current(&c.name, c.current_a);
                self.heat.set_current(&format!("W:{}", c.name), c.current_a);
            }
            for p in &self.sspcs {
                let a = s.sspc(&p.id).map_or(0.0, |r| r.current_a.abs());
                self.heat.set_current(&p.wire, a);
                if let Some(r) = &p.rpdu {
                    *module_a.entry(r.as_str()).or_insert(0.0) += a;
                }
                if let Some(sp) = &p.spdu {
                    *module_a.entry(sp.as_str()).or_insert(0.0) += a;
                }
            }
            for (k, b) in s.batteries.iter().enumerate().take(2) {
                battery_a[k] = b.current_a.abs();
            }
            for (k, name) in MACHINE_IDS.iter().enumerate() {
                machine_a[k] = s.machine(name).map_or(0.0, |m| m.current_a);
            }
            for p in &self.gear {
                if let Some(panel) = &p.panel {
                    let a = s.contactor(&p.name).map_or(0.0, |c| c.current_a);
                    *module_a.entry(panel.as_str()).or_insert(0.0) += a;
                }
            }
        }
        for g in &self.gear {
            let closed = e
                .gear
                .iter()
                .find(|x| x.name == g.name)
                .is_some_and(|x| x.out.contacts_closed);
            let coil = if closed { COIL_HEAT_W } else { 0.0 };
            self.heat.set_dissipation(&g.name, coil + fx.extra_of(&g.name));
            self.heat.set_resistance_multiplier(&g.name, fx.mult_of(&g.name));
            self.heat.set_dissipation(&g.wire, fx.extra_of(&g.wire));
            self.heat.set_resistance_multiplier(&g.wire, fx.mult_of(&g.wire));
        }
        for p in &self.sspcs {
            self.heat.set_dissipation(&p.wire, fx.extra_of(&p.wire));
            self.heat.set_resistance_multiplier(&p.wire, fx.mult_of(&p.wire));
        }
        for m in &self.modules {
            let a = module_a.get(m.as_str()).copied().unwrap_or(0.0);
            let a = if m.starts_with("PNL ") {
                module_a
                    .get(m.strip_prefix("PNL ").unwrap_or(""))
                    .copied()
                    .unwrap_or(0.0)
            } else {
                a
            };
            self.heat.set_current(m, a);
            self.heat.set_dissipation(m, fx.extra_of(m));
            self.heat.set_cooling_flow(m, MODULE_FLOW_KG_S * fx.flow_of(m));
        }
        for (k, id) in BATTERY_IDS.iter().enumerate() {
            self.heat.set_current(id, battery_a[k]);
            self.heat.set_dissipation(id, fx.extra_of(id));
        }
        for (k, id) in MACHINE_IDS.iter().enumerate() {
            self.heat.set_current(id, machine_a[k]);
            self.heat.set_dissipation(id, fx.extra_of(id));
            self.heat.set_oil_cooling(id, fx.oil_of(id));
        }
        self.heat.step(dt_us);
        self.feed_back(e, &fx);
    }

    fn feed_back(&mut self, e: &mut Electrical, fx: &Effects) {
        for g in self.gear.iter_mut() {
            let cf = self.heat.resistance_factor(&g.name).unwrap_or(1.0);
            let wire_ohm = self.heat.resistance_ohm(&g.wire).unwrap_or(0.0) * fx.mult_of(&g.wire);
            let r = CONTACT_R_BASE_OHM * cf * fx.mult_of(&g.name)
                + wire_ohm
                + e.fault_link.applied.series_for_gear(&g.name);
            if (r - g.applied_ohm).abs() > RESISTANCE_PUSH_RATIO * g.applied_ohm {
                e.plant.net_mut().set_contactor_resistance_ohm(&g.name, r);
                g.applied_ohm = r;
            }
        }
        for (k, id) in BATTERY_IDS.iter().enumerate() {
            if let Some(t) = self.heat.temperature_k(id)
                && let Some(pack) = e.plant.net_mut().battery_pack_mut(k)
            {
                pack.set_temp_c(t - 273.15);
            }
        }
        for id in MACHINE_IDS {
            if let Some(t) = self.heat.temperature_k(id) {
                e.plant.net_mut().set_machine_oil_inlet_c(id, t - 273.15);
            }
        }
    }

    pub(crate) fn applied_ohm(&self, name: &str) -> Option<f64> {
        self.gear.iter().find(|g| g.name == name).map(|g| g.applied_ohm)
    }
}

#[derive(Default)]
struct Effects {
    mult: BTreeMap<String, f64>,
    extra: BTreeMap<String, f64>,
    flow: BTreeMap<String, f64>,
    oil: BTreeMap<String, f64>,
}

impl Effects {
    fn mult(&mut self, id: &str, m: f64) {
        *self.mult.entry(id.to_string()).or_insert(1.0) *= m;
    }
    fn extra(&mut self, id: &str, w: f64) {
        *self.extra.entry(id.to_string()).or_insert(0.0) += w;
    }
    fn flow(&mut self, id: &str, f: f64) {
        *self.flow.entry(id.to_string()).or_insert(1.0) *= f.clamp(0.0, 1.0);
    }
    fn oil(&mut self, id: &str, f: f64) {
        *self.oil.entry(id.to_string()).or_insert(1.0) *= f.clamp(0.0, 1.0);
    }
    fn mult_of(&self, id: &str) -> f64 {
        self.mult.get(id).copied().unwrap_or(1.0)
    }
    fn extra_of(&self, id: &str) -> f64 {
        self.extra.get(id).copied().unwrap_or(0.0)
    }
    fn flow_of(&self, id: &str) -> f64 {
        self.flow.get(id).copied().unwrap_or(1.0)
    }
    fn oil_of(&self, id: &str) -> f64 {
        self.oil.get(id).copied().unwrap_or(1.0)
    }
}

impl Electrical {
    pub(crate) fn stage_heat(&mut self, dt_us: u64) {
        let Some(mut th) = self.thermal.take() else {
            return;
        };
        th.acc_us += dt_us;
        if th.acc_us >= HEAT_TICK_US {
            let step = th.acc_us;
            th.acc_us = 0;
            th.tick(self, step);
        }
        self.thermal = Some(th);
    }

    pub fn heat_advance(&mut self, dt_us: u64) {
        let Some(mut th) = self.thermal.take() else {
            return;
        };
        let mut left = dt_us;
        while left > 0 {
            let h = left.min(HEAT_TICK_US);
            th.tick(self, h);
            left -= h;
        }
        self.thermal = Some(th);
    }

    pub fn cold_soak(&mut self, kelvin: f64) {
        if let Some(th) = self.thermal.as_mut() {
            th.soak_k = Some(kelvin);
            for z in ZONES {
                th.heat.cold_soak(z, kelvin);
            }
        }
    }

    pub fn end_cold_soak(&mut self) {
        if let Some(th) = self.thermal.as_mut() {
            th.soak_k = None;
        }
    }

    pub fn heat(&self) -> Option<&ElecThermal> {
        self.thermal.as_ref().map(|t| &t.heat)
    }

    pub fn part_temp_c(&self, id: &str) -> Option<f64> {
        self.heat()?.temperature_k(id).map(|k| k - 273.15)
    }

    pub fn wire_temp_c(&self, feeder: &str) -> Option<f64> {
        self.part_temp_c(&format!("W:{feeder}"))
    }

    pub fn contactor_temp_c(&self, name: &str) -> Option<f64> {
        self.part_temp_c(name)
    }

    pub fn feeder_resistance_ohm(&self, contactor: &str) -> Option<f64> {
        self.thermal.as_ref()?.applied_ohm(contactor)
    }

    pub(crate) fn thermal_value(&self, name: &str) -> Option<SignalValue> {
        let rest = name.strip_prefix("elec.temp.")?;
        let th = self.thermal.as_ref()?;
        let id = th.keys.get(rest)?;
        th.heat.temperature_k(id).map(|k| SignalValue::Num(k - 273.15))
    }

    pub fn thermal_part_ids(&self) -> Vec<String> {
        self.thermal
            .as_ref()
            .map(|t| t.keys.keys().cloned().collect())
            .unwrap_or_default()
    }
}
