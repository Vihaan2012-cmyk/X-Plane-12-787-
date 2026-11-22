use crate::eval::Status;
use crate::runner::{Check, Counts, FactResult, ScenarioResult};

pub type FactSet = (String, Vec<FactResult>);

fn fmt_t(t_us: u64) -> String {
    format!("{:.3}", t_us as f64 / 1_000_000.0)
}

fn cite_of(c: &str) -> &str {
    if c.is_empty() { "none" } else { c }
}

pub fn scenario_line(r: &ScenarioResult) -> String {
    let c = r.counts();
    format!(
        "{} : {} pass, {} fail, {} unbound, {} warn ({} s)",
        r.id,
        c.pass,
        c.fail,
        c.unbound,
        c.warn,
        fmt_t(r.end_us)
    )
}

pub fn check_line(k: &Check) -> String {
    let mut s = format!(
        "  {:<7} t={:>9} {} {} {} expected {} actual {} [{}]",
        k.status.label(),
        fmt_t(k.t_us),
        k.id,
        k.signal,
        k.op,
        k.expected,
        k.actual,
        cite_of(&k.cite)
    );
    if !k.detail.is_empty() {
        s.push_str(&format!(" ({})", k.detail));
    }
    s
}

pub fn fact_line(f: &FactResult) -> String {
    let mut s = format!(
        "  {:<7} {} {} expected {} actual {} [{}]",
        f.status.label(),
        f.id,
        f.quantity,
        f.expected,
        f.actual,
        cite_of(&f.cite)
    );
    if !f.detail.is_empty() {
        s.push_str(&format!(" ({})", f.detail));
    }
    s
}

pub fn fact_counts(list: &[FactResult]) -> Counts {
    let mut c = Counts::default();
    for f in list {
        c.add(f.status);
    }
    c
}

pub fn totals(scenarios: &[ScenarioResult], facts: &[FactSet]) -> (Counts, Counts) {
    let mut sc = Counts::default();
    for r in scenarios {
        sc.merge(&r.counts());
    }
    let mut fc = Counts::default();
    for (_, l) in facts {
        fc.merge(&fact_counts(l));
    }
    (sc, fc)
}

pub fn failed(scenarios: &[ScenarioResult], facts: &[FactSet]) -> bool {
    let (sc, fc) = totals(scenarios, facts);
    sc.fail > 0 || fc.fail > 0
}

pub fn console(scenarios: &[ScenarioResult], facts: &[FactSet], verbose: bool) -> String {
    let mut out = String::new();
    for r in scenarios {
        out.push_str(&scenario_line(r));
        out.push('\n');
        for i in &r.issues {
            out.push_str(&format!(
                "  {} t={} {} {}: {}\n",
                if i.unbound { "UNBOUND" } else { "FAIL" },
                fmt_t(i.t_us),
                i.action,
                i.target,
                i.message
            ));
        }
        for k in &r.checks {
            if verbose || k.status != Status::Pass {
                out.push_str(&check_line(k));
                out.push('\n');
            }
        }
    }
    for (src, list) in facts {
        let c = fact_counts(list);
        out.push_str(&format!(
            "facts {} : {} pass, {} fail, {} unbound, {} warn\n",
            src, c.pass, c.fail, c.unbound, c.warn
        ));
        for f in list {
            if verbose || f.status != Status::Pass {
                out.push_str(&fact_line(f));
                out.push('\n');
            }
        }
    }
    let (sc, fc) = totals(scenarios, facts);
    out.push_str(&format!(
        "TOTAL scenarios: {} pass, {} fail, {} unbound, {} warn; facts: {} pass, {} fail, {} unbound, {} warn\n",
        sc.pass, sc.fail, sc.unbound, sc.warn, fc.pass, fc.fail, fc.unbound, fc.warn
    ));
    out
}

pub fn markdown(target: &str, scenarios: &[ScenarioResult], facts: &[FactSet]) -> String {
    let (sc, fc) = totals(scenarios, facts);
    let mut o = String::new();
    o.push_str("# Test report\n\n");
    o.push_str(&format!("Target: {target}\n\n"));
    o.push_str("| Kind | Pass | Fail | Unbound | Warn |\n|---|---|---|---|---|\n");
    o.push_str(&format!(
        "| Scenario expectations | {} | {} | {} | {} |\n",
        sc.pass, sc.fail, sc.unbound, sc.warn
    ));
    o.push_str(&format!(
        "| Facts | {} | {} | {} | {} |\n\n",
        fc.pass, fc.fail, fc.unbound, fc.warn
    ));
    if !scenarios.is_empty() {
        o.push_str("## Scenarios\n\n| Scenario | Pass | Fail | Unbound | Warn |\n|---|---|---|---|---|\n");
        for r in scenarios {
            let c = r.counts();
            o.push_str(&format!(
                "| {} | {} | {} | {} | {} |\n",
                r.id, c.pass, c.fail, c.unbound, c.warn
            ));
        }
        o.push('\n');
        for r in scenarios {
            let bad: Vec<&Check> = r.checks.iter().filter(|k| k.status != Status::Pass).collect();
            if bad.is_empty() && r.issues.is_empty() {
                continue;
            }
            o.push_str(&format!("### {}\n\n", r.id));
            for i in &r.issues {
                o.push_str(&format!(
                    "- {} t={} {} {}: {}\n",
                    if i.unbound { "UNBOUND" } else { "FAIL" },
                    fmt_t(i.t_us),
                    i.action,
                    i.target,
                    i.message
                ));
            }
            if !bad.is_empty() {
                o.push_str("\n| Status | t (s) | Id | Signal | Op | Expected | Actual | Cite |\n");
                o.push_str("|---|---|---|---|---|---|---|---|\n");
                for k in bad {
                    o.push_str(&format!(
                        "| {} | {} | {} | {} | {} | {} | {} | {} |\n",
                        k.status.label(),
                        fmt_t(k.t_us),
                        k.id,
                        k.signal,
                        k.op,
                        k.expected,
                        k.actual,
                        cite_of(&k.cite)
                    ));
                }
            }
            o.push('\n');
        }
    }
    for (src, list) in facts {
        let c = fact_counts(list);
        o.push_str(&format!(
            "## Facts {}\n\n{} pass, {} fail, {} unbound, {} warn\n\n",
            src, c.pass, c.fail, c.unbound, c.warn
        ));
        o.push_str("| Status | Id | Quantity | Condition | Expected | Actual | Cite |\n");
        o.push_str("|---|---|---|---|---|---|---|\n");
        for f in list.iter().filter(|f| f.status != Status::Pass) {
            o.push_str(&format!(
                "| {} | {} | {} | {} | {} | {} | {} |\n",
                f.status.label(),
                f.id,
                f.quantity,
                f.condition,
                f.expected,
                f.actual,
                cite_of(&f.cite)
            ));
        }
        o.push('\n');
    }
    o
}
