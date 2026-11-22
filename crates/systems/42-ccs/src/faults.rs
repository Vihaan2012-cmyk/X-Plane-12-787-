use crate::cabinet::GPM_COUNT;
use crate::ccs::{Ccs, IFZ_NAMES};
use crate::cdn::DevKind;

pub const CATALOG: &[(&str, &[&str])] = &[
    ("pcm", &["failed"]),
    ("gpm", &["failed", "lane-a", "lane-b", "partition-crash"]),
    ("acs", &["failed", "replay"]),
    ("ars", &["failed", "replay"]),
    ("fox", &["failed"]),
    ("link", &["open", "bit-errors"]),
    (
        "rdc",
        &[
            "failed", "babbling", "corrupt", "ifz1", "ifz2", "ifz3", "ifz4", "ifz5", "ifz6", "ifz7", "ifz8",
            "ifz9", "ifz10", "ifz11", "ifz12", "ifz13", "ifz14",
        ],
    ),
    ("lru-es", &["failed", "babbling", "corrupt", "wrong-load"]),
    ("cooling", &["fan", "valve"]),
];

pub const APPROX_FAULT_BER: f64 = 1.0e-5;

pub fn modes(part_type: &str) -> &'static [&'static str] {
    CATALOG
        .iter()
        .find(|(t, _)| *t == part_type)
        .map(|(_, m)| *m)
        .unwrap_or(&[])
}

pub fn instances(c: &Ccs, part_type: &str) -> Vec<String> {
    match part_type {
        "pcm" => ["L", "R"]
            .iter()
            .flat_map(|s| (1..=2).map(move |k| format!("PCM {s}{k}")))
            .collect(),
        "gpm" => crate::network::GPMS.iter().map(|g| format!("GPM {g}")).collect(),
        "acs" | "ars" | "fox" => {
            let kind = if part_type == "fox" {
                DevKind::Fox
            } else {
                DevKind::Switch
            };
            let prefix = part_type.to_ascii_uppercase();
            c.cdn
                .devices
                .iter()
                .filter(|d| d.kind == kind && d.name.starts_with(&prefix))
                .map(|d| d.name.clone())
                .collect()
        }
        "link" => c
            .cdn
            .links
            .iter()
            .map(|l| format!("LINK {}|{}", c.cdn.devices[l.a].name, c.cdn.devices[l.b].name))
            .collect(),
        "rdc" => c.rdcs.iter().map(|r| r.def.id.clone()).collect(),
        "lru-es" => ["P300", "P400", "RPDU 71", "RPDU 72", "RPDU 81", "RPDU 82"]
            .iter()
            .map(|s| s.to_string())
            .collect(),
        "cooling" => vec!["CCR COOLING L".into(), "CCR COOLING R".into()],
        _ => Vec::new(),
    }
}

pub fn failure_points(c: &Ccs) -> usize {
    CATALOG.iter().map(|(t, m)| instances(c, t).len() * m.len()).sum()
}

pub fn part_type_of(c: &Ccs, part: &str) -> Option<&'static str> {
    CATALOG
        .iter()
        .map(|(t, _)| *t)
        .find(|t| instances(c, t).iter().any(|p| p == part))
}

pub fn arm(c: &mut Ccs, part: &str, mode: &str, on: bool) -> Result<(), String> {
    let t = part_type_of(c, part).ok_or_else(|| format!("unknown CCS part {part}"))?;
    if !modes(t).contains(&mode) {
        return Err(format!("{part} has no mode {mode}"));
    }
    match t {
        "pcm" => {
            let side = if part.contains(" L") { 0 } else { 1 };
            let k: usize = part[part.len() - 1..].parse().map_err(|_| "pcm index")?;
            c.cabinets[side].pcm[k - 1].failed = on;
        }
        "gpm" => {
            let g = part.trim_start_matches("GPM ");
            let side = if g.starts_with('L') { 0 } else { 1 };
            let k: usize = g[1..].parse().map_err(|_| "gpm index")?;
            match mode {
                "failed" => c.cabinets[side].gpm[k - 1].failed = on,
                "lane-a" | "lane-b" => {
                    let sw = &mut c.sw[side * GPM_COUNT + k - 1];
                    sw.lane_failed[usize::from(mode == "lane-b")] = on;
                }
                _ => {
                    let first = c
                        .sw_of(g)
                        .and_then(|s| s.partitions.iter().find(|p| p.hosted))
                        .map(|p| p.name.clone())
                        .ok_or("no hosted partition")?;
                    let key = (g.to_string(), first);
                    if on {
                        c.part_faults.insert(key);
                    } else {
                        c.part_faults.remove(&key);
                    }
                }
            }
        }
        "acs" | "ars" | "fox" => {
            c.set_device_fault(part, mode, on);
        }
        "link" => {
            let (a, b) = part.trim_start_matches("LINK ").split_once('|').ok_or("link")?;
            let (ber, open) = match mode {
                "open" => (0.0, on),
                _ => (if on { APPROX_FAULT_BER } else { 0.0 }, false),
            };
            c.set_link_fault(a, b, ber, open);
        }
        "rdc" => match mode {
            "failed" => {
                c.set_rdc_failed(part, on);
            }
            "babbling" | "corrupt" => {
                c.set_device_fault(part, mode, on);
            }
            z => {
                let k = IFZ_NAMES.iter().position(|n| *n == z).ok_or("ifz")?;
                if let Some(r) = c.rdcs.iter_mut().find(|r| r.def.id == part) {
                    r.ifz_failed[k] = on;
                }
            }
        },
        "lru-es" => match mode {
            "wrong-load" => {
                let k = match part {
                    "P300" => 0,
                    "P400" => 1,
                    _ => return Err("wrong-load only on P300/P400".into()),
                };
                c.icd_mismatch[k] = on;
            }
            m => {
                c.set_device_fault(part, m, on);
            }
        },
        "cooling" => {
            let side = if part.ends_with('L') { 0 } else { 1 };
            if mode == "fan" {
                c.cabinets[side].fan_failed = on;
            } else {
                c.cabinets[side].valve_failed = on;
            }
        }
        _ => return Err("unhandled".into()),
    }
    Ok(())
}
