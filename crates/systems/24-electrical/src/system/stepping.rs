use super::{Electrical, ElectricalInputs, SUBSTEP_US};
use crate::control::{BpcuOutput, LmOutput};
use crate::sources::{ExtPowerOutput, GcuOutput};
use crate::storage::{BmuOutput, ChargerOutput};

pub const MAX_COARSE_MS: u64 = 50;
const QUIET_NEEDED: u32 = 3;
const DRIFT_QUIET_PER_MS: f64 = 2.0;
const DRIFT_BUDGET: f64 = 100.0;
const APPROX_REGULATION_QUIET_A_PER_MS: f64 = 0.0003;
const APPROX_REGULATION_QUIET_V_PER_MS: f64 = 0.01;

#[derive(Clone, Debug, Default)]
pub(crate) struct Frame {
    pub(crate) gcu_out: Vec<GcuOutput>,
    pub(crate) ext_out: Vec<ExtPowerOutput>,
    pub(crate) bpcu_out: Option<BpcuOutput>,
    pub(crate) lm_out: Option<LmOutput>,
    pub(crate) bmu_out: Vec<BmuOutput>,
    pub(crate) charger_out: Vec<ChargerOutput>,
    pub(crate) gear: Vec<u8>,
    pub(crate) analog: Vec<f64>,
    valid: bool,
}

#[derive(Clone, Debug, Default)]
pub(crate) struct Pace {
    pub(crate) fine: bool,
    pub(crate) quiet: u32,
    pub(crate) drift_per_ms: f64,
    prev: Frame,
    pub(crate) scratch: Frame,
    pub(crate) solves: u64,
}

impl Pace {
    pub(crate) fn new() -> Pace {
        Pace::default()
    }
}

fn lm_same(a: &LmOutput, b: &LmOutput) -> bool {
    let mut x = *a;
    let mut y = *b;
    x.total_kw = 0.0;
    y.total_kw = 0.0;
    x == y
}

fn bmu_same(a: &BmuOutput, b: &BmuOutput) -> bool {
    let mut x = *a;
    let mut y = *b;
    for o in [&mut x, &mut y] {
        o.min_cell_v = 0.0;
        o.max_cell_v = 0.0;
        o.spread_v = 0.0;
    }
    x == y
}

fn lm_opt_same(a: &Option<LmOutput>, b: &Option<LmOutput>) -> bool {
    match (a, b) {
        (Some(x), Some(y)) => lm_same(x, y),
        (None, None) => true,
        _ => false,
    }
}

fn bmus_same(a: &[BmuOutput], b: &[BmuOutput]) -> bool {
    a.len() == b.len() && a.iter().zip(b).all(|(x, y)| bmu_same(x, y))
}

fn gcu_same(a: &GcuOutput, b: &GcuOutput) -> bool {
    let mut x = *a;
    let mut y = *b;
    for o in [&mut x, &mut y] {
        o.exciter_current_a = 0.0;
        o.regulated_v = 0.0;
        o.powered_ticks = 0;
    }
    x == y
}

fn gcus_same(a: &[GcuOutput], b: &[GcuOutput]) -> bool {
    a.len() == b.len() && a.iter().zip(b).all(|(x, y)| gcu_same(x, y))
}

fn regulation_settled(a: &[GcuOutput], b: &[GcuOutput], h: u64) -> bool {
    let scale = SUBSTEP_US as f64 / h as f64;
    a.iter().zip(b).all(|(x, y)| {
        !(x.vr_enabled && y.vr_enabled)
            || (y.voltage_established
                && (x.exciter_current_a - y.exciter_current_a).abs() * scale
                    < APPROX_REGULATION_QUIET_A_PER_MS
                && (x.regulated_v - y.regulated_v).abs() * scale < APPROX_REGULATION_QUIET_V_PER_MS)
    })
}

fn chargers_same(a: &[ChargerOutput], b: &[ChargerOutput]) -> bool {
    a.len() == b.len()
        && a.iter()
            .zip(b)
            .all(|(x, y)| x.mode == y.mode && x.failed == y.failed)
}

fn gear_bits(g: &super::Switchgear) -> u8 {
    u8::from(g.cmd)
        | u8::from(g.out.contacts_closed) << 1
        | u8::from(g.out.armature_closed) << 2
        | u8::from(g.out.aux_no) << 3
        | u8::from(g.out.aux_nc) << 4
        | u8::from(g.out.bouncing) << 5
        | u8::from(g.out.welded) << 6
        | u8::from(g.forced_open) << 7
}

impl Electrical {
    pub fn set_fine_stepping(&mut self, fine: bool) {
        self.pace.fine = fine;
    }

    pub fn solve_count(&self) -> u64 {
        self.pace.solves
    }

    pub fn step(&mut self, dt_us: u64, inputs: &ElectricalInputs) {
        self.apply_crew(&inputs.crew);
        let mut shared = inputs.clone();
        shared.crew.clear();
        if !inputs.crew.is_empty() || shared != self.inputs {
            self.pace.quiet = 0;
        }
        self.inputs = shared;
        let mut left = dt_us;
        while left > 0 {
            let h = self.next_substep(left);
            self.substep(h);
            self.pace.solves += 1;
            self.observe(h);
            left -= h;
        }
    }

    pub fn step_idle(&mut self, dt_us: u64) {
        let inputs = self.inputs.clone();
        self.step(dt_us, &inputs);
    }

    pub(crate) fn next_substep(&self, left: u64) -> u64 {
        let fine = left.min(SUBSTEP_US);
        if self.pace.fine || self.pace.quiet < QUIET_NEEDED || left < 2 * SUBSTEP_US || self.rat.transient() {
            return fine;
        }
        let mut ms = (left / SUBSTEP_US).min(MAX_COARSE_MS);
        if self.pace.drift_per_ms > 1.0e-9 {
            let cap = (DRIFT_BUDGET / self.pace.drift_per_ms) as u64;
            ms = ms.min(cap);
        }
        while ms > 1 && !self.timers_hold(ms * SUBSTEP_US) {
            ms /= 2;
        }
        if ms <= 1 {
            return fine;
        }
        ms * SUBSTEP_US
    }

    fn timers_hold(&self, h: u64) -> bool {
        let now = self.now_us + h;
        let on_ground = self.inputs.on_ground;
        let mut bpcu = self.bpcu.clone();
        let mut bi = self.bpcu_input(on_ground);
        bi.now_us = now;
        if bpcu.step(&bi) != self.bpcu_out {
            return false;
        }
        for m in 0..self.gcus.len() {
            let mut g = self.gcus[m];
            let out = g.step(&self.gcu_input(m, h));
            if !gcu_same(&out, &self.gcu_out[m]) {
                return false;
            }
        }
        for k in 0..self.ext_mon.len() {
            let mut m = self.ext_mon[k];
            if m.step(&self.ext_input(k, h)) != self.ext_out[k] {
                return false;
            }
        }
        let mut lm = self.lm.clone();
        let mut li = self.lm_input(on_ground);
        li.now_us = now;
        if !lm_same(&lm.step(&li), &self.lm_out) {
            return false;
        }
        let mut eicas = self.eicas.clone();
        let mut ci = self.crew_in.clone();
        ci.now_us = now;
        eicas.update(&ci);
        eicas.same_state(&self.eicas)
    }

    pub(crate) fn capture(&mut self) {
        let mut f = std::mem::take(&mut self.pace.scratch);
        f.gcu_out.clone_from(&self.gcu_out);
        f.ext_out.clone_from(&self.ext_out);
        f.bpcu_out = Some(self.bpcu_out);
        f.lm_out = Some(self.lm_out);
        f.bmu_out.clear();
        f.bmu_out.extend_from_slice(&self.bmu_out);
        f.charger_out.clear();
        f.charger_out.extend_from_slice(&self.charger_out);
        f.gear.clear();
        f.gear.extend(self.gear.iter().map(gear_bits));
        self.plant.analog_samples(&mut f.analog);
        f.analog.push(self.lm_out.total_kw);
        f.analog.push(self.pdu.activity());
        for c in &self.charger_out {
            f.analog.extend([c.charge_current_a, c.output_v]);
        }
        for g in &self.gcu_out {
            f.analog.extend([g.exciter_current_a, g.regulated_v]);
        }
        for b in &self.bmu_out {
            f.analog.extend([b.min_cell_v, b.max_cell_v, b.spread_v]);
        }
        f.valid = true;
        self.pace.scratch = f;
    }

    pub(crate) fn observe(&mut self, h: u64) {
        self.capture();
        let p = &self.pace.prev;
        let s = &self.pace.scratch;
        let mut quiet = p.valid
            && gcus_same(&p.gcu_out, &s.gcu_out)
            && p.ext_out == s.ext_out
            && p.bpcu_out == s.bpcu_out
            && lm_opt_same(&p.lm_out, &s.lm_out)
            && bmus_same(&p.bmu_out, &s.bmu_out)
            && chargers_same(&p.charger_out, &s.charger_out)
            && p.gear == s.gear
            && p.analog.len() == s.analog.len();
        let mut drift = 0.0f64;
        if quiet && !regulation_settled(&p.gcu_out, &s.gcu_out, h) {
            quiet = false;
        }
        if quiet {
            for (a, b) in p.analog.iter().zip(s.analog.iter()) {
                drift = drift.max((a - b).abs());
            }
            let per_ms = drift * SUBSTEP_US as f64 / h as f64;
            self.pace.drift_per_ms = per_ms;
            quiet = per_ms < DRIFT_QUIET_PER_MS;
        }
        let settled = self.gear.iter().all(|g| {
            g.cmd == g.out.contacts_closed
                && g.out.armature_closed == g.out.contacts_closed
                && !g.out.bouncing
        });
        if quiet && settled {
            self.pace.quiet += 1;
        } else {
            self.pace.quiet = 0;
        }
        std::mem::swap(&mut self.pace.prev, &mut self.pace.scratch);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn data_dir() -> std::path::PathBuf {
        std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../../aircraft/24-electrical")
    }

    fn observe_all(e: &Electrical) -> Vec<String> {
        let mut out = Vec::new();
        for (name, v) in e.snapshot() {
            match v {
                super::super::SignalValue::Bool(b) => out.push(format!("{name}={b}")),
                super::super::SignalValue::Text(t) => out.push(format!("{name}={t}")),
                super::super::SignalValue::Num(_) => {}
            }
        }
        for n in e.contactor_names() {
            out.push(format!("{n}={:?}", e.contactor_closed(&n)));
        }
        out.push(format!("eicas={:?}", e.eicas_messages()));
        out
    }

    fn engines() -> ElectricalInputs {
        ElectricalInputs {
            engine_rpm: [7600.0; 2],
            engine_running: [true; 2],
            on_ground: false,
            radio_alt_ft: 20000.0,
            ..ElectricalInputs::default()
        }
    }

    fn drive(fine: bool) -> (Vec<Vec<String>>, u64) {
        let mut e = Electrical::new(&data_dir()).expect("system builds");
        e.set_fine_stepping(fine);
        let mut log = Vec::new();
        let advance = |e: &mut Electrical, ms: u64, inputs: &ElectricalInputs, log: &mut Vec<Vec<String>>| {
            let mut left = ms;
            while left > 0 {
                let c = left.min(250);
                e.step(c * 1000, inputs);
                left -= c;
                log.push(observe_all(e));
            }
        };
        let idle = ElectricalInputs::default();
        advance(&mut e, 1_000, &idle, &mut log);
        e.action("crew", "press", "BATTERY", Some("ON")).unwrap();
        advance(&mut e, 2_000, &idle, &mut log);
        for i in 0..4 {
            e.panel_mut().gen_ctrl[i].set_latched(true);
        }
        for i in 0..2 {
            e.panel_mut().apu_gen[i].set_latched(true);
        }
        e.action(
            "ground",
            "connect",
            "EXT_PWR_FWD_L",
            Some("v_ll=115 hz=400 kva=90"),
        )
        .unwrap();
        advance(&mut e, 8_000, &idle, &mut log);
        advance(&mut e, 10_000, &engines(), &mut log);
        e.faults_mut().gen_loss[0] = true;
        advance(&mut e, 5_000, &engines(), &mut log);
        e.faults_mut().gen_loss[0] = false;
        advance(&mut e, 10_000, &engines(), &mut log);
        (log, e.solve_count())
    }

    #[test]
    fn adaptive_matches_fine_stepping_at_checkpoints() {
        let (fine, fine_solves) = drive(true);
        let (fast, fast_solves) = drive(false);
        assert_eq!(fine.len(), fast.len());
        for (k, (a, b)) in fine.iter().zip(fast.iter()).enumerate() {
            assert_eq!(a, b, "checkpoint {k} at {} ms", (k as u64 + 1) * 250);
        }
        assert!(
            fast_solves * 4 < fine_solves,
            "fast {fast_solves} fine {fine_solves}"
        );
    }
}
