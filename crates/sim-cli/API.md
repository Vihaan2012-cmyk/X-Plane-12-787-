# b789-sim-cli (lib `sim_cli`, bin `sim-cli`)

Scenario and fact runner plus tablet PNG render. Host crate; may use external crates (toml).

## CLI
- `sim-cli scenario <file|dir>...` run scenario TOML (dirs searched recursively).
- `sim-cli facts <file>...` run fact TOML.
- `sim-cli all` run `tests/scenarios/**` and `tests/facts/*` (found by walking up from cwd, or `--root <tests dir>`).
- `sim-cli render <out.png>` render the tablet (demo electrical page).
- Options: `--report <path>` markdown report, `--target mock|electrical`, `--data-dir <dir>` (electrical data, `aircraft/24-electrical`), `--ff-ms <n>` fast-forward step (default 50), `--min-dt-ms <n>` floor on the tick dt (full-rate electrical runs cost ~130 us per tick), `-v` list passes.
- Exit: 0 unless a FAIL; 2 on usage or load error. Default target is `mock` (all UNBOUND); `electrical` when built with `--features electrical`.

## Semantics
- Time is integer microseconds. Tick dt = 1e6 / tick_hz; steps are split so every step and expectation lands on its exact time.
- At equal time: expectations read first, then steps apply. Initial table applied before t=0 as `apply("sim","initial",key,value)`.
- `sim advance clock_s V` at t: runner steps coarse (`--ff-ms`) over [t, t+V); it is not forwarded to the target.
- Status: PASS, FAIL (non-approx miss; action error not prefixed `unbound:`), UNBOUND (target returns None or an `unbound: ...` error), WARN (approx=true miss).
- Ops: true, false, eq (tol, string and bool exact), approx, within, lt, gt, present, absent. Numbers are truthy when nonzero.

## Library
- `trait Target { name, apply(actor, action, target, Option<&Value>) -> Result<(), String>, read(signal) -> Option<Value>, step(dt_us), fact(quantity, condition) -> Option<Value> (default read) }`.
- `Value { Bool, Num, Str, List, Table }`; `target::unbound(msg)` builds an unbound error.
- `MockTarget`: `with_signal`, `with_fact`, `with_rule(action, target, signal, value)`; reads `mock.time_us`; records `applied()` and `step_sizes()`.
- `model::{parse_scenario, parse_facts, load_*, collect_files}`, `runner::{run_scenario, run_facts}`, `report::{console, markdown, failed}`.
- Feature `electrical`: `ElectricalTarget::new(data_dir)` over `sys_electrical::system::Electrical` (`value`, `action`, `step_idle`); signals read with or without the `elec.` prefix; action errors map to UNBOUND.
