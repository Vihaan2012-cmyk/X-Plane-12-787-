use super::{BUS_LIVE_V, Electrical};
use crate::crew::{CrewInputs, Lamp, PanelLamps, PanelOutputs};
use crate::faults::{self, BINARY_THRESHOLD, resistance_multiplier};
use std::collections::{BTreeMap, BTreeSet};

const STUD_OHM_PER_MULT: f64 = 0.03;
const STUD_OPEN_OHM: f64 = 20.0;
const LAMP_SHORT_OHM: f64 = 56.0;
const HEATER_OHM: f64 = 2.6;
const HEATER_COLD_C: f64 = 5.0;
const GROUND_TRIP_CONTACTOR_A: f64 = 300.0;
const WINDING_SHORT_OHM: f64 = 0.5;
const APPROX_CIRCUIT_WIRE_OHM: f64 = 0.05;
const APPROX_CIRCUIT_TRIP_A: f64 = 15.0;
const ATU_SHORT_OHM: f64 = 2.0;
const GEN_FEEDERS: [&str; 6] = ["L1 GCB", "L2 GCB", "R1 GCB", "R2 GCB", "L APB", "R APB"];
const ATRU_GF_OHM: f64 = 200.0;
const SPU_SHORT_OHM: f64 = 5.0;
const CHATTER_US: u64 = 20_000;
const BOUNCE_US: u64 = 20_000;
const FLICKER_US: u64 = 100_000;

#[derive(Clone, Debug, Default, PartialEq)]
pub(crate) struct NetState {
    shunts: BTreeMap<String, (String, f64)>,
    phase_shunts: BTreeMap<String, (String, f64)>,
    load_scale: BTreeMap<String, f64>,
    conv: BTreeMap<String, (f64, f64, bool)>,
    series: BTreeMap<String, f64>,
    diode: BTreeMap<String, (Option<bool>, f64)>,
    comms: BTreeSet<String>,
    ctrl_power: BTreeSet<String>,
}

impl NetState {
    pub(crate) fn is_empty(&self) -> bool {
        *self == NetState::default()
    }

    pub(crate) fn series_for_gear(&self, gear: &str) -> f64 {
        gear.strip_prefix("AC FEED RPDU ")
            .and_then(|n| self.series.get(n.trim()))
            .copied()
            .unwrap_or(0.0)
    }
}

fn on(m: f64) -> bool {
    m >= BINARY_THRESHOLD
}

fn parity(n: &str) -> Option<u32> {
    n.parse::<u32>().ok().map(|v| v % 2)
}

fn gear_for_rpdu_ac(num: &str) -> String {
    format!("AC FEED RPDU {num}")
}

fn lamp_mut<'a>(l: &'a mut PanelLamps, name: &str) -> Option<&'a mut Lamp> {
    let gi = |s: &str| ["L1", "L2", "R1", "R2"].iter().position(|g| *g == s);
    Some(match name {
        "BATTERY" => &mut l.battery_off,
        "CABIN_UTIL" => &mut l.cabin_util_off,
        "IFE_SEATS" => &mut l.ife_off,
        "APU_GEN_L" => &mut l.apu_gen_off[0],
        "APU_GEN_R" => &mut l.apu_gen_off[1],
        "GROUND_SERVICE" => &mut l.ground_service_on,
        other => {
            let i = gi(other
                .strip_prefix("GEN_CTRL_")
                .or_else(|| other.strip_prefix("GEN_"))?)?;
            &mut l.gen_off[i]
        }
    })
}

fn switch_mut<'a>(o: &'a mut PanelOutputs, name: &str) -> Option<&'a mut bool> {
    let gi = |s: &str| ["L1", "L2", "R1", "R2"].iter().position(|g| *g == s);
    Some(match name {
        "BATTERY" => &mut o.battery_on,
        "CABIN_UTIL" => &mut o.cabin_util_on,
        "IFE_SEATS" => &mut o.ife_on,
        "APU_GEN_L" => &mut o.apu_gen_on[0],
        "APU_GEN_R" => &mut o.apu_gen_on[1],
        "GROUND_SERVICE" => &mut o.ground_service_on,
        other => {
            let i = gi(other.strip_prefix("GEN_CTRL_")?)?;
            &mut o.gen_ctrl_on[i]
        }
    })
}

fn dark(l: &mut Lamp) {
    l.lit = false;
    l.brightness = 0.0;
}

impl Electrical {
    fn active_parts_of(&self, t: &str) -> Vec<String> {
        let mut v: Vec<String> = self
            .fault_link
            .active_parts()
            .into_iter()
            .filter(|p| self.fault_link.book.part_type(p) == Some(t))
            .map(str::to_string)
            .collect();
        v.dedup();
        v
    }

    fn mag(&self, part: &str, mode: &str) -> f64 {
        self.fault_link.book.magnitude(part, mode, self.now_us)
    }

    fn rpdus_of_parity(&self, parity_want: Option<u32>, ac_only: bool) -> Vec<String> {
        self.plant
            .net()
            .rpdu_numbers()
            .into_iter()
            .filter(|n| parity(n) == parity_want && (!ac_only || self.plant.net().rpdu_has_ac(n)))
            .collect()
    }

    fn elcc_rpdus(&self, part: &str) -> Vec<String> {
        let tag = part.strip_prefix("ELCC_").unwrap_or(part);
        let ac: Vec<String> = self
            .plant
            .net()
            .rpdu_numbers()
            .into_iter()
            .filter(|n| self.plant.net().rpdu_has_ac(n))
            .collect();
        if ac.iter().any(|n| n == tag) {
            return vec![tag.to_string()];
        }
        match tag.parse::<usize>() {
            Ok(i) if (1..=ac.len()).contains(&i) => vec![ac[i - 1].clone()],
            _ => Vec::new(),
        }
    }

    fn elcu_rpdus(&self, part: &str) -> Vec<String> {
        let n = part
            .strip_prefix("ELCU_")
            .and_then(|t| t.parse::<u32>().ok())
            .unwrap_or(1);
        self.rpdus_of_parity(Some(n % 2), true)
    }

    fn first_feed_sspcs(&self, rpdus: &[String]) -> Vec<String> {
        let feeds = self.plant.net().feed_sspc_ids();
        let mut out = Vec::new();
        for r in rpdus {
            let key = format!("RPDU {r}");
            if let Some(id) = feeds
                .iter()
                .find(|id| self.plant.net().sspc_rpdu(id).as_deref() == Some(key.as_str()))
            {
                out.push(id.clone());
            }
        }
        out
    }

    fn all_feed_sspcs(&self, rpdu: &str) -> Vec<String> {
        self.plant
            .net()
            .feed_sspc_ids()
            .into_iter()
            .filter(|id| self.plant.net().sspc_rpdu(id).as_deref() == Some(rpdu))
            .collect()
    }

    fn stud_rpdus(&self, part: &str) -> Vec<String> {
        let tag = part.strip_prefix("GND:").unwrap_or(part);
        if let Some(n) = tag.strip_prefix("RPDU ") {
            return vec![n.trim().to_string()];
        }
        match tag {
            "P100" | "P300" | "P700" | "P150" => self.rpdus_of_parity(Some(1), false),
            "P200" | "P400" => self.rpdus_of_parity(Some(0), false),
            _ => Vec::new(),
        }
    }

    pub(crate) fn causal_heater(&mut self) {
        let stuck = on(self.mag("RATHTR", "thermostat-stuck"));
        let open = on(self.mag("RATHTR", "element-open"));
        let cold = self.inputs.ambient_c < HEATER_COLD_C;
        let want = (cold || stuck) && !open;
        if want != self.fault_link.heater_on {
            self.fault_link.heater_on = want;
            self.plant.net_mut().set_shunt(
                "rat-heater",
                "HBB_MAIN",
                if want { Some(HEATER_OHM) } else { None },
            );
        }
    }

    pub(crate) fn causal_post(&mut self, sspc_live: &mut BTreeSet<String>) {
        let now = self.now_us;
        let mut want = NetState::default();
        let mut open_gears: Vec<String> = Vec::new();
        let mut weld_gears: Vec<String> = Vec::new();
        let mut open_sspcs: Vec<String> = Vec::new();
        let mut closed_sspcs: Vec<String> = Vec::new();
        let mut trips: Vec<String> = Vec::new();
        let mut parts: Vec<String> = self
            .fault_link
            .active_parts()
            .into_iter()
            .map(str::to_string)
            .collect();
        parts.dedup();
        for part in &parts {
            let Some(t) = self.fault_link.book.part_type(part).map(str::to_string) else {
                continue;
            };
            let book = &self.fault_link.book;
            match t.as_str() {
                faults::DIODE_MODULE => {
                    let batt = format!("BATT_{}", part.strip_prefix("DIODE_").unwrap_or("MAIN"));
                    let force = if on(book.magnitude(part, "short", now)) {
                        Some(true)
                    } else if on(book.magnitude(part, "open", now)) {
                        Some(false)
                    } else {
                        None
                    };
                    let drop = 2.0 * book.magnitude(part, "forward-drop-rise", now);
                    want.diode.insert(batt, (force, drop));
                }
                faults::VFSG | faults::ASG => {
                    let m = book.magnitude(part, "winding-short", now);
                    let k = super::MACHINES.iter().position(|n| n == part).filter(|k| *k < 6);
                    if m > 0.0
                        && let Some(k) = k
                        && let Some(site) = self.plant.net().shunt_site(GEN_FEEDERS[k])
                    {
                        want.phase_shunts
                            .insert(format!("{part}.ws"), (site, WINDING_SHORT_OHM / m.min(1.0)));
                    }
                }
                faults::ATRU => {
                    let tag = part.strip_prefix("ATRU_").unwrap_or("L1");
                    let c = book.converter(part, now);
                    let sat = book.magnitude(part, "saturation", now);
                    let z = 1.0 / c.cooling_scale.max(0.1) * (1.0 + 4.0 * sat);
                    want.conv.insert(part.clone(), (c.output_scale, z, false));
                    if c.ground_fault {
                        want.shunts.insert(
                            format!("{part}.gf"),
                            (
                                format!("HVDC_{tag}"),
                                ATRU_GF_OHM / book.magnitude(part, "ground-fault", now).max(0.5),
                            ),
                        );
                    }
                    if c.forced_trip {
                        open_gears.push(format!("{tag} ATRUC"));
                        trips.push(part.clone());
                    }
                }
                faults::ATU => {
                    let tag = part.strip_prefix("ATU_").unwrap_or("L");
                    let sh = book.magnitude(part, "winding-short", now);
                    let ot = book.magnitude(part, "overtemp", now);
                    let gain = (1.0 - 0.3 * sh) * (1.0 - 0.1 * ot);
                    let z = 1.0 + 20.0 * sh + 4.0 * ot;
                    want.conv.insert(part.clone(), (gain, z, false));
                    want.conv.insert(format!("{part}_115"), (gain, z, false));
                    if sh > 0.0 {
                        want.phase_shunts
                            .insert(format!("{part}.ws"), (part.clone(), ATU_SHORT_OHM / sh.min(1.0)));
                    }
                    if on(book.magnitude(part, "contactor-failed-open", now)) {
                        open_gears.push(format!("{tag} ATUC"));
                    }
                }
                faults::TRU => {
                    let c = book.converter(part, now);
                    let z = 1.0 / c.cooling_scale.max(0.1) * (1.0 + 2.0 * c.ripple);
                    let open = on(book.magnitude(part, "overcurrent-trip", now));
                    want.conv.insert(part.clone(), (c.output_scale, z, open));
                }
                faults::SPU => {
                    let m = book.magnitude(part, "short", now);
                    if m > 0.0 {
                        want.shunts.insert(
                            "SPU.short".to_string(),
                            ("SPU_OUT".to_string(), SPU_SHORT_OHM / m.max(0.1)),
                        );
                    }
                }
                faults::ELCC => {
                    let rp = self.elcc_rpdus(part);
                    let open = on(book.magnitude(part, "stuck-open", now));
                    let nuisance = on(book.magnitude(part, "nuisance-trip", now));
                    let weld = on(book.magnitude(part, "stuck-closed", now));
                    for n in rp {
                        let g = gear_for_rpdu_ac(&n);
                        if open || nuisance {
                            open_gears.push(g.clone());
                        } else if weld {
                            weld_gears.push(g.clone());
                        }
                        if nuisance {
                            trips.push(part.clone());
                        }
                    }
                }
                faults::ELCU => {
                    let failed = on(book.magnitude(part, "internal-fault", now))
                        || on(book.magnitude(part, "power-sense-fail", now));
                    if failed {
                        for n in self.elcu_rpdus(part) {
                            open_gears.push(gear_for_rpdu_ac(&n));
                        }
                    }
                }
                faults::TCB => {
                    let feeds = self.plant.net().feed_sspc_ids();
                    let k = part
                        .strip_prefix("TCB_")
                        .and_then(|t| t.parse::<usize>().ok())
                        .unwrap_or(1)
                        .max(1);
                    if let Some(id) = feeds.get((k - 1) % feeds.len().max(1)) {
                        if on(book.magnitude(part, "nuisance-trip", now)) {
                            open_sspcs.push(id.clone());
                            trips.push(part.clone());
                        } else if on(book.magnitude(part, "stuck-open", now)) {
                            open_sspcs.push(id.clone());
                        } else if on(book.magnitude(part, "stuck-closed", now)) {
                            closed_sspcs.push(id.clone());
                        }
                    }
                }
                faults::SPDU_CARD => {
                    let par = if part == "SPDU_R" { 0 } else { 1 };
                    if on(book.magnitude(part, "card-fail", now))
                        || on(book.magnitude(part, "backplane-fail", now))
                    {
                        let rp = self.rpdus_of_parity(Some(par), false);
                        open_sspcs.extend(self.first_feed_sspcs(&rp));
                    }
                }
                faults::RPDU_MODULE => {
                    if on(book.magnitude(part, "power-module-fail", now)) {
                        open_sspcs.extend(self.all_feed_sspcs(part));
                    }
                }
                faults::RPDU_CONTROLLER => {
                    let rpdu = part.replacen("RPDUC ", "RPDU ", 1);
                    let comms = [
                        "gateway-fail",
                        "comms-loss",
                        "backplane-fail",
                        "micro-channel-fail",
                    ]
                    .iter()
                    .any(|m| on(book.magnitude(part, m, now)));
                    if comms {
                        want.comms.insert(rpdu.clone());
                    }
                    if on(book.magnitude(part, "power-loss", now)) {
                        want.ctrl_power.insert(rpdu);
                    }
                }
                faults::GROUND_STUD => {
                    let ohm = if on(book.magnitude(part, "open", now)) {
                        STUD_OPEN_OHM
                    } else {
                        STUD_OHM_PER_MULT
                            * (resistance_multiplier(book.magnitude(part, "high-resistance", now)) - 1.0)
                    };
                    if ohm > 0.0 {
                        for n in self.stud_rpdus(part) {
                            let e = want.series.entry(n).or_insert(0.0);
                            *e += ohm;
                        }
                    }
                }
                faults::LAMP => {
                    let m = book.magnitude(part, "short", now);
                    if m > 0.0 {
                        want.shunts.insert(
                            format!("{part}.short"),
                            ("DC28_L".to_string(), LAMP_SHORT_OHM / m),
                        );
                    }
                }
                faults::WIRE_SEGMENT | faults::CONNECTOR => {
                    let (target, gf, single_phase) = if t == faults::WIRE_SEGMENT {
                        let e = book.wire(part, now);
                        (
                            part.strip_prefix("W:").unwrap_or(part).to_string(),
                            if e.bus_short {
                                Some(0.05)
                            } else {
                                e.ground_fault_ohm
                            },
                            !e.bus_short,
                        )
                    } else {
                        let e = book.connector(part, now);
                        (
                            part.strip_prefix("CONN:").unwrap_or(part).to_string(),
                            if e.pin_short { Some(2.0) } else { None },
                            true,
                        )
                    };
                    if let Some(bus) = self.plant.net().circuit_bus(&target).map(str::to_string) {
                        let open = if t == faults::WIRE_SEGMENT {
                            let e = book.wire(part, now);
                            e.open || e.arcing
                        } else {
                            book.connector(part, now).open
                        };
                        let mult = if t == faults::WIRE_SEGMENT {
                            book.wire(part, now).resistance_mult
                        } else {
                            book.connector(part, now).resistance_mult
                        };
                        let v = self.plant.bus_v(&bus);
                        let tripped = self.fault_link.tripped.contains(part);
                        let base = self
                            .fault_link
                            .load_base
                            .get(&target)
                            .copied()
                            .or_else(|| self.plant.net().load_demand_w(&target))
                            .unwrap_or(0.0);
                        if open || tripped {
                            want.load_scale.insert(target.clone(), 0.0);
                        } else if mult > 1.0 && v > 1.0 {
                            let r = APPROX_CIRCUIT_WIRE_OHM * (mult - 1.0);
                            want.load_scale.insert(target.clone(), 1.0 + base * r / (v * v));
                        }
                        if let Some(r) = gf
                            && !tripped
                        {
                            if self.plant.net().is_ac_node(&bus) && single_phase {
                                want.phase_shunts.insert(format!("{part}.gf"), (bus.clone(), r));
                            } else {
                                want.shunts.insert(format!("{part}.gf"), (bus.clone(), r));
                            }
                            if v / r.max(1.0e-3) > APPROX_CIRCUIT_TRIP_A {
                                trips.push(part.clone());
                            }
                        }
                        if t == faults::WIRE_SEGMENT && book.wire(part, now).arcing {
                            trips.push(part.clone());
                        }
                        continue;
                    }
                    let gen_feeder = GEN_FEEDERS.iter().position(|n| *n == target);
                    if let Some(r) = gf
                        && let Some(site) = self.plant.net().shunt_site(&target)
                    {
                        if single_phase {
                            want.phase_shunts.insert(format!("{part}.gf"), (site, r));
                        } else {
                            want.shunts.insert(format!("{part}.gf"), (site, r));
                        }
                        if let Some(m) = gen_feeder {
                            if self.gcu_out[m].trip_latched || self.fault_link.tripped.contains(part) {
                                trips.push(part.clone());
                            }
                        } else if self.ground_pickup(&target) || self.fault_link.tripped.contains(part) {
                            trips.push(part.clone());
                            if self.gear.iter().any(|g| g.name == target) {
                                open_gears.push(target.clone());
                            } else {
                                open_sspcs.push(target.clone());
                            }
                        }
                    }
                }
                _ => {}
            }
        }
        for (k, p) in self.atru_prot.iter().enumerate() {
            if p.shut_down() {
                let name = crate::protection::atru_prot::ATRU_NAMES[k];
                want.conv.entry(name.to_string()).or_insert((1.0, 1.0, true)).2 = true;
                if k == 1 {
                    want.conv
                        .entry("ATRU_L2_115".to_string())
                        .or_insert((1.0, 1.0, true))
                        .2 = true;
                }
            }
        }
        for (k, oc) in self.tru_oc.iter().enumerate() {
            if oc.tripped() {
                let name = crate::protection::tru_oc::TRU_NAMES[k].to_string();
                want.conv.entry(name).or_insert((1.0, 1.0, true)).2 = true;
            }
        }
        for p in trips {
            self.fault_link.tripped.insert(p);
        }
        for g in open_gears {
            self.force_gear(&g, true, false, false);
        }
        for g in weld_gears {
            self.force_gear(&g, false, true, false);
        }
        for s in open_sspcs {
            self.force_sspc(&s, false);
            sspc_live.insert(s);
        }
        for s in closed_sspcs {
            if !sspc_live.contains(&s) {
                self.force_sspc(&s, true);
                sspc_live.insert(s);
            }
        }
        self.apply_net_state(want);
    }

    fn ground_pickup(&self, target: &str) -> bool {
        self.plant.contactor_a(target) > GROUND_TRIP_CONTACTOR_A
    }

    fn apply_net_state(&mut self, want: NetState) {
        let had = std::mem::take(&mut self.fault_link.applied);
        if had == want {
            self.fault_link.applied = want;
            return;
        }
        for (k, v) in &want.shunts {
            if had.shunts.get(k) != Some(v) {
                self.plant.net_mut().set_shunt(k, &v.0, Some(v.1));
            }
        }
        for (k, v) in &had.shunts {
            if !want.shunts.contains_key(k) {
                self.plant.net_mut().set_shunt(k, &v.0, None);
            }
        }
        for (k, v) in &want.phase_shunts {
            if had.phase_shunts.get(k) != Some(v) {
                self.plant.net_mut().set_phase_shunt(k, &v.0, Some(v.1));
            }
        }
        for (k, v) in &had.phase_shunts {
            if !want.phase_shunts.contains_key(k) {
                self.plant.net_mut().set_phase_shunt(k, &v.0, None);
            }
        }
        for (k, v) in &want.load_scale {
            if had.load_scale.get(k) != Some(v) {
                let base = match self.fault_link.load_base.get(k) {
                    Some(b) => *b,
                    None => {
                        let b = self.plant.net().load_demand_w(k).unwrap_or(0.0);
                        self.fault_link.load_base.insert(k.clone(), b);
                        b
                    }
                };
                self.plant.net_mut().set_load(k, base * v);
            }
        }
        for k in had.load_scale.keys() {
            if !want.load_scale.contains_key(k)
                && let Some(b) = self.fault_link.load_base.remove(k)
            {
                self.plant.net_mut().set_load(k, b);
            }
        }
        for (k, v) in &want.conv {
            if had.conv.get(k) != Some(v) {
                self.plant.net_mut().set_converter_mod(k, v.0, v.1, v.2);
            }
        }
        for k in had.conv.keys() {
            if !want.conv.contains_key(k) {
                self.plant.net_mut().set_converter_mod(k, 1.0, 1.0, false);
            }
        }
        for (k, v) in &want.series {
            if had.series.get(k) != Some(v) {
                self.plant.net_mut().set_rpdu_feed_series_ohm(k, *v);
            }
        }
        for k in had.series.keys() {
            if !want.series.contains_key(k) {
                self.plant.net_mut().set_rpdu_feed_series_ohm(k, 0.0);
            }
        }
        for (k, v) in &want.diode {
            if had.diode.get(k) != Some(v) {
                self.plant.net_mut().set_battery_diode_force(k, v.0);
                self.plant.net_mut().set_battery_diode_drop_ohm(k, v.1);
            }
        }
        for k in had.diode.keys() {
            if !want.diode.contains_key(k) {
                self.plant.net_mut().set_battery_diode_force(k, None);
                self.plant.net_mut().set_battery_diode_drop_ohm(k, 0.0);
            }
        }
        for r in want.comms.difference(&had.comms) {
            self.pdu.set_comms_loss(r, true);
        }
        for r in had.comms.difference(&want.comms) {
            self.pdu.set_comms_loss(r, false);
        }
        for r in want.ctrl_power.difference(&had.ctrl_power) {
            self.pdu.set_controller_power_loss(r, true);
        }
        for r in had.ctrl_power.difference(&want.ctrl_power) {
            self.pdu.set_controller_power_loss(r, false);
        }
        self.fault_link.applied = want;
    }

    pub(crate) fn causal_panel(
        &mut self,
        mut out: PanelOutputs,
        mut lamps: PanelLamps,
    ) -> (PanelOutputs, PanelLamps) {
        let now = self.now_us;
        let sw = self.active_parts_of(faults::PANEL_SWITCH);
        let lp = self.active_parts_of(faults::LAMP);
        let mut held: BTreeMap<String, bool> = BTreeMap::new();
        for part in sw {
            let name = part.strip_prefix("SW:").unwrap_or(&part).to_string();
            let m = |mode: &str| self.fault_link.book.magnitude(&part, mode, now);
            let (stuck, open, bounce, light) = (
                on(m("stuck")),
                on(m("open-contact")),
                on(m("contact-bounce")),
                on(m("light-element-open")),
            );
            if let Some(v) = switch_mut(&mut out, &name) {
                if stuck {
                    let h = *self.fault_link.panel_hold.entry(part.clone()).or_insert(*v);
                    held.insert(part.clone(), h);
                    *v = h;
                } else if open {
                    *v = false;
                } else if bounce {
                    *v = *v && (now / BOUNCE_US).is_multiple_of(2);
                }
            }
            if light && let Some(l) = lamp_mut(&mut lamps, &name) {
                dark(l);
            }
        }
        self.fault_link.panel_hold = held;
        for part in lp {
            let name = part.strip_prefix("LAMP:").unwrap_or(&part).to_string();
            let book = &self.fault_link.book;
            let (open, dim, short) = (
                on(book.magnitude(&part, "filament-open", now)),
                book.magnitude(&part, "dim", now),
                on(book.magnitude(&part, "short", now)),
            );
            if let Some(l) = lamp_mut(&mut lamps, &name) {
                if open || short {
                    dark(l);
                } else {
                    l.brightness *= 1.0 - dim;
                }
            }
        }
        (out, lamps)
    }

    pub(crate) fn causal_crew_inputs(&self, c: &mut CrewInputs) {
        let now = self.now_us;
        let book = &self.fault_link.book;
        for part in self.active_parts_of(faults::EICAS_INPUT) {
            let e = book.indication(&part, now);
            let flicker = on(book.magnitude(&part, "intermittent", now)) && (now / FLICKER_US) % 2 == 1;
            let f = |v: f64, nominal: f64| -> f64 {
                if e.forced_low || flicker {
                    0.0
                } else if e.forced_high {
                    nominal * 1.5
                } else {
                    v * (1.0 + e.bias)
                }
            };
            let tag = part.strip_prefix("EICASIN:").unwrap_or(&part);
            let bus = ["ACBUS_L1", "ACBUS_L2", "ACBUS_R1", "ACBUS_R2"]
                .iter()
                .position(|n| *n == tag);
            let bus = if tag == "ELEC" { Some(0) } else { bus };
            if let Some(k) = bus {
                c.ac_bus_volts[k] = f(c.ac_bus_volts[k], 235.0);
                c.ac_bus_energized[k] = c.ac_bus_volts[k] > BUS_LIVE_V;
                continue;
            }
            match tag {
                "BATT_MAIN" => c.main_batt_volts = f(c.main_batt_volts, 28.0),
                "DC28_L" => c.dc28_volts[0] = f(c.dc28_volts[0], 28.0),
                "DC28_R" => c.dc28_volts[1] = f(c.dc28_volts[1], 28.0),
                "AC115_L" => c.ac115_volts[0] = f(c.ac115_volts[0], 115.0),
                "AC115_R" => c.ac115_volts[1] = f(c.ac115_volts[1], 115.0),
                "BACKUP" => c.backup_volts = f(c.backup_volts, 28.0),
                _ => {}
            }
        }
        for part in self.active_parts_of(faults::CMSC) {
            let bad = ["internal-fault", "can-loss", "cooling-loss", "motor-select-stuck"]
                .iter()
                .any(|m| on(book.magnitude(&part, m, now)));
            if !bad {
                continue;
            }
            if let Some(i) = ["CMSC_CAC_L1", "CMSC_CAC_L2", "CMSC_CAC_R1", "CMSC_CAC_R2"]
                .iter()
                .position(|n| *n == part)
            {
                c.mc_cac_inop[i] = true;
            }
            if let Some(i) = ["CMSC_EMP_L", "CMSC_EMP_R", "CMSC_EMP_C1", "CMSC_EMP_C2"]
                .iter()
                .position(|n| *n == part)
            {
                c.emp_mc_inop[i] = true;
            }
        }
        for part in self.active_parts_of(faults::SPU) {
            if on(book.magnitude(&part, "internal-fault", now)) || on(book.magnitude(&part, "short", now)) {
                c.spu_main_fail = true;
            }
            if on(book.magnitude(&part, "housekeeping-fault", now)) {
                c.spu_housekeeping_fail = true;
            }
        }
        for part in self.active_parts_of(faults::OIL_AIR_VALVE) {
            let bad = ["position-sensor-fail", "locked-closed", "locked-open"]
                .iter()
                .any(|m| on(book.magnitude(&part, m, now)));
            if bad {
                let k = usize::from(part.contains('R'));
                c.oil_cooler_valve_fault[k] = true;
            }
        }
        for part in self.active_parts_of(faults::BMU) {
            let k = usize::from(part == "BMU_APU");
            let bad = on(book.magnitude(&part, "bmu-fail", now)) || self.bmu_out[k].bmu_fail;
            if k == 0 {
                c.main_batt_data_invalid |= bad;
            } else {
                c.apu_batt_data_invalid |= bad;
            }
        }
        if self.charger_out[0].failed {
            c.main_batt_charger_fail = true;
        }
        if self.charger_out[1].failed {
            c.apu_batt_charger_fail = true;
        }
    }

    pub(crate) fn ext_receptacle_index(part: &str) -> Option<usize> {
        part.strip_prefix("EXTRCPT_")?
            .parse::<usize>()
            .ok()?
            .checked_sub(1)
    }

    pub(crate) fn causal_ext_pin_ef(&self, k: usize, ok: bool) -> bool {
        let mut v = ok;
        for part in self.active_parts_of(faults::EXT_POWER_RECEPTACLE) {
            if Self::ext_receptacle_index(&part) != Some(k) {
                continue;
            }
            if on(self.mag(&part, "pin-ef-open")) {
                v = false;
            }
            if on(self.mag(&part, "chatter")) && (self.now_us / CHATTER_US) % 2 == 1 {
                v = false;
            }
        }
        v
    }

    pub(crate) fn causal_ext_phase_v(&self, k: usize, v: f64) -> [f64; 3] {
        let mut out = [v; 3];
        for part in self.active_parts_of(faults::EXT_POWER_RECEPTACLE) {
            if Self::ext_receptacle_index(&part) == Some(k) {
                out[0] *= 1.0 - 0.15 * self.mag(&part, "contact-high-resistance");
            }
        }
        out
    }

    pub(crate) fn causal_bmu(&self, b: usize) -> (bool, f64, f64) {
        let id = if b == 0 { "BMU_MAIN" } else { "BMU_APU" };
        let pack = if b == 0 { "BATT_MAIN" } else { "BATT_APU" };
        let fail = on(self.mag(id, "bmu-fail")) || on(self.mag(id, "balance-circuit-stuck"));
        let volt_off = 0.15 * self.mag(id, "voltage-sense-offset");
        let temp_bias = 15.0 * self.mag(pack, "temp-sensor-bias");
        (fail, volt_off, temp_bias)
    }

    pub(crate) fn causal_charger(&self, b: usize) -> (bool, bool, f64) {
        let id = if b == 0 { "CHG_MAIN" } else { "CHG_APU" };
        let fault = on(self.mag(id, "internal-fault"));
        let inhibit = on(self.mag(id, "overtemp")) || on(self.mag(id, "cold-lockout"));
        let scale = 1.0 + 0.5 * self.mag(id, "output-drift");
        (fault, inhibit, scale)
    }
}
