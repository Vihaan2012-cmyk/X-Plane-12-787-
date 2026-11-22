use crate::eval::{Status, Verdict, evaluate, to_status};
use crate::model::{Expect, Fact, Op, Scenario, secs_to_us};
use crate::target::{Target, UNBOUND_PREFIX};
use crate::value::Value;

#[derive(Debug, Clone)]
pub struct Check {
    pub id: String,
    pub t_us: u64,
    pub signal: String,
    pub op: String,
    pub expected: String,
    pub actual: String,
    pub unit: String,
    pub status: Status,
    pub detail: String,
    pub cite: String,
}

#[derive(Debug, Clone)]
pub struct ActionIssue {
    pub t_us: u64,
    pub action: String,
    pub target: String,
    pub message: String,
    pub unbound: bool,
}

#[derive(Debug, Clone, Default)]
pub struct Counts {
    pub pass: usize,
    pub fail: usize,
    pub unbound: usize,
    pub warn: usize,
}

impl Counts {
    pub fn add(&mut self, s: Status) {
        match s {
            Status::Pass => self.pass += 1,
            Status::Fail => self.fail += 1,
            Status::Unbound => self.unbound += 1,
            Status::Warn => self.warn += 1,
        }
    }

    pub fn merge(&mut self, o: &Counts) {
        self.pass += o.pass;
        self.fail += o.fail;
        self.unbound += o.unbound;
        self.warn += o.warn;
    }

    pub fn total(&self) -> usize {
        self.pass + self.fail + self.unbound + self.warn
    }
}

#[derive(Debug, Clone)]
pub struct ScenarioResult {
    pub id: String,
    pub title: String,
    pub source: String,
    pub checks: Vec<Check>,
    pub issues: Vec<ActionIssue>,
    pub end_us: u64,
}

impl ScenarioResult {
    pub fn counts(&self) -> Counts {
        let mut c = Counts::default();
        for k in &self.checks {
            c.add(k.status);
        }
        for i in &self.issues {
            if !i.unbound {
                c.fail += 1;
            }
        }
        c
    }
}

#[derive(Debug, Clone)]
pub struct RunOptions {
    pub fast_forward_us: u64,
    pub min_dt_us: u64,
    pub fine: bool,
}

impl Default for RunOptions {
    fn default() -> RunOptions {
        RunOptions {
            fast_forward_us: 50_000,
            min_dt_us: 0,
            fine: false,
        }
    }
}

#[derive(Clone, Copy)]
enum Event {
    Expect(usize),
    Step(usize),
}

fn describe_expected(e: &Expect) -> String {
    match &e.value {
        Some(v) if e.tol > 0.0 => format!("{v} +/- {}", e.tol),
        Some(v) => v.to_string(),
        None => String::new(),
    }
}

fn check_expect(target: &dyn Target, e: &Expect) -> Check {
    let actual = target.read(&e.signal);
    let verdict = evaluate(e.op, e.value.as_ref(), e.tol, actual.as_ref());
    let status = to_status(&verdict, e.approx);
    let detail = match &verdict {
        Verdict::Fail(m) => m.clone(),
        Verdict::Unbound => "signal not bound".to_string(),
        Verdict::Pass => String::new(),
    };
    Check {
        id: e.id.clone(),
        t_us: e.t_us,
        signal: e.signal.clone(),
        op: e.op.name().to_string(),
        expected: describe_expected(e),
        actual: actual.map(|v| v.to_string()).unwrap_or_else(|| "-".to_string()),
        unit: e.unit.clone(),
        status,
        detail,
        cite: e.cite.clone(),
    }
}

fn is_advance(action: &str, actor: &str, target: &str) -> bool {
    actor == "sim" && action == "advance" && target == "clock_s"
}

pub fn run_scenario(
    target: &mut dyn Target,
    sc: &Scenario,
    source: &str,
    opts: &RunOptions,
) -> ScenarioResult {
    let mut issues = Vec::new();
    for (k, v) in &sc.initial {
        if let Err(m) = target.apply("sim", "initial", k, Some(v)) {
            issues.push(ActionIssue {
                t_us: 0,
                action: "initial".to_string(),
                target: k.clone(),
                unbound: m.starts_with(UNBOUND_PREFIX),
                message: m,
            });
        }
    }

    let mut events: Vec<(u64, u8, usize, Event)> = Vec::new();
    for (i, e) in sc.expects.iter().enumerate() {
        events.push((e.t_us, 0, i, Event::Expect(i)));
    }
    for (i, s) in sc.steps.iter().enumerate() {
        events.push((s.t_us, 1, i, Event::Step(i)));
    }
    events.sort_by_key(|e| (e.0, e.1, e.2));

    let mut windows: Vec<(u64, u64)> = Vec::new();
    for s in &sc.steps {
        if is_advance(&s.action, &s.actor, &s.target)
            && let Some(secs) = s.value.as_ref().and_then(Value::as_f64)
        {
            let len = secs_to_us(secs);
            if len > 0 {
                windows.push((s.t_us, s.t_us + len));
            }
        }
    }

    let end_us = events.last().map(|e| e.0).unwrap_or(0);
    let mut breaks: Vec<u64> = events.iter().map(|e| e.0).collect();
    for w in &windows {
        breaks.push(w.0);
        breaks.push(w.1);
    }
    breaks.sort_unstable();
    breaks.dedup();

    target.set_fine(opts.fine);
    let base_dt = (1_000_000 / u64::from(sc.tick_hz.max(1)))
        .max(opts.min_dt_us)
        .max(1);
    let mut results: Vec<Option<Check>> = vec![None; sc.expects.len()];
    let mut t: u64 = 0;
    let mut cursor = 0usize;
    let mut bi = 0usize;

    loop {
        while cursor < events.len() && events[cursor].0 == t {
            match events[cursor].3 {
                Event::Expect(i) => results[i] = Some(check_expect(target, &sc.expects[i])),
                Event::Step(i) => {
                    let s = &sc.steps[i];
                    if !is_advance(&s.action, &s.actor, &s.target)
                        && let Err(m) = target.apply(&s.actor, &s.action, &s.target, s.value.as_ref())
                    {
                        issues.push(ActionIssue {
                            t_us: t,
                            action: s.action.clone(),
                            target: s.target.clone(),
                            unbound: m.starts_with(UNBOUND_PREFIX),
                            message: m,
                        });
                    }
                }
            }
            cursor += 1;
        }
        if t >= end_us {
            break;
        }
        while bi < breaks.len() && breaks[bi] <= t {
            bi += 1;
        }
        let next_break = breaks.get(bi).copied().unwrap_or(end_us).min(end_us);
        let in_window = windows.iter().any(|w| t >= w.0 && t < w.1);
        let dt = if opts.fine || !target.adaptive() {
            if in_window {
                opts.fast_forward_us.max(base_dt)
            } else {
                base_dt
            }
        } else {
            u64::MAX
        };
        let next = t.saturating_add(dt).min(next_break);
        target.step(next - t);
        t = next;
    }

    let mut checks: Vec<Check> = results.into_iter().flatten().collect();
    checks.sort_by_key(|c| c.t_us);
    ScenarioResult {
        id: sc.id.clone(),
        title: sc.title.clone(),
        source: source.to_string(),
        checks,
        issues,
        end_us,
    }
}

#[derive(Debug, Clone)]
pub struct FactResult {
    pub id: String,
    pub quantity: String,
    pub condition: String,
    pub expected: String,
    pub actual: String,
    pub unit: String,
    pub status: Status,
    pub detail: String,
    pub cite: String,
}

pub fn run_fact(target: &dyn Target, f: &Fact) -> FactResult {
    let actual = target.fact(&f.quantity, &f.condition);
    let op = match &f.expected {
        Value::Num(_) => Op::Approx,
        _ => Op::Eq,
    };
    let verdict = evaluate(op, Some(&f.expected), f.tol, actual.as_ref());
    let detail = match &verdict {
        Verdict::Fail(m) => m.clone(),
        Verdict::Unbound => "quantity not bound".to_string(),
        Verdict::Pass => String::new(),
    };
    FactResult {
        id: f.id.clone(),
        quantity: f.quantity.clone(),
        condition: f.condition.clone(),
        expected: if f.tol > 0.0 {
            format!("{} +/- {}", f.expected, f.tol)
        } else {
            f.expected.to_string()
        },
        actual: actual.map(|v| v.to_string()).unwrap_or_else(|| "-".to_string()),
        unit: f.unit.clone(),
        status: to_status(&verdict, f.approx),
        detail,
        cite: f.cite.clone(),
    }
}

pub fn run_facts(target: &dyn Target, facts: &[Fact]) -> Vec<FactResult> {
    facts.iter().map(|f| run_fact(target, f)).collect()
}
