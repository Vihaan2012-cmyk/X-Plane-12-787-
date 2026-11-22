use crate::cabinet::{Cabinet, CabinetInput, CabinetState, GPM_COUNT, Side, StartMode};
use crate::cdn::Cdn;
use crate::data::{CcsData, RdcDef};
use crate::gpm::GpmSoftware;
use crate::icd::{Dataset, Encoding, FS_NORMAL};
use crate::network;
use crate::pcm::PCM_SWITCH_V;
use std::collections::BTreeMap;
use std::collections::BTreeSet;
use std::path::Path;

pub const NETWORK_BOOT_MAX_US: u64 = 45_000_000;
pub const RDC_IFZ: usize = 14;
pub const RDC_CAN_IFZ: usize = 10;
pub const RDC_ANALOG_IFZ: usize = 4;
pub const APPROX_NETWORK_BOOT_US: u64 = 30_000_000;
pub const APPROX_RDC_W: f64 = 20.0;
pub const PRIMARY_BUSES: [[&str; 2]; 2] = [["CIB", "FIB"], ["DC28_R", "DC28_L"]];
pub const BATTERY_BUS: &str = "HBB_MAIN";
pub const LOAD_NAMES: [[&str; 3]; 2] = [
    ["CABINET CCR L PRI", "CABINET CCR L SEC", "CABINET CCR L BAT"],
    [
        "CB CE4221821 CABINET CCR R PRI",
        "CB CE4221822 CABINET CCR R SEC",
        "CABINET CCR R BAT",
    ],
];
pub const LPS_APP: &str = "LOW PRESSURE SYSTEM";
pub const IFZ_NAMES: [&str; RDC_IFZ] = [
    "ifz1", "ifz2", "ifz3", "ifz4", "ifz5", "ifz6", "ifz7", "ifz8", "ifz9", "ifz10", "ifz11", "ifz12",
    "ifz13", "ifz14",
];
pub const APPROX_STALE_BAGS: u64 = 3;
pub const APPROX_STALE_MARGIN_US: u64 = 2_000;
pub const DAYS_51_US: u64 = 51 * 86_400 * 1_000_000;

#[derive(Clone, Debug, PartialEq)]
pub enum CcsValue {
    Bool(bool),
    Num(f64),
    Text(String),
}

#[derive(Clone, Debug, PartialEq)]
pub struct CcsInputs {
    pub bus_v: BTreeMap<String, f64>,
    pub on_ground: bool,
    pub fuel_run: [bool; 2],
    pub fuel_data_valid: bool,
    pub cooling_supply_ok: bool,
    pub cooling_air_c: f64,
    pub ambient_c: f64,
}

impl Default for CcsInputs {
    fn default() -> Self {
        CcsInputs {
            bus_v: BTreeMap::new(),
            on_ground: true,
            fuel_run: [false; 2],
            fuel_data_valid: true,
            cooling_supply_ok: true,
            cooling_air_c: 20.0,
            ambient_c: 25.0,
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct Rdc {
    pub def: RdcDef,
    pub powered_since: Option<u64>,
    pub failed: bool,
    pub on_secondary: bool,
    pub ifz_failed: [bool; RDC_IFZ],
}

impl Rdc {
    pub fn running(&self, now_us: u64) -> bool {
        !self.failed
            && self
                .powered_since
                .is_some_and(|t| now_us.saturating_sub(t) >= APPROX_NETWORK_BOOT_US)
    }
}

#[derive(Clone, Debug)]
pub struct Ccs {
    pub now_us: u64,
    pub cabinets: [Cabinet; 2],
    pub rdcs: Vec<Rdc>,
    pub hosted: BTreeMap<String, Vec<String>>,
    pub inputs: CcsInputs,
    resets: [u32; 2],
    pub cdn: Cdn,
    pub powered_us: u64,
    ars_since: Option<u64>,
    pub bpcu_payload: [Option<Vec<u8>>; 2],
    heartbeat: Dataset,
    pub rdc_ds: Dataset,
    pub icd_mismatch: [bool; 2],
    pub gnd_test_sel: u8,
    pub gnd_test_eff: u8,
    pub(crate) gnd_test_since: u64,
    pub sw: Vec<GpmSoftware>,
    pub part_faults: BTreeSet<(String, String)>,
    pub elec_icd: Option<Dataset>,
    pub ccshm_lost_since: Option<u64>,
    pub(crate) last_boots: [u32; 2],
}

fn slug(s: &str) -> String {
    s.to_ascii_lowercase()
        .chars()
        .map(|c| if c.is_ascii_alphanumeric() { c } else { '_' })
        .collect::<String>()
        .split('_')
        .filter(|p| !p.is_empty())
        .collect::<Vec<_>>()
        .join("_")
}

impl Ccs {
    pub fn new(data_dir: &Path, seed: u64) -> Result<Ccs, String> {
        let data = CcsData::load(data_dir)?;
        Ok(Ccs::from_data(data, seed))
    }

    pub fn from_data(data: CcsData, seed: u64) -> Ccs {
        let inputs = CcsInputs::default();
        let cdn = network::build(&data.rdcs, &data.hosted, seed);
        let sw = network::GPMS
            .iter()
            .map(|g| GpmSoftware::new(g, &data.hosted))
            .collect();
        Ccs {
            now_us: 0,
            cabinets: [
                Cabinet::new(Side::L, seed, inputs.ambient_c),
                Cabinet::new(Side::R, seed, inputs.ambient_c),
            ],
            rdcs: data
                .rdcs
                .into_iter()
                .map(|def| Rdc {
                    def,
                    powered_since: None,
                    failed: false,
                    on_secondary: false,
                    ifz_failed: [false; RDC_IFZ],
                })
                .collect(),
            hosted: data.hosted,
            inputs,
            resets: [0; 2],
            cdn,
            powered_us: 0,
            ars_since: None,
            bpcu_payload: [None, None],
            icd_mismatch: [false; 2],
            gnd_test_sel: 0,
            gnd_test_eff: 0,
            gnd_test_since: 0,
            rdc_ds: Dataset::new(
                "RDC DATA",
                1,
                &(1..=RDC_IFZ)
                    .map(|k| (IFZ_NAMES[k - 1], Encoding::Discrete))
                    .chain([("counter", Encoding::Raw { bits: 16 })])
                    .collect::<Vec<_>>(),
            ),
            heartbeat: Dataset::new(
                "HEARTBEAT",
                1,
                &[
                    ("source_ok", Encoding::Discrete),
                    ("counter", Encoding::Raw { bits: 16 }),
                ],
            ),
            sw,
            part_faults: BTreeSet::new(),
            elec_icd: None,
            ccshm_lost_since: None,
            last_boots: [0; 2],
        }
    }

    pub fn monitor_lost(&self) -> bool {
        self.powered_us >= DAYS_51_US
    }

    pub fn set_powered_days(&mut self, days: f64) {
        self.powered_us = (days.max(0.0) * 86_400.0e6) as u64;
    }

    pub fn set_device_fault(&mut self, name: &str, mode: &str, on: bool) -> bool {
        let Some(d) = self.cdn.device(name) else {
            return false;
        };
        let dev = &mut self.cdn.devices[d];
        match mode {
            "failed" => dev.failed = on,
            "replay" => dev.replay = on,
            "babbling" => dev.babbling = on,
            "corrupt" => dev.corrupt_before_crc = on,
            _ => return false,
        }
        true
    }

    pub fn set_link_fault(&mut self, a: &str, b: &str, ber: f64, open: bool) -> bool {
        let (Some(da), Some(db)) = (self.cdn.device(a), self.cdn.device(b)) else {
            return false;
        };
        let mut hit = false;
        for l in self.cdn.links.iter_mut() {
            if (l.a == da && l.b == db) || (l.a == db && l.b == da) {
                l.ber = ber;
                l.failed = open;
                hit = true;
            }
        }
        hit
    }

    pub fn read(&self, dest: &str, vl: &str) -> Option<(Vec<u8>, u64, bool)> {
        let d = self.cdn.device(dest)?;
        let id = self.cdn.vl_by_name(vl)?;
        let bag = self.cdn.vls[id as usize - 1].bag_us;
        let r = self.cdn.rx(d, id)?;
        let rx = r.rx_us?;
        let now_ms = (self.now_us / 1_000) as u32;
        let age = match Dataset::timestamp_of(&r.data) {
            Some(ts) if r.data.len() >= crate::icd::STATUS_BYTES => {
                u64::from(now_ms.wrapping_sub(ts)) * 1_000
            }
            _ => self.now_us.saturating_sub(rx),
        };
        let fresh = age <= bag * APPROX_STALE_BAGS + APPROX_STALE_MARGIN_US;
        Some((r.data.clone(), age, fresh || self.monitor_lost()))
    }

    fn device_up(&self, name: &str, instr: bool, main_dc: bool) -> bool {
        let now = self.now_us;
        let cab = |side: &str| {
            if side.starts_with('L') {
                &self.cabinets[0]
            } else {
                &self.cabinets[1]
            }
        };
        let parts: Vec<&str> = name.split(' ').collect();
        match parts.as_slice() {
            ["ACS", side, _] | ["FOX", side, _] => {
                let c = cab(side);
                c.powered()
                    && (c.hosted_ready()
                        || c.boot_elapsed_us(now)
                            .is_some_and(|e| e >= APPROX_NETWORK_BOOT_US))
            }
            ["ARS", _] => self
                .ars_since
                .is_some_and(|t| now.saturating_sub(t) >= APPROX_NETWORK_BOOT_US),
            ["GPM", g] => self.gpm_running(g),
            ["GG", g] => cab(g).displays_ready(now),
            ["RDC", _] => self.rdcs.iter().any(|r| r.def.id == name && r.running(now)),
            ["P300"] | ["P400"] => instr,
            ["RPDU", _] => instr || main_dc,
            _ => false,
        }
    }

    fn step_network(&mut self, dt_us: u64) {
        let now = self.now_us;
        if self.cabinets.iter().any(|c| c.powered()) {
            self.powered_us = self.powered_us.saturating_add(dt_us);
        } else {
            self.powered_us = 0;
        }
        let instr = self.v("CIB") >= PCM_SWITCH_V || self.v("FIB") >= PCM_SWITCH_V;
        let main_dc = self.v("DC28_L") >= PCM_SWITCH_V || self.v("DC28_R") >= PCM_SWITCH_V;
        if instr {
            self.ars_since.get_or_insert(now);
        } else {
            self.ars_since = None;
        }
        let names: Vec<String> = self.cdn.devices.iter().map(|d| d.name.clone()).collect();
        let ups: Vec<bool> = names.iter().map(|n| self.device_up(n, instr, main_dc)).collect();
        for (dev, up) in self.cdn.devices.iter_mut().zip(ups) {
            dev.powered = up;
        }
        let counter = ((now / 1_000) % 65_536) as f64;
        let beat = self
            .heartbeat
            .encode(&[("counter", counter), ("source_ok", 1.0)], FS_NORMAL);
        let ids: Vec<(u16, String)> = self.cdn.vls.iter().map(|v| (v.id, v.name.clone())).collect();
        for (id, name) in ids {
            if let Some((app_slug, gpm)) = name.split_once(" @ ") {
                let live = self
                    .hosted
                    .keys()
                    .find(|a| network::slug(a) == app_slug)
                    .is_some_and(|a| self.instance_live(a, gpm));
                if !live {
                    self.cdn.unpublish(id);
                    continue;
                }
            }
            if let Some(g) = name.strip_prefix("TIME ")
                && !self.instance_live("TIME MANAGER", g)
            {
                self.cdn.unpublish(id);
                continue;
            }
            let rdc_payload = name.strip_suffix(" DATA").and_then(|rid| {
                self.rdcs.iter().find(|r| r.def.id == rid).map(|r| {
                    let mut vals: Vec<(&str, f64)> = (0..RDC_IFZ)
                        .map(|k| (IFZ_NAMES[k], if r.ifz_failed[k] { 0.0 } else { 1.0 }))
                        .collect();
                    vals.push(("counter", counter));
                    self.rdc_ds.encode(&vals, FS_NORMAL)
                })
            });
            let bpcu = |k: usize| {
                let mut p = self.bpcu_payload[k].clone().unwrap_or_else(|| beat.clone());
                if self.icd_mismatch[k] && p.len() > 2 {
                    p[1] ^= 0xFF;
                }
                p
            };
            let payload = match name.as_str() {
                "BPCU P300" => bpcu(0),
                "BPCU P400" => bpcu(1),
                _ => rdc_payload.unwrap_or_else(|| beat.clone()),
            };
            let mut payload = payload;
            Dataset::stamp(&mut payload, (now / 1_000) as u32);
            self.cdn.publish(id, payload);
        }
        self.cdn.run_until(now);
    }

    fn v(&self, bus: &str) -> f64 {
        self.inputs.bus_v.get(bus).copied().unwrap_or(0.0)
    }

    pub fn press_reset(&mut self, side: usize) {
        let now = self.now_us;
        self.cabinets[side.min(1)].press_reset(now);
        self.resets[side.min(1)] += 1;
    }

    pub fn gpm_running(&self, name: &str) -> bool {
        let (c, n) = match name.split_at(1) {
            ("L", n) => (0, n),
            ("R", n) => (1, n),
            _ => return false,
        };
        n.parse::<usize>()
            .ok()
            .filter(|k| (1..=GPM_COUNT).contains(k))
            .is_some_and(|k| {
                self.cabinets[c].gpm_running(k - 1) && self.sw[c * GPM_COUNT + k - 1].hardware_ok()
            })
    }

    pub fn sw_of(&self, gpm: &str) -> Option<&GpmSoftware> {
        self.sw.iter().find(|s| s.name == gpm)
    }

    pub fn instance_live(&self, app: &str, gpm: &str) -> bool {
        self.gpm_running(gpm) && self.sw_of(gpm).is_some_and(|s| s.running(app))
    }

    pub fn app_available(&self, app: &str) -> bool {
        self.hosted
            .get(app)
            .is_some_and(|gpms| gpms.iter().any(|g| self.instance_live(app, g)))
    }

    pub fn set_gpm_failed(&mut self, name: &str, failed: bool) -> bool {
        let c = match name.chars().next() {
            Some('L') => 0,
            Some('R') => 1,
            _ => return false,
        };
        match name[1..].parse::<usize>() {
            Ok(k) if (1..=GPM_COUNT).contains(&k) => {
                self.cabinets[c].gpm[k - 1].failed = failed;
                true
            }
            _ => false,
        }
    }

    pub fn set_rdc_failed(&mut self, id: &str, failed: bool) -> bool {
        match self.rdcs.iter_mut().find(|r| r.def.id == id) {
            Some(r) => {
                r.failed = failed;
                true
            }
            None => false,
        }
    }

    pub fn inhibit(&self) -> bool {
        !self.inputs.on_ground || self.inputs.fuel_run.iter().any(|b| *b) || !self.inputs.fuel_data_valid
    }

    pub fn step(&mut self, dt_us: u64, inputs: &CcsInputs) {
        self.now_us += dt_us;
        self.inputs = inputs.clone();
        let now = self.now_us;
        let inhibit = self.inhibit();
        let battery_v = self.v(BATTERY_BUS);
        for (c, buses) in PRIMARY_BUSES.iter().enumerate() {
            let lps = self.app_available(LPS_APP);
            let input = CabinetInput {
                now_us: now,
                dt_us,
                primary_v: [self.v(buses[0]), self.v(buses[1])],
                battery_v,
                on_ground: inputs.on_ground,
                inhibit,
                cooling_supply_ok: inputs.cooling_supply_ok,
                cooling_air_c: inputs.cooling_air_c,
                ambient_c: inputs.ambient_c,
                lps_available: lps,
            };
            self.cabinets[c].step(&input);
        }
        for k in 0..self.rdcs.len() {
            let pri = self.v(&self.rdcs[k].def.pri_bus.clone()) >= PCM_SWITCH_V;
            let sec = self.v(&self.rdcs[k].def.sec_bus.clone()) >= PCM_SWITCH_V;
            let r = &mut self.rdcs[k];
            r.on_secondary = !pri && sec;
            if pri || sec {
                r.powered_since.get_or_insert(now);
            } else {
                r.powered_since = None;
            }
        }
        self.step_software();
        self.step_network(dt_us);
    }

    pub fn loads_w(&self) -> Vec<(&'static str, f64)> {
        let mut out = Vec::new();
        for (c, cab) in self.cabinets.iter().enumerate() {
            let on: Vec<usize> = (0..2).filter(|k| cab.pcm[*k].on).collect();
            let share = if on.is_empty() {
                0.0
            } else {
                cab.power_w() / on.len() as f64
            };
            let mut w = [0.0; 3];
            for k in on {
                if cab.pcm[k].on_battery() {
                    w[2] += share;
                } else {
                    w[k] += share;
                }
            }
            for (j, name) in LOAD_NAMES[c].iter().enumerate() {
                out.push((*name, w[j]));
            }
        }
        out
    }

    pub fn status_messages(&self) -> Vec<String> {
        let mut m = Vec::new();
        for cab in &self.cabinets {
            if cab.fan_failed || cab.valve_failed {
                m.push(format!("CCR CABINET COOLING {}", cab.letter()));
            }
        }
        if self.icd_mismatch.iter().any(|b| *b) {
            m.push("CCS CONFIGURATION".into());
        }
        let instr_live = self.v("CIB") >= PCM_SWITCH_V || self.v("FIB") >= PCM_SWITCH_V;
        if instr_live && self.rdcs.iter().any(|r| r.failed) {
            m.push("CCS RDC".into());
        }
        m
    }

    pub fn value(&self, name: &str) -> Option<CcsValue> {
        use CcsValue::*;
        let parts: Vec<&str> = name.strip_prefix("ccs.").unwrap_or(name).split('.').collect();
        let side = |s: &str| match s {
            "l" | "L" => Some(0usize),
            "r" | "R" => Some(1usize),
            _ => None,
        };
        match parts.as_slice() {
            ["ccr", s, what] => {
                let c = &self.cabinets[side(s)?];
                Some(match *what {
                    "state" => Text(
                        match c.state {
                            CabinetState::Off => "off",
                            CabinetState::Booting { .. } => "booting",
                            CabinetState::Operational { .. } => "operational",
                        }
                        .into(),
                    ),
                    "mode" => Text(
                        match c.mode() {
                            None => "none",
                            Some(StartMode::Uninhibited) => "uninhibited",
                            Some(StartMode::Inhibited) => "inhibited",
                        }
                        .into(),
                    ),
                    "powered" => Bool(c.powered()),
                    "on_battery" => Bool(c.on_battery()),
                    "hosted_ready" => Bool(c.hosted_ready()),
                    "displays_ready" => Bool(c.displays_ready(self.now_us)),
                    "temp_c" => Num(c.temp_c),
                    "fan_on" => Bool(c.fan_on),
                    "valve_closed" => Bool(c.valve_closed),
                    "power_w" => Num(c.power_w()),
                    "boots" => Num(c.boots as f64),
                    "nvm_saves" => Num(c.nvm_saves as f64),
                    "hard_offs" => Num(c.hard_offs as f64),
                    "startup_complete" => Bool(c.startup_events > 0),
                    "resets" => Num(self.resets[side(s)?] as f64),
                    _ => return None,
                })
            }
            ["gpm", g, "running"] => Some(Bool(self.gpm_running(&g.to_ascii_uppercase()))),
            ["app", a, "available"] => {
                let app = self.hosted.keys().find(|k| slug(k) == *a)?;
                Some(Bool(self.app_available(app)))
            }
            ["rdc", n, what] => {
                let r = self.rdcs.iter().find(|r| r.def.id == format!("RDC {n}"))?;
                Some(match *what {
                    "running" => Bool(r.running(self.now_us)),
                    "powered" => Bool(r.powered_since.is_some()),
                    "on_secondary" => Bool(r.on_secondary),
                    _ => return None,
                })
            }
            ["rdc", n, "ifz", k, "valid"] => {
                let k: usize = k.parse().ok()?;
                let name = IFZ_NAMES.get(k.checked_sub(1)?)?;
                let got = self
                    .read("GPM L1", &format!("RDC {n} DATA"))
                    .or_else(|| self.read("GPM R1", &format!("RDC {n} DATA")));
                Some(Bool(got.is_some_and(|(d, _, valid)| {
                    valid && self.rdc_ds.decode(&d, name) == Some(1.0)
                })))
            }
            ["param", p] => Some(Num(crate::params::param(p)?)),
            ["rdc", "running_count"] => Some(Num(
                self.rdcs.iter().filter(|r| r.running(self.now_us)).count() as f64,
            )),
            ["cdn", what] => Some(Num(match *what {
                "delivered" => self.cdn.delivered as f64,
                "police_drops" => self.cdn.police_drops() as f64,
                "queue_drops" => self.cdn.queue_drops() as f64,
                "crc_errors" => self.cdn.links.iter().map(|l| l.crc_errors).sum::<u64>() as f64,
                "unknown_vl_drops" => self.cdn.unknown_vl_drops as f64,
                "vl_count" => self.cdn.vls.len() as f64,
                _ => return None,
            })),
            ["switch", sw, "up"] => {
                let name = sw.replace('_', " ").to_ascii_uppercase();
                let d = self.cdn.device(&name)?;
                Some(Bool(self.cdn.devices[d].powered && !self.cdn.devices[d].failed))
            }
            ["data", vl, dest, what] => {
                let vlname = self
                    .cdn
                    .vls
                    .iter()
                    .find(|v| network::slug(&v.name) == *vl)?
                    .name
                    .clone();
                let dest = dest.replace('_', " ").to_ascii_uppercase();
                let got = self.read(&dest, &vlname);
                let (age, valid) = got.map(|(_, a, v)| (a, v)).unwrap_or((u64::MAX, false));
                Some(match *what {
                    "valid" => Bool(valid),
                    "age_ms" => Num(if age == u64::MAX {
                        f64::INFINITY
                    } else {
                        age as f64 / 1000.0
                    }),
                    _ => return None,
                })
            }
            ["monitor_lost"] => Some(Bool(self.monitor_lost())),
            ["gnd_test"] => Some(Text(
                ["NORM", "ENABLE", "DATA LOAD/ENABLE"][self.gnd_test_eff.min(2) as usize].into(),
            )),
            ["gpm", g, "data_load"] => Some(Bool(self.sw_of(&g.to_ascii_uppercase())?.data_load)),
            ["gpm", g, "partition", part, "running"] => {
                let gpm = g.to_ascii_uppercase();
                let sw = self.sw_of(&gpm)?;
                let pname = sw
                    .partitions
                    .iter()
                    .find(|p| network::slug(&p.name) == *part)?
                    .name
                    .clone();
                Some(Bool(self.instance_live(&pname, &gpm)))
            }
            ["gpm", g, "single_lane"] => Some(Bool(self.sw_of(&g.to_ascii_uppercase())?.single_lane())),
            ["cas", "available"] => Some(Bool(self.cas_live_host().is_some())),
            ["cas", "elec_data_valid"] => Some(Bool(self.cas_elec_messages().is_some_and(|(_, v)| v))),
            ["cas", "msg", m] => {
                Some(Bool(self.cas_elec_messages().is_some_and(|(msgs, _)| {
                    msgs.iter().any(|x| network::slug(x) == *m)
                })))
            }
            ["cas", "count"] => Some(Num(
                self.cas_elec_messages().map(|(m, _)| m.len()).unwrap_or(0) as f64
            )),
            ["crew_status", m] => Some(Bool(
                self.crew_status_messages().iter().any(|x| network::slug(x) == *m),
            )),
            ["maint", page, item] => {
                let n: u8 = page.strip_prefix("page")?.parse().ok()?;
                let rows = self.maint_page(n);
                let (_, st) = rows.into_iter().find(|(k, _)| network::slug(k) == *item)?;
                Some(Text(st))
            }
            ["powered_days"] => Some(Num(self.powered_us as f64 / 86_400.0e6)),
            ["status", msg] => {
                let want = msg.replace('_', " ").to_ascii_uppercase();
                Some(Bool(self.status_messages().contains(&want)))
            }
            _ => None,
        }
    }
}
