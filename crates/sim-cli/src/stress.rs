use std::panic::{AssertUnwindSafe, catch_unwind};
use std::path::Path;
use sys_electrical::faults::CATALOG;
use sys_electrical::system::{Electrical, SignalValue};

pub struct Rng(u64);

impl Rng {
    pub fn new(seed: u64) -> Rng {
        Rng(seed ^ 0x9e37_79b9_7f4a_7c15)
    }
    pub fn next_u64(&mut self) -> u64 {
        self.0 = self.0.wrapping_add(0x9e37_79b9_7f4a_7c15);
        let mut z = self.0;
        z = (z ^ (z >> 30)).wrapping_mul(0xbf58_476d_1ce4_e5b9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94d0_49bb_1331_11eb);
        z ^ (z >> 31)
    }
    pub fn below(&mut self, n: usize) -> usize {
        (self.next_u64() % n.max(1) as u64) as usize
    }
    pub fn unit(&mut self) -> f64 {
        (self.next_u64() >> 11) as f64 / (1u64 << 53) as f64
    }
}

#[derive(Clone, Debug, Default)]
pub struct StressReport {
    pub seed: u64,
    pub actions: usize,
    pub refused: usize,
    pub faults_armed: usize,
    pub checks: usize,
    pub violations: Vec<String>,
    pub trigger: Vec<String>,
    pub panicked: Option<String>,
    pub fingerprint: u64,
    pub interlock_blocks: usize,
    pub blocked: std::collections::BTreeMap<String, usize>,
    pub deterministic: Option<bool>,
}

const OPEN_LEAK_A: f64 = 0.5;
const LOAD_GEN_W: f64 = 1.0;
const DEAD_V: f64 = 1.0;
const DEAD_LOAD_W: f64 = 5.0;
const SOURCE_V: f64 = 50.0;
const BATTERY_IDLE_A: f64 = 0.5;
const GHOST_V: f64 = 5.0;

const EXT: &str = "v_ll = 115.0, hz = 400.0, kva = 90.0";

const CREW: &[(&str, &str, &str, &[&str])] = &[
    ("crew", "press", "BATTERY", &["ON", "OFF"]),
    ("crew", "press", "EXT_PWR_FWD_L", &["ON", "OFF"]),
    ("crew", "press", "EXT_PWR_FWD_R", &["ON", "OFF"]),
    ("crew", "press", "EXT_PWR_AFT", &["ON", "OFF"]),
    ("crew", "press", "APU_GEN_L", &["ON", "OFF"]),
    ("crew", "press", "APU_GEN_R", &["ON", "OFF"]),
    ("crew", "press", "GEN_CTRL_L1", &["ON", "OFF"]),
    ("crew", "press", "GEN_CTRL_L2", &["ON", "OFF"]),
    ("crew", "press", "GEN_CTRL_R1", &["ON", "OFF"]),
    ("crew", "press", "GEN_CTRL_R2", &["ON", "OFF"]),
    ("crew", "select", "APU_SWITCH", &["START", "ON", "OFF"]),
    ("crew", "rotate", "ENG_START_L", &["START"]),
    ("crew", "move", "FUEL_CONTROL_L", &["RUN"]),
    ("ground", "connect", "EXT_PWR_FWD_L", &[EXT]),
    ("ground", "connect", "EXT_PWR_FWD_R", &[EXT]),
    ("ground", "connect", "EXT_PWR_AFT", &[EXT]),
    ("ground", "disconnect", "EXT_PWR_FWD_L", &[""]),
    ("ground", "set", "EXT_PWR_FWD_L.hz", &["370.0", "400.0", "430.0"]),
    (
        "ground",
        "set",
        "EXT_PWR_FWD_L.v_worst_phase_rms",
        &["100.0", "115.0", "131.0"],
    ),
    ("fault", "vfsg_fail", "L1", &["drive_disconnect"]),
    ("fault", "gen_loss", "ALL_VFSG", &[""]),
    ("fault", "bus_fail", "L1_235", &["dead_short_lockout"]),
    ("fault", "asg_fail", "L", &[""]),
    ("fault", "bpcu_fail", "L_AND_R", &[""]),
    ("fault", "all_engines_flameout", "ALL", &[""]),
];

const SEEDS: &[&[(&str, &str)]] = &[
    &[("airplane_state", "cold_dark"), ("battery_switch", "OFF")],
    &[("airplane_state", "cold_dark"), ("battery_switch", "ON")],
    &[
        ("airplane_state", "cold_dark"),
        ("battery_switch", "ON"),
        ("apu_running", "true"),
        ("apu_gens_online", "true"),
    ],
    &[
        ("airplane_state", "cold_dark"),
        ("battery_switch", "ON"),
        ("engines_running", "[true, true]"),
    ],
    &[
        ("airplane_state", "cruise"),
        ("battery_switch", "ON"),
        ("alt_ft", "35000.0"),
        ("engines_running", "[true, true]"),
    ],
];

fn fault_parts(sys: &Electrical) -> Vec<String> {
    let mut parts: Vec<String> = Vec::new();
    if let Some(s) = sys.net_snapshot() {
        parts.extend(s.contactors.iter().map(|c| c.name.clone()));
        parts.extend(s.sspcs.iter().map(|c| c.id.clone()));
        parts.extend(s.contactors.iter().take(20).map(|c| format!("W:{}", c.name)));
    }
    for (k, c) in sys.circuit_names().iter().enumerate() {
        if k % 8 == 0 {
            parts.push(format!("W:{c}"));
            parts.push(format!("CONN:{c}"));
        }
    }
    for p in [
        "VFSG_L1",
        "VFSG_L2",
        "VFSG_R1",
        "VFSG_R2",
        "ASG_L",
        "ASG_R",
        "RAT",
        "BATT_MAIN",
        "BATT_APU",
        "CHG_MAIN",
        "CHG_APU",
        "DIODE_MAIN",
        "DIODE_APU",
        "SPU",
        "SPDU_L",
        "SPDU_R",
        "GCU_VFSG_L1",
        "GCU_VFSG_R2",
        "GCU_ASG_L",
        "ATU_L",
        "ATU_R",
        "TRU_L",
        "TRU_C1",
        "ATRU_L1",
        "ATRU_R2",
        "ATRU_L1",
        "ATRU_R2",
        "ATU_L",
        "ATU_R",
        "TRU_L",
        "TRU_C1",
        "CMSC_L1",
        "GND:P100",
        "GND:P200",
        "SW:BATTERY",
        "LAMP:BATTERY",
        "EICASIN:L1",
        "RPDU 71",
        "RPDUC 72",
        "ELCC_1",
        "ELCU_1",
        "TCB_1",
        "EXTRCPT_L",
        "OAV_L1",
        "PECS_L",
        "RATHTR",
    ] {
        parts.push(p.to_string());
    }
    parts.retain(|p| sys.part_type_of(p).is_some());
    parts
}

fn modes_for(t: &str) -> Vec<&'static str> {
    CATALOG.iter().filter(|e| e.0 == t).map(|e| e.1).collect()
}

fn fnv(h: &mut u64, bytes: &[u8]) {
    for b in bytes {
        *h ^= *b as u64;
        *h = h.wrapping_mul(0x100_0000_01b3);
    }
}

fn fingerprint(sys: &Electrical) -> u64 {
    let mut h = 0xcbf2_9ce4_8422_2325u64;
    for (k, v) in sys.snapshot() {
        fnv(&mut h, k.as_bytes());
        match v {
            SignalValue::Bool(b) => fnv(&mut h, &[b as u8]),
            SignalValue::Num(x) => fnv(&mut h, &x.to_bits().to_le_bytes()),
            SignalValue::Text(t) => fnv(&mut h, t.as_bytes()),
        }
    }
    for m in sys.eicas_messages() {
        fnv(&mut h, m.as_bytes());
    }
    h
}

fn check(sys: &Electrical, t_s: f64, out: &mut Vec<String>) {
    let Some(s) = sys.net_snapshot() else {
        out.push(format!("t={t_s:.1} no network solution"));
        return;
    };
    if !s.converged {
        out.push(format!(
            "t={t_s:.1} network did not converge (outer {} it, outer resid {:.2e}, inner resid {:.2e})",
            s.outer_iterations, s.outer_residual, s.inner_residual
        ));
    }
    for b in &s.buses {
        if !b.voltage_v.is_finite() || b.voltage_v < -1.0 || b.voltage_v > 450.0 {
            out.push(format!("t={t_s:.1} bus {} voltage {}", b.name, b.voltage_v));
        }
        if !b.freq_hz.is_finite() || b.freq_hz < 0.0 || b.freq_hz > 1000.0 {
            out.push(format!("t={t_s:.1} bus {} frequency {}", b.name, b.freq_hz));
        }
    }
    for bat in &s.batteries {
        if !(0.0..=1.0).contains(&bat.soc) || !bat.voltage_v.is_finite() || !bat.current_a.is_finite() {
            out.push(format!(
                "t={t_s:.1} battery {} soc {} v {} i {}",
                bat.name, bat.soc, bat.voltage_v, bat.current_a
            ));
        }
        if !(200.0..=420.0).contains(&bat.temp_k) {
            out.push(format!(
                "t={t_s:.1} battery {} temperature {} K",
                bat.name, bat.temp_k
            ));
        }
    }
    for m in &s.machines {
        if !m.terminal_v.is_finite() || m.terminal_v > 600.0 || !m.current_a.is_finite() {
            out.push(format!(
                "t={t_s:.1} machine {} v {} i {}",
                m.name, m.terminal_v, m.current_a
            ));
        }
    }
    for c in &s.contactors {
        if !c.current_a.is_finite() {
            out.push(format!("t={t_s:.1} contactor {} current {}", c.name, c.current_a));
        }
        if let Some(temp) = sys.contactor_temp_c(&c.name)
            && (!temp.is_finite() || !(-80.0..=400.0).contains(&temp))
        {
            out.push(format!(
                "t={t_s:.1} contactor {} temperature {temp} C at {:.1} A closed {}",
                c.name, c.current_a, c.closed
            ));
        }
    }
    for c in &s.contactors {
        if !c.closed && c.current_a.abs() > OPEN_LEAK_A {
            out.push(format!(
                "t={t_s:.1} contactor {} open but carrying {:.2} A",
                c.name, c.current_a
            ));
        }
    }
    for l in &s.loads {
        if l.power_w < -LOAD_GEN_W {
            out.push(format!(
                "t={t_s:.1} load {} generating {:.1} W",
                l.name, l.power_w
            ));
        }
        if l.voltage_v.abs() < DEAD_V && l.power_w.abs() > DEAD_LOAD_W {
            out.push(format!(
                "t={t_s:.1} load {} draws {:.1} W at {:.2} V",
                l.name, l.power_w, l.voltage_v
            ));
        }
    }
    let machine_live = s.machines.iter().any(|m| m.terminal_v > SOURCE_V);
    let battery_live = s.batteries.iter().any(|b| b.current_a.abs() > BATTERY_IDLE_A);
    let ext_live = sys.inputs().ext.iter().any(|e| e.connected && e.v_ll > SOURCE_V)
        || s.buses.iter().any(|b| b.name == "SPU" && b.voltage_v > SOURCE_V);
    if !machine_live && !battery_live && !ext_live {
        for b in &s.buses {
            if b.voltage_v > GHOST_V {
                out.push(format!(
                    "t={t_s:.1} bus {} at {:.1} V with every source dead",
                    b.name, b.voltage_v
                ));
            }
        }
    }
    for (k, v) in sys.snapshot() {
        if let SignalValue::Num(x) = v
            && x.is_nan()
        {
            out.push(format!("t={t_s:.1} signal {k} is NaN"));
        }
    }
}

fn run_inner(data_dir: &Path, seed: u64, sim_s: u64) -> Result<(StressReport, u64), String> {
    let mut rng = Rng::new(seed);
    let mut sys = Electrical::new(data_dir)?;
    let mut rep = StressReport {
        seed,
        ..StressReport::default()
    };
    for (k, v) in SEEDS[rng.below(SEEDS.len())] {
        let _ = sys.action("sim", "set", k, Some(v));
    }
    sys.step_idle(2_000_000);
    let parts = fault_parts(&sys);
    let mut t_s = 2u64;
    let mut seen = Vec::new();
    let mut log: Vec<String> = Vec::new();
    while t_s < sim_s {
        let gap = 1 + rng.below(20) as u64;
        for _ in 0..gap {
            sys.step_idle(1_000_000);
            t_s += 1;
            rep.checks += 1;
            rep.interlock_blocks += sys.parallel_blocks().len();
            for n in sys.parallel_blocks() {
                *rep.blocked.entry(n.clone()).or_insert(0) += 1;
            }
            let before = seen.len();
            check(&sys, t_s as f64, &mut seen);
            if before == 0 && !seen.is_empty() {
                rep.trigger = log.iter().rev().take(6).rev().cloned().collect();
                rep.trigger.push(format!("mode {}", sys.mode()));
                if let Some(sn) = sys.net_snapshot() {
                    let closed: Vec<String> = sn
                        .contactors
                        .iter()
                        .filter(|c| c.closed && c.current_a.abs() > 50.0)
                        .map(|c| format!("{} {:.0}A", c.name, c.current_a))
                        .collect();
                    rep.trigger
                        .push(format!("high-current closed: {}", closed.join(", ")));
                    let m: Vec<String> = sn
                        .machines
                        .iter()
                        .filter(|m| m.terminal_v > 1.0)
                        .map(|m| {
                            format!(
                                "{} {:.0}Hz {:.0}V {:.0}A",
                                m.name, m.freq_hz, m.terminal_v, m.current_a
                            )
                        })
                        .collect();
                    rep.trigger.push(format!("machines: {}", m.join(", ")));
                }
            }
        }
        let roll = rng.below(10);
        if roll < 6 {
            let (actor, action, target, values) = CREW[rng.below(CREW.len())];
            let v = values[rng.below(values.len())];
            let value = if v.is_empty() { None } else { Some(v) };
            rep.actions += 1;
            log.push(format!(
                "t={t_s} {actor} {action} {target} {}",
                value.unwrap_or("")
            ));
            if sys.action(actor, action, target, value).is_err() {
                rep.refused += 1;
            }
        } else if roll < 9 && !parts.is_empty() {
            let part = &parts[rng.below(parts.len())];
            if let Some(t) = sys.part_type_of(part) {
                let modes = modes_for(t);
                if !modes.is_empty() {
                    let mode = modes[rng.below(modes.len())];
                    let mag = 0.2 + 0.8 * rng.unit();
                    rep.actions += 1;
                    log.push(format!("t={t_s} fault arm {part} {mode} {mag:.3}"));
                    if sys
                        .action("fault", "arm", part, Some(&format!("{mode} {mag:.3}")))
                        .is_ok()
                    {
                        rep.faults_armed += 1;
                    } else {
                        rep.refused += 1;
                    }
                }
            }
        } else if let Some((p, _, _)) = sys.active_faults().first().cloned() {
            rep.actions += 1;
            log.push(format!("t={t_s} fault clear {p}"));
            let _ = sys.action("fault", "clear", &p, None);
        }
    }
    seen.dedup();
    rep.violations = seen;
    let fp = fingerprint(&sys);
    rep.fingerprint = fp;
    Ok((rep, fp))
}

pub fn run_seed(data_dir: &Path, seed: u64, sim_s: u64, verify_determinism: bool) -> StressReport {
    let first = catch_unwind(AssertUnwindSafe(|| run_inner(data_dir, seed, sim_s)));
    let mut rep = match first {
        Ok(Ok((r, _))) => r,
        Ok(Err(e)) => StressReport {
            seed,
            violations: vec![format!("setup failed: {e}")],
            ..StressReport::default()
        },
        Err(p) => StressReport {
            seed,
            panicked: Some(
                p.downcast_ref::<String>()
                    .cloned()
                    .or_else(|| p.downcast_ref::<&str>().map(|s| s.to_string()))
                    .unwrap_or_else(|| String::from("panic")),
            ),
            ..StressReport::default()
        },
    };
    if verify_determinism && rep.panicked.is_none() {
        let again = catch_unwind(AssertUnwindSafe(|| run_inner(data_dir, seed, sim_s)));
        rep.deterministic = Some(matches!(again, Ok(Ok((_, fp))) if fp == rep.fingerprint));
    }
    rep
}

pub fn summary(reports: &[StressReport]) -> String {
    let mut s = String::new();
    let mut bad = 0;
    for r in reports {
        let ok = r.panicked.is_none() && r.violations.is_empty() && r.deterministic != Some(false);
        if !ok {
            bad += 1;
        }
        s.push_str(&format!(
            "seed {:>5}  {}  actions {:>4} refused {:>3} faults {:>3} checks {:>5} interlock {:>4} violations {:>3}{}{}\n",
            r.seed,
            if ok { "ok  " } else { "FAIL" },
            r.actions,
            r.refused,
            r.faults_armed,
            r.checks,
            r.interlock_blocks,
            r.violations.len(),
            match r.deterministic {
                Some(true) => "  deterministic",
                Some(false) => "  NOT DETERMINISTIC",
                None => "",
            },
            r.panicked.as_ref().map(|p| format!("  PANIC: {p}")).unwrap_or_default(),
        ));
        for v in r.violations.iter().take(3) {
            s.push_str(&format!("      {v}\n"));
        }
        for a in &r.trigger {
            s.push_str(&format!("        after: {a}\n"));
        }
    }
    let mut all: std::collections::BTreeMap<&str, usize> = std::collections::BTreeMap::new();
    for r in reports {
        for (k, v) in &r.blocked {
            *all.entry(k.as_str()).or_insert(0) += v;
        }
    }
    let mut top: Vec<(&str, usize)> = all.into_iter().collect();
    top.sort_by(|a, b| b.1.cmp(&a.1).then(a.0.cmp(b.0)));
    for (k, v) in top.iter().take(12) {
        s.push_str(&format!("interlock held open: {k} x{v}\n"));
    }
    s.push_str(&format!("TOTAL seeds {} failed {}\n", reports.len(), bad));
    s
}
