use crate::value::Value;
use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Op {
    True,
    False,
    Eq,
    Approx,
    Within,
    Lt,
    Gt,
    Present,
    Absent,
}

impl Op {
    pub fn parse(s: &str) -> Option<Op> {
        Some(match s {
            "true" => Op::True,
            "false" => Op::False,
            "eq" => Op::Eq,
            "approx" => Op::Approx,
            "within" => Op::Within,
            "lt" => Op::Lt,
            "gt" => Op::Gt,
            "present" => Op::Present,
            "absent" => Op::Absent,
            _ => return None,
        })
    }

    pub fn name(self) -> &'static str {
        match self {
            Op::True => "true",
            Op::False => "false",
            Op::Eq => "eq",
            Op::Approx => "approx",
            Op::Within => "within",
            Op::Lt => "lt",
            Op::Gt => "gt",
            Op::Present => "present",
            Op::Absent => "absent",
        }
    }
}

#[derive(Debug, Clone)]
pub struct Step {
    pub t_us: u64,
    pub actor: String,
    pub action: String,
    pub target: String,
    pub value: Option<Value>,
}

#[derive(Debug, Clone)]
pub struct Expect {
    pub id: String,
    pub t_us: u64,
    pub signal: String,
    pub op: Op,
    pub value: Option<Value>,
    pub tol: f64,
    pub unit: String,
    pub approx: bool,
    pub cite: String,
}

#[derive(Debug, Clone)]
pub struct Scenario {
    pub id: String,
    pub title: String,
    pub system: String,
    pub tick_hz: u32,
    pub initial: BTreeMap<String, Value>,
    pub steps: Vec<Step>,
    pub expects: Vec<Expect>,
}

#[derive(Debug, Clone)]
pub struct Fact {
    pub id: String,
    pub quantity: String,
    pub expected: Value,
    pub unit: String,
    pub tol: f64,
    pub condition: String,
    pub approx: bool,
    pub cite: String,
}

pub fn secs_to_us(t: f64) -> u64 {
    (t * 1_000_000.0).round().max(0.0) as u64
}

fn get_str(t: &toml::Table, key: &str) -> String {
    t.get(key).and_then(|v| v.as_str()).unwrap_or("").to_string()
}

fn get_num(t: &toml::Table, key: &str) -> Option<f64> {
    t.get(key).and_then(|v| match v {
        toml::Value::Integer(i) => Some(*i as f64),
        toml::Value::Float(f) => Some(*f),
        _ => None,
    })
}

fn tables<'a>(root: &'a toml::Table, key: &str) -> Vec<&'a toml::Table> {
    root.get(key)
        .and_then(|v| v.as_array())
        .map(|a| a.iter().filter_map(|v| v.as_table()).collect())
        .unwrap_or_default()
}

pub fn parse_scenario(text: &str) -> Result<Scenario, String> {
    let root: toml::Table = text.parse().map_err(|e| format!("toml: {e}"))?;
    let head = root
        .get("scenario")
        .and_then(|v| v.as_table())
        .ok_or_else(|| "missing [scenario]".to_string())?;
    let id = get_str(head, "id");
    if id.is_empty() {
        return Err("scenario.id missing".to_string());
    }
    let tick_hz = get_num(head, "tick_hz").unwrap_or(1000.0);
    if !(1.0..=1_000_000.0).contains(&tick_hz) {
        return Err(format!("tick_hz out of range: {tick_hz}"));
    }
    let initial = root
        .get("initial")
        .and_then(|v| v.as_table())
        .map(|t| t.iter().map(|(k, v)| (k.clone(), Value::from_toml(v))).collect())
        .unwrap_or_default();
    let mut steps = Vec::new();
    for (i, t) in tables(&root, "step").into_iter().enumerate() {
        let t_s = get_num(t, "t_s").ok_or_else(|| format!("step {i}: t_s missing"))?;
        steps.push(Step {
            t_us: secs_to_us(t_s),
            actor: get_str(t, "actor"),
            action: get_str(t, "action"),
            target: get_str(t, "target"),
            value: t.get("value").map(Value::from_toml),
        });
    }
    let mut expects = Vec::new();
    for (i, t) in tables(&root, "expect").into_iter().enumerate() {
        let t_s = get_num(t, "t_s").ok_or_else(|| format!("expect {i}: t_s missing"))?;
        let op_s = get_str(t, "op");
        let op = Op::parse(&op_s).ok_or_else(|| format!("expect {i}: unknown op '{op_s}'"))?;
        let signal = get_str(t, "signal");
        if signal.is_empty() {
            return Err(format!("expect {i}: signal missing"));
        }
        let value = t.get("value").map(Value::from_toml);
        if matches!(op, Op::Eq | Op::Approx | Op::Within | Op::Lt | Op::Gt) && value.is_none() {
            return Err(format!("expect {i} ({signal}): op {op_s} needs value"));
        }
        let mut eid = get_str(t, "id");
        if eid.is_empty() {
            eid = format!("{id}.e{:02}", i + 1);
        }
        expects.push(Expect {
            id: eid,
            t_us: secs_to_us(t_s),
            signal,
            op,
            value,
            tol: get_num(t, "tol").unwrap_or(0.0),
            unit: get_str(t, "unit"),
            approx: t.get("approx").and_then(|v| v.as_bool()).unwrap_or(false),
            cite: get_str(t, "cite"),
        });
    }
    Ok(Scenario {
        title: get_str(head, "title"),
        system: get_str(head, "system"),
        id,
        tick_hz: tick_hz as u32,
        initial,
        steps,
        expects,
    })
}

pub fn parse_facts(text: &str) -> Result<Vec<Fact>, String> {
    let root: toml::Table = text.parse().map_err(|e| format!("toml: {e}"))?;
    let mut out = Vec::new();
    for (i, t) in tables(&root, "fact").into_iter().enumerate() {
        let id = get_str(t, "id");
        let quantity = get_str(t, "quantity");
        if quantity.is_empty() {
            return Err(format!("fact {i}: quantity missing"));
        }
        let expected = t
            .get("expected")
            .map(Value::from_toml)
            .ok_or_else(|| format!("fact {id}: expected missing"))?;
        out.push(Fact {
            id,
            quantity,
            expected,
            unit: get_str(t, "unit"),
            tol: get_num(t, "tol").unwrap_or(0.0),
            condition: get_str(t, "condition"),
            approx: t.get("approx").and_then(|v| v.as_bool()).unwrap_or(false),
            cite: get_str(t, "cite"),
        });
    }
    Ok(out)
}

pub fn load_scenario(path: &Path) -> Result<Scenario, String> {
    let text = fs::read_to_string(path).map_err(|e| format!("{}: {e}", path.display()))?;
    parse_scenario(&text).map_err(|e| format!("{}: {e}", path.display()))
}

pub fn load_facts(path: &Path) -> Result<Vec<Fact>, String> {
    let text = fs::read_to_string(path).map_err(|e| format!("{}: {e}", path.display()))?;
    parse_facts(&text).map_err(|e| format!("{}: {e}", path.display()))
}

pub fn collect_files(path: &Path, ext: &str, out: &mut Vec<PathBuf>) -> Result<(), String> {
    if path.is_file() {
        out.push(path.to_path_buf());
        return Ok(());
    }
    let mut entries: Vec<PathBuf> = fs::read_dir(path)
        .map_err(|e| format!("{}: {e}", path.display()))?
        .filter_map(|e| e.ok().map(|e| e.path()))
        .collect();
    entries.sort();
    for p in entries {
        if p.is_dir() {
            collect_files(&p, ext, out)?;
        } else if p.extension().and_then(|e| e.to_str()) == Some(ext)
            && p.file_name().and_then(|n| n.to_str()) != Some("README.toml")
        {
            out.push(p);
        }
    }
    Ok(())
}
