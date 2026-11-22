use crate::aircraft::AircraftTarget;
use crate::target::Target;
use crate::value::Value;
use std::collections::BTreeSet;
use std::path::Path;
use sys_ccs::cabinet::CabinetState;
use sys_ccs::faults;

pub const STEP_US: u64 = 100_000;
pub const CHECK_EVERY: u64 = 10;
pub const APPROX_CAS_LAG_US: u64 = 2_000_000;

#[derive(Clone, Debug, Default)]
pub struct AircraftReport {
    pub seed: u64,
    pub actions: usize,
    pub faults: usize,
    pub checks: usize,
    pub violations: Vec<String>,
    pub fingerprint: u64,
    pub deterministic: Option<bool>,
    pub panicked: Option<String>,
}

fn rng(s: &mut u64) -> u64 {
    *s ^= *s << 13;
    *s ^= *s >> 7;
    *s ^= *s << 17;
    *s
}

fn pick<'a, T>(s: &mut u64, v: &'a [T]) -> &'a T {
    &v[(rng(s) % v.len() as u64) as usize]
}

fn fnv(h: &mut u64, text: &str) {
    for b in text.bytes() {
        *h ^= u64::from(b);
        *h = h.wrapping_mul(0x100_0000_01B3);
    }
}

fn act(a: &mut AircraftTarget, actor: &str, action: &str, target: &str, value: Option<&str>) -> bool {
    let v = value.map(|t| Value::Str(t.to_string()));
    a.apply(actor, action, target, v.as_ref()).is_ok()
}

fn run_inner(data_dir: &Path, seed: u64, sim_s: u64) -> Result<AircraftReport, String> {
    let mut a = AircraftTarget::new(data_dir)?;
    let mut s = seed.wrapping_mul(0x9E37_79B9_7F4A_7C15) | 1;
    let mut rep = AircraftReport {
        seed,
        ..AircraftReport::default()
    };
    act(&mut a, "sim", "set", "airplane_state", Some("cold_dark"));
    act(&mut a, "crew", "press", "BATTERY", Some("ON"));
    let steps = sim_s * 1_000_000 / STEP_US;
    let mut next_action = 50u64;
    let mut armed: BTreeSet<(String, String)> = BTreeSet::new();
    let mut recent_elec: Vec<(u64, Vec<String>)> = Vec::new();
    let types: Vec<&str> = faults::CATALOG.iter().map(|(t, _)| *t).collect();
    let mut h = 0xCBF2_9CE4_8422_2325u64;
    for k in 0..steps {
        let now = k * STEP_US;
        if k >= next_action {
            next_action = k + 50 + rng(&mut s) % 150;
            rep.actions += 1;
            match rng(&mut s) % 10 {
                0 => {
                    let side = *pick(&mut s, &["EXT_PWR_FWD_L", "EXT_PWR_FWD_R"]);
                    act(&mut a, "ground", "connect", side, Some("v_ll=115 hz=400 kva=90"));
                    act(&mut a, "crew", "press", side, None);
                }
                1 => {
                    act(
                        &mut a,
                        "crew",
                        "press",
                        pick(&mut s, &["CCR_RESET_L", "CCR_RESET_R"]),
                        None,
                    );
                }
                2 => {
                    act(
                        &mut a,
                        "crew",
                        "press",
                        "BATTERY",
                        Some(*pick(&mut s, &["ON", "OFF"])),
                    );
                }
                3 => {
                    if let Some(f) = armed.iter().next().cloned() {
                        act(&mut a, "fault", "clear", &f.0, Some(&f.1));
                        armed.remove(&f);
                    }
                }
                4 => {
                    act(
                        &mut a,
                        "ground",
                        "disconnect",
                        pick(&mut s, &["EXT_PWR_FWD_L", "EXT_PWR_FWD_R"]),
                        None,
                    );
                }
                _ => {
                    let t = *pick(&mut s, &types);
                    let parts = faults::instances(a.ccs(), t);
                    if parts.is_empty() {
                        continue;
                    }
                    let part = pick(&mut s, &parts).clone();
                    let mode = *pick(&mut s, faults::modes(t));
                    if act(&mut a, "fault", "arm", &part, Some(mode)) {
                        rep.faults += 1;
                        armed.insert((part, mode.to_string()));
                    }
                }
            }
        }
        a.step(STEP_US);
        let elec_now = a.elec().eicas_messages();
        recent_elec.push((now, elec_now));
        recent_elec.retain(|(t, _)| now.saturating_sub(*t) <= APPROX_CAS_LAG_US);
        if k % CHECK_EVERY != 0 {
            continue;
        }
        rep.checks += 1;
        let c = a.ccs();
        for app in c.hosted.keys() {
            if c.app_available(app) && !c.hosted[app].iter().any(|g| c.gpm_running(g)) {
                rep.violations.push(format!(
                    "t={} {app} available with no host running",
                    now / 1_000_000
                ));
            }
        }
        for (i, cab) in c.cabinets.iter().enumerate() {
            if cab.state != CabinetState::Off && !cab.powered() {
                rep.violations
                    .push(format!("t={} cabinet {i} up without power", now / 1_000_000));
            }
            let w: f64 = c.loads_w()[i * 3..i * 3 + 3].iter().map(|(_, w)| *w).sum();
            if !cab.powered() && w > 0.0 {
                rep.violations
                    .push(format!("t={} cabinet {i} draws {w} W unpowered", now / 1_000_000));
            }
        }
        let corrupt_source = armed
            .iter()
            .any(|(p, m)| m == "corrupt" && (p == "P300" || p == "P400"));
        if let Some((msgs, true)) = c.cas_elec_messages().filter(|_| !corrupt_source) {
            for m in msgs {
                if !recent_elec.iter().any(|(_, list)| list.contains(&m)) {
                    let host = c.cas_live_host().unwrap_or("-");
                    let dest = format!("GPM {host}");
                    let ages: Vec<String> = ["BPCU P300", "BPCU P400"]
                        .iter()
                        .map(|v| format!("{v}:{:?}", c.read(&dest, v).map(|(_, a, ok)| (a, ok))))
                        .collect();
                    rep.violations.push(format!(
                        "t={} CAS shows {m} not active in electrical [host {host} {ages:?} lost {} faults {armed:?}]",
                        now / 1_000_000,
                        c.monitor_lost()
                    ));
                }
            }
        }
        for r in &c.rdcs {
            if let Some((_, _, true)) = c.read("GPM L1", &format!("{} DATA", r.def.id))
                && !c.monitor_lost()
                && r.powered_since.is_none()
            {
                rep.violations.push(format!(
                    "t={} {} data valid while unpowered",
                    now / 1_000_000,
                    r.def.id
                ));
            }
        }
        if c.cdn.unknown_vl_drops > 0 {
            rep.violations.push(format!(
                "t={} unknown VL drops {}",
                now / 1_000_000,
                c.cdn.unknown_vl_drops
            ));
        }
        fnv(&mut h, &format!("{:?}", c.crew_status_messages()));
        fnv(
            &mut h,
            &format!("{:?}{:?}", c.cabinets[0].state, c.cabinets[1].state),
        );
        fnv(&mut h, &format!("{:?}", c.cas_elec_messages()));
        if rep.violations.len() > 50 {
            break;
        }
    }
    rep.fingerprint = h;
    Ok(rep)
}

pub fn run_seed(data_dir: &Path, seed: u64, sim_s: u64, determinism: bool) -> AircraftReport {
    let first = std::panic::catch_unwind(|| run_inner(data_dir, seed, sim_s));
    let mut rep = match first {
        Ok(Ok(r)) => r,
        Ok(Err(e)) => AircraftReport {
            seed,
            violations: vec![format!("setup failed: {e}")],
            ..AircraftReport::default()
        },
        Err(p) => AircraftReport {
            seed,
            panicked: Some(
                p.downcast_ref::<String>()
                    .cloned()
                    .or_else(|| p.downcast_ref::<&str>().map(|s| s.to_string()))
                    .unwrap_or_default(),
            ),
            ..AircraftReport::default()
        },
    };
    if determinism && rep.panicked.is_none() {
        let again = std::panic::catch_unwind(|| run_inner(data_dir, seed, sim_s));
        rep.deterministic = Some(matches!(again, Ok(Ok(ref r)) if r.fingerprint == rep.fingerprint));
    }
    rep
}

pub fn summary(reports: &[AircraftReport]) -> String {
    let mut out = String::new();
    let mut bad = 0;
    for r in reports {
        let ok = r.panicked.is_none() && r.violations.is_empty() && r.deterministic != Some(false);
        if !ok {
            bad += 1;
        }
        out.push_str(&format!(
            "seed {:5}  {}  actions {:4} faults {:4} checks {:5} violations {:3}{}\n",
            r.seed,
            if ok { "ok  " } else { "FAIL" },
            r.actions,
            r.faults,
            r.checks,
            r.violations.len(),
            match r.deterministic {
                Some(true) => "  deterministic",
                Some(false) => "  NOT DETERMINISTIC",
                None => "",
            }
        ));
        if let Some(p) = &r.panicked {
            out.push_str(&format!("      panic: {p}\n"));
        }
        for v in r.violations.iter().take(5) {
            out.push_str(&format!("      {v}\n"));
        }
    }
    out.push_str(&format!("TOTAL seeds {} failed {}\n", reports.len(), bad));
    out
}
