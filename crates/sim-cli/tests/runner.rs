use sim_cli::eval::Status;
use sim_cli::mock::MockTarget;
use sim_cli::model::{collect_files, load_facts, load_scenario, parse_scenario};
use sim_cli::report;
use sim_cli::runner::{RunOptions, run_facts, run_scenario};
use sim_cli::target::unbound;
use sim_cli::{Target, Value};
use std::path::{Path, PathBuf};

fn repo_tests() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("..")
        .join("tests")
}

const OPS: &str = r#"
[scenario]
id = "t"
title = "ops"
tick_hz = 1000

[initial]
airplane_state = "cold_dark"

[[step]]
t_s = 1.0
actor = "crew"
action = "press"
target = "BATTERY"
value = "ON"

[[expect]]
id = "t.true"
t_s = 0.5
signal = "s.b"
op = "true"

[[expect]]
id = "t.false"
t_s = 0.5
signal = "s.b2"
op = "false"

[[expect]]
id = "t.eq"
t_s = 0.5
signal = "s.str"
op = "eq"
value = "mode"

[[expect]]
id = "t.approx"
t_s = 0.5
signal = "s.n"
op = "approx"
value = 10.0
tol = 0.5

[[expect]]
id = "t.within"
t_s = 0.5
signal = "s.n"
op = "within"
value = 9.0
tol = 2.0

[[expect]]
id = "t.lt"
t_s = 0.5
signal = "s.n"
op = "lt"
value = 11.0

[[expect]]
id = "t.gt"
t_s = 0.5
signal = "s.n"
op = "gt"
value = 9.0

[[expect]]
id = "t.present"
t_s = 0.5
signal = "s.b"
op = "present"

[[expect]]
id = "t.absent"
t_s = 0.5
signal = "s.b2"
op = "absent"

[[expect]]
id = "t.unbound"
t_s = 0.5
signal = "s.nothing"
op = "true"

[[expect]]
id = "t.fail"
t_s = 0.5
signal = "s.n"
op = "gt"
value = 50.0
approx = false
cite = "x.md:1"

[[expect]]
id = "t.warn"
t_s = 0.5
signal = "s.n"
op = "gt"
value = 50.0
approx = true
cite = "none"
"#;

fn ops_target() -> MockTarget {
    MockTarget::new()
        .with_signal("s.b", Value::Bool(true))
        .with_signal("s.b2", Value::Bool(false))
        .with_signal("s.str", Value::Str("mode".into()))
        .with_signal("s.n", Value::Num(10.2))
}

fn status_of(r: &sim_cli::runner::ScenarioResult, id: &str) -> Status {
    r.checks.iter().find(|c| c.id == id).unwrap().status
}

#[test]
fn every_op_on_mock() {
    let sc = parse_scenario(OPS).unwrap();
    let mut t = ops_target();
    let r = run_scenario(&mut t, &sc, "inline", &RunOptions::default());
    for id in [
        "t.true",
        "t.false",
        "t.eq",
        "t.approx",
        "t.within",
        "t.lt",
        "t.gt",
        "t.present",
        "t.absent",
    ] {
        assert_eq!(status_of(&r, id), Status::Pass, "{id}");
    }
    assert_eq!(status_of(&r, "t.unbound"), Status::Unbound);
    assert_eq!(status_of(&r, "t.fail"), Status::Fail);
    assert_eq!(status_of(&r, "t.warn"), Status::Warn);
    let c = r.counts();
    assert_eq!(c.fail, 1);
    assert_eq!(c.unbound, 1);
    assert_eq!(c.warn, 1);
    assert!(report::failed(&[r], &[]));
}

#[test]
fn warn_and_unbound_do_not_fail() {
    let text = r#"
[scenario]
id = "w"
[[expect]]
t_s = 0.1
signal = "a"
op = "true"
approx = true
cite = "none"
[[expect]]
t_s = 0.1
signal = "missing"
op = "true"
"#;
    let sc = parse_scenario(text).unwrap();
    let mut t = MockTarget::new().with_signal("a", Value::Bool(false));
    let r = run_scenario(&mut t, &sc, "inline", &RunOptions::default());
    assert_eq!(r.checks[0].status, Status::Warn);
    assert_eq!(r.checks[1].status, Status::Unbound);
    assert!(!report::failed(&[r], &[]));
}

#[test]
fn steps_land_at_exact_times() {
    let text = r#"
[scenario]
id = "timing"
tick_hz = 1000
[[step]]
t_s = 1.2345
actor = "crew"
action = "press"
target = "X"
value = "ON"
[[step]]
t_s = 0.0
actor = "ground"
action = "set"
target = "Y"
[[expect]]
id = "before"
t_s = 1.234
signal = "mock.time_us"
op = "eq"
value = 1234000
[[expect]]
id = "after"
t_s = 2.0
signal = "mock.time_us"
op = "eq"
value = 2000000
"#;
    let sc = parse_scenario(text).unwrap();
    let mut t = MockTarget::new();
    let r = run_scenario(&mut t, &sc, "inline", &RunOptions::default());
    assert!(r.checks.iter().all(|c| c.status == Status::Pass));
    let applied = t.applied();
    assert_eq!(applied.iter().find(|a| a.target == "X").unwrap().t_us, 1_234_500);
    assert_eq!(applied.iter().find(|a| a.target == "Y").unwrap().t_us, 0);
    assert_eq!(t.time_us(), 2_000_000);
    assert_eq!(t.step_sizes().iter().sum::<u64>(), 2_000_000);
    assert!(t.step_sizes().iter().all(|d| *d <= 1000));
}

#[test]
fn expect_sees_state_before_same_time_step() {
    let text = r#"
[scenario]
id = "order"
[[step]]
t_s = 1.0
actor = "crew"
action = "press"
target = "SW"
[[expect]]
id = "same"
t_s = 1.0
signal = "lamp"
op = "false"
[[expect]]
id = "later"
t_s = 1.001
signal = "lamp"
op = "true"
"#;
    let sc = parse_scenario(text).unwrap();
    let mut t = MockTarget::new()
        .with_signal("lamp", Value::Bool(false))
        .with_rule("press", "SW", "lamp", Value::Bool(true));
    let r = run_scenario(&mut t, &sc, "inline", &RunOptions::default());
    assert!(
        r.checks.iter().all(|c| c.status == Status::Pass),
        "{:?}",
        r.checks
    );
}

#[test]
fn advance_fast_forwards_with_coarse_steps() {
    let text = r#"
[scenario]
id = "ff"
tick_hz = 1000
[[step]]
t_s = 1.0
actor = "sim"
action = "advance"
target = "clock_s"
value = 10.0
[[expect]]
id = "e"
t_s = 20.0
signal = "mock.time_us"
op = "eq"
value = 20000000
"#;
    let sc = parse_scenario(text).unwrap();
    let mut t = MockTarget::new();
    let r = run_scenario(&mut t, &sc, "inline", &RunOptions::default());
    assert_eq!(r.checks[0].status, Status::Pass);
    assert!(t.step_sizes().len() < 20_000 - 9_000);
    assert!(t.applied().iter().all(|a| a.action != "advance"));
}

#[test]
fn action_errors_are_reported() {
    let text = r#"
[scenario]
id = "err"
[[step]]
t_s = 0.1
actor = "fault"
action = "reject"
target = "Q"
[[expect]]
t_s = 0.2
signal = "a"
op = "true"
"#;
    let sc = parse_scenario(text).unwrap();
    let mut t = MockTarget::new().with_signal("a", Value::Bool(true));
    let r = run_scenario(&mut t, &sc, "inline", &RunOptions::default());
    assert_eq!(r.issues.len(), 1);
    assert!(!r.issues[0].unbound);
    assert_eq!(r.counts().fail, 1);
    assert!(unbound("x").starts_with("unbound:"));
}

#[test]
fn initial_preset_reaches_target() {
    let sc = parse_scenario(OPS).unwrap();
    let mut t = ops_target();
    let _ = run_scenario(&mut t, &sc, "inline", &RunOptions::default());
    assert_eq!(
        t.read("initial.airplane_state"),
        Some(Value::Str("cold_dark".into()))
    );
}

#[test]
fn parses_every_scenario_and_fact_file() {
    let root = repo_tests();
    let mut files = Vec::new();
    collect_files(&root.join("scenarios"), "toml", &mut files).unwrap();
    assert!(files.len() >= 16);
    let mut expects = 0;
    let mut steps = 0;
    for f in &files {
        let sc = load_scenario(f).unwrap();
        expects += sc.expects.len();
        steps += sc.steps.len();
    }
    assert!(expects >= 259, "expects {expects}");
    assert!(steps >= 58, "steps {steps}");
    let mut facts = Vec::new();
    collect_files(&root.join("facts"), "toml", &mut facts).unwrap();
    let mut n = 0;
    for f in &facts {
        n += load_facts(f).unwrap().len();
    }
    let mut pending = Vec::new();
    collect_files(&root.join("facts-pending"), "toml", &mut pending).unwrap();
    for f in &pending {
        n += load_facts(f).unwrap().len();
    }
    assert!(n >= 284, "facts {n}");
}

#[test]
fn all_scenarios_run_on_mock_without_failure() {
    let root = repo_tests();
    let mut files = Vec::new();
    collect_files(&root.join("scenarios"), "toml", &mut files).unwrap();
    for f in &files {
        let sc = load_scenario(f).unwrap();
        let mut t = MockTarget::new();
        let r = run_scenario(&mut t, &sc, &f.display().to_string(), &RunOptions::default());
        assert_eq!(r.checks.len(), sc.expects.len());
        let c = r.counts();
        assert_eq!(c.fail, 0, "{}", sc.id);
    }
}

#[test]
fn facts_on_mock() {
    let text = r#"
[[fact]]
id = "F1"
quantity = "q.a"
expected = 4
tol = 0
cite = "a:1"
[[fact]]
id = "F2"
quantity = "q.b"
expected = 1.5
tol = 0.1
[[fact]]
id = "F3"
quantity = "q.c"
expected = 3
[[fact]]
id = "F4"
quantity = "q.d"
expected = 3
approx = true
"#;
    let facts = sim_cli::model::parse_facts(text).unwrap();
    let t = MockTarget::new()
        .with_fact("q.a", Value::Num(4.0))
        .with_fact("q.b", Value::Num(1.55))
        .with_fact("q.d", Value::Num(9.0));
    let r = run_facts(&t, &facts);
    assert_eq!(r[0].status, Status::Pass);
    assert_eq!(r[1].status, Status::Pass);
    assert_eq!(r[2].status, Status::Unbound);
    assert_eq!(r[3].status, Status::Warn);
}

#[test]
fn report_formatting() {
    let sc = parse_scenario(OPS).unwrap();
    let mut t = ops_target();
    let r = run_scenario(&mut t, &sc, "inline", &RunOptions::default());
    let text = report::console(std::slice::from_ref(&r), &[], false);
    assert!(text.contains("t : 9 pass, 1 fail, 1 unbound, 1 warn"));
    assert!(text.contains("FAIL    t=    0.500 t.fail s.n gt expected 50 actual 10.2 [x.md:1]"));
    assert!(text.contains("UNBOUND"));
    assert!(text.contains("WARN"));
    assert!(text.contains("[none]"));
    assert!(!text.contains("PASS "));
    let verbose = report::console(std::slice::from_ref(&r), &[], true);
    assert!(verbose.contains("PASS"));
    let md = report::markdown("mock", &[r], &[]);
    assert!(md.starts_with("# Test report"));
    assert!(md.contains("| t | 9 | 1 | 1 | 1 |"));
    assert!(md.contains("| FAIL | 0.500 | t.fail |"));
}

#[test]
fn bad_input_is_rejected() {
    assert!(
        parse_scenario("[scenario]\nid = \"x\"\n[[expect]]\nt_s = 1\nsignal = \"a\"\nop = \"bogus\"\n")
            .is_err()
    );
    assert!(parse_scenario("[[step]]\nt_s = 1\n").is_err());
    assert!(
        parse_scenario("[scenario]\nid = \"x\"\n[[expect]]\nt_s = 1\nsignal = \"a\"\nop = \"eq\"\n").is_err()
    );
}

#[test]
fn cli_args_and_render() {
    let args: Vec<String> = ["scenario", "a", "b", "--report", "r.md", "-v"]
        .iter()
        .map(|s| s.to_string())
        .collect();
    let o = sim_cli::cli::parse_args(&args).unwrap();
    assert_eq!(o.command, "scenario");
    assert_eq!(o.paths.len(), 2);
    assert!(o.verbose);
    assert!(sim_cli::cli::parse_args(&[]).is_err());
    let dir = std::env::temp_dir().join("b789_sim_cli_render_test.png");
    sim_cli::cli::render_tablet(&dir).unwrap();
    let bytes = std::fs::read(&dir).unwrap();
    assert_eq!(&bytes[1..4], b"PNG");
    let _ = std::fs::remove_file(&dir);
}
