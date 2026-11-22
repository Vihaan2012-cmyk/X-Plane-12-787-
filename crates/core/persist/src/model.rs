use crate::document::{Document, Section};
use crate::error::PersistError;
use crate::value::Value;
use std::collections::BTreeMap;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum GapMode {
    Sim,
    Real,
}

impl GapMode {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Sim => "sim",
            Self::Real => "real",
        }
    }

    pub fn parse(s: &str) -> Option<Self> {
        match s {
            "sim" => Some(Self::Sim),
            "real" => Some(Self::Real),
            _ => None,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SessionClock {
    pub sim_s: f64,
    pub real_s: u64,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Identity {
    pub tail: String,
    pub serial: String,
    pub model: String,
    pub options: BTreeMap<String, String>,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Settings {
    pub gap_mode: GapMode,
    pub max_gap_s: f64,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            gap_mode: GapMode::Sim,
            max_gap_s: 10.0 * 365.25 * 86400.0,
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct Counter {
    pub value: f64,
    pub powered_only: bool,
}

#[derive(Clone, Debug, PartialEq)]
pub struct FaultEntry {
    pub t_s: f64,
    pub code: String,
    pub flags: u64,
}

#[derive(Clone, Debug, PartialEq)]
pub struct PartRecord {
    pub wear: f64,
    pub run_s: f64,
    pub cycles: u64,
    pub powered_s: f64,
    pub unpowered_s: f64,
    pub soak_c: f64,
    pub soak_tau_s: f64,
    pub nvm: BTreeMap<String, Vec<u8>>,
    pub counters: BTreeMap<String, Counter>,
    pub faults: Vec<FaultEntry>,
}

impl PartRecord {
    pub fn new(soak_c: f64, soak_tau_s: f64) -> Self {
        Self {
            wear: 0.0,
            run_s: 0.0,
            cycles: 0,
            powered_s: 0.0,
            unpowered_s: 0.0,
            soak_c,
            soak_tau_s,
            nvm: BTreeMap::new(),
            counters: BTreeMap::new(),
            faults: Vec::new(),
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct BatteryRecord {
    pub soc: f64,
    pub cell_soc: Vec<f64>,
    pub cell_health: Vec<f64>,
    pub cycles: u64,
    pub soak_c: f64,
    pub soak_tau_s: f64,
    pub self_discharge_per_day: f64,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BreakerState {
    Open,
    Closed,
}

#[derive(Clone, Debug, PartialEq)]
pub struct BreakerRecord {
    pub state: BreakerState,
    pub locked: bool,
    pub tag: Option<String>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Airframe {
    pub identity: Identity,
    pub settings: Settings,
    pub stamp: Option<SessionClock>,
    pub parts: BTreeMap<String, PartRecord>,
    pub batteries: BTreeMap<String, BatteryRecord>,
    pub breakers: BTreeMap<String, BreakerRecord>,
    pub extra: BTreeMap<String, Section>,
}

impl Airframe {
    pub fn new(tail: &str, serial: &str) -> Self {
        Self {
            identity: Identity {
                tail: tail.to_string(),
                serial: serial.to_string(),
                model: "787-9".to_string(),
                options: BTreeMap::new(),
            },
            settings: Settings::default(),
            stamp: None,
            parts: BTreeMap::new(),
            batteries: BTreeMap::new(),
            breakers: BTreeMap::new(),
            extra: BTreeMap::new(),
        }
    }

    pub fn to_document(&self) -> Document {
        let mut doc = Document::current();
        let a = doc.section_mut("airframe");
        a.set("tail", Value::Str(self.identity.tail.clone()));
        a.set("serial", Value::Str(self.identity.serial.clone()));
        a.set("model", Value::Str(self.identity.model.clone()));
        for (k, v) in &self.identity.options {
            a.set(&format!("option.{k}"), Value::Str(v.clone()));
        }
        a.set(
            "settings.gap_mode",
            Value::Str(self.settings.gap_mode.as_str().to_string()),
        );
        a.set("settings.max_gap_s", Value::F64(self.settings.max_gap_s));
        if let Some(s) = self.stamp {
            a.set("stamp.sim_s", Value::F64(s.sim_s));
            a.set("stamp.real_s", Value::U64(s.real_s));
        }
        for (name, p) in &self.parts {
            let s = doc.section_mut(&format!("part/{name}"));
            s.set("wear", Value::F64(p.wear));
            s.set("run_s", Value::F64(p.run_s));
            s.set("cycles", Value::U64(p.cycles));
            s.set("powered_s", Value::F64(p.powered_s));
            s.set("unpowered_s", Value::F64(p.unpowered_s));
            s.set("soak_c", Value::F64(p.soak_c));
            s.set("soak_tau_s", Value::F64(p.soak_tau_s));
            for (k, b) in &p.nvm {
                s.set(&format!("nvm.{k}"), Value::Bytes(b.clone()));
            }
            for (k, c) in &p.counters {
                s.set(
                    &format!("counter.{k}"),
                    Value::Array(vec![Value::F64(c.value), Value::Bool(c.powered_only)]),
                );
            }
            let faults = p
                .faults
                .iter()
                .map(|f| {
                    Value::Array(vec![
                        Value::F64(f.t_s),
                        Value::Str(f.code.clone()),
                        Value::U64(f.flags),
                    ])
                })
                .collect();
            s.set("faults", Value::Array(faults));
        }
        for (name, b) in &self.batteries {
            let s = doc.section_mut(&format!("battery/{name}"));
            s.set("soc", Value::F64(b.soc));
            s.set(
                "cell_soc",
                Value::Array(b.cell_soc.iter().map(|x| Value::F64(*x)).collect()),
            );
            s.set(
                "cell_health",
                Value::Array(b.cell_health.iter().map(|x| Value::F64(*x)).collect()),
            );
            s.set("cycles", Value::U64(b.cycles));
            s.set("soak_c", Value::F64(b.soak_c));
            s.set("soak_tau_s", Value::F64(b.soak_tau_s));
            s.set("self_discharge_per_day", Value::F64(b.self_discharge_per_day));
        }
        if !self.breakers.is_empty() {
            let s = doc.section_mut("breakers");
            for (name, b) in &self.breakers {
                let state = match b.state {
                    BreakerState::Open => 0,
                    BreakerState::Closed => 1,
                };
                s.set(
                    name,
                    Value::Array(vec![
                        Value::U64(state),
                        Value::Bool(b.locked),
                        Value::Str(b.tag.clone().unwrap_or_default()),
                    ]),
                );
            }
        }
        for (name, sec) in &self.extra {
            doc.sections.insert(name.clone(), sec.clone());
        }
        doc
    }

    pub fn from_document(doc: &Document) -> Result<Airframe, PersistError> {
        if doc.version != crate::document::CURRENT_VERSION {
            return Err(PersistError::UnsupportedVersion(doc.version));
        }
        let a = doc
            .section("airframe")
            .ok_or_else(|| PersistError::Missing("airframe".to_string()))?;
        let mut options = BTreeMap::new();
        for (k, v) in &a.entries {
            if let Some(name) = k.strip_prefix("option.") {
                match v {
                    Value::Str(s) => {
                        options.insert(name.to_string(), s.clone());
                    }
                    _ => return Err(PersistError::WrongType(k.clone())),
                }
            }
        }
        let gap_mode = GapMode::parse(a.str("settings.gap_mode")?)
            .ok_or_else(|| PersistError::WrongType("settings.gap_mode".to_string()))?;
        let stamp = if a.contains("stamp.sim_s") {
            Some(SessionClock {
                sim_s: a.f64("stamp.sim_s")?,
                real_s: a.u64("stamp.real_s")?,
            })
        } else {
            None
        };
        let mut af = Airframe {
            identity: Identity {
                tail: a.str("tail")?.to_string(),
                serial: a.str("serial")?.to_string(),
                model: a.str("model")?.to_string(),
                options,
            },
            settings: Settings {
                gap_mode,
                max_gap_s: a.f64("settings.max_gap_s")?,
            },
            stamp,
            parts: BTreeMap::new(),
            batteries: BTreeMap::new(),
            breakers: BTreeMap::new(),
            extra: BTreeMap::new(),
        };
        for (name, s) in &doc.sections {
            if name == "airframe" {
                continue;
            }
            if let Some(p) = name.strip_prefix("part/") {
                af.parts.insert(p.to_string(), read_part(s)?);
            } else if let Some(b) = name.strip_prefix("battery/") {
                af.batteries.insert(b.to_string(), read_battery(s)?);
            } else if name == "breakers" {
                for (k, v) in &s.entries {
                    af.breakers.insert(k.clone(), read_breaker(k, v)?);
                }
            } else {
                af.extra.insert(name.clone(), s.clone());
            }
        }
        Ok(af)
    }
}

fn read_part(s: &Section) -> Result<PartRecord, PersistError> {
    let mut p = PartRecord::new(s.f64("soak_c")?, s.f64("soak_tau_s")?);
    p.wear = s.f64("wear")?;
    p.run_s = s.f64("run_s")?;
    p.cycles = s.u64("cycles")?;
    p.powered_s = s.f64("powered_s")?;
    p.unpowered_s = s.f64("unpowered_s")?;
    for (k, v) in &s.entries {
        if let Some(n) = k.strip_prefix("nvm.") {
            match v {
                Value::Bytes(b) => {
                    p.nvm.insert(n.to_string(), b.clone());
                }
                _ => return Err(PersistError::WrongType(k.clone())),
            }
        } else if let Some(n) = k.strip_prefix("counter.") {
            let bad = || PersistError::WrongType(k.clone());
            match v {
                Value::Array(items) => match items.as_slice() {
                    [Value::F64(value), Value::Bool(powered_only)] => {
                        p.counters.insert(
                            n.to_string(),
                            Counter {
                                value: *value,
                                powered_only: *powered_only,
                            },
                        );
                    }
                    _ => return Err(bad()),
                },
                _ => return Err(bad()),
            }
        }
    }
    if s.contains("faults") {
        for item in s.array("faults")? {
            match item {
                Value::Array(f) => match f.as_slice() {
                    [Value::F64(t_s), Value::Str(code), Value::U64(flags)] => p.faults.push(FaultEntry {
                        t_s: *t_s,
                        code: code.clone(),
                        flags: *flags,
                    }),
                    _ => return Err(PersistError::WrongType("faults".to_string())),
                },
                _ => return Err(PersistError::WrongType("faults".to_string())),
            }
        }
    }
    Ok(p)
}

fn read_battery(s: &Section) -> Result<BatteryRecord, PersistError> {
    Ok(BatteryRecord {
        soc: s.f64("soc")?,
        cell_soc: s.f64_array("cell_soc")?,
        cell_health: s.f64_array("cell_health")?,
        cycles: s.u64("cycles")?,
        soak_c: s.f64("soak_c")?,
        soak_tau_s: s.f64("soak_tau_s")?,
        self_discharge_per_day: s.f64("self_discharge_per_day")?,
    })
}

fn read_breaker(key: &str, v: &Value) -> Result<BreakerRecord, PersistError> {
    let bad = || PersistError::WrongType(key.to_string());
    match v {
        Value::Array(items) => match items.as_slice() {
            [Value::U64(st), Value::Bool(locked), Value::Str(tag)] => Ok(BreakerRecord {
                state: match st {
                    0 => BreakerState::Open,
                    1 => BreakerState::Closed,
                    _ => return Err(bad()),
                },
                locked: *locked,
                tag: if tag.is_empty() { None } else { Some(tag.clone()) },
            }),
            _ => Err(bad()),
        },
        _ => Err(bad()),
    }
}
