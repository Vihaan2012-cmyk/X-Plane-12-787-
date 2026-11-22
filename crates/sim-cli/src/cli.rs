use crate::mock::MockTarget;
use crate::model::{collect_files, load_facts, load_scenario};
use crate::report;
use crate::runner::{FactResult, RunOptions, ScenarioResult, run_facts, run_scenario};
use crate::target::Target;
use std::fs;
use std::path::{Path, PathBuf};

pub struct Options {
    pub command: String,
    pub paths: Vec<PathBuf>,
    pub report: Option<PathBuf>,
    pub target: String,
    pub data_dir: PathBuf,
    pub root: Option<PathBuf>,
    pub verbose: bool,
    pub fast_forward_us: u64,
    pub min_dt_us: u64,
    pub fine: bool,
    pub seeds: u64,
    pub start_seed: u64,
    pub sim_s: u64,
    pub determinism: bool,
}

pub fn parse_args(args: &[String]) -> Result<Options, String> {
    let mut o = Options {
        command: String::new(),
        paths: Vec::new(),
        report: None,
        target: if cfg!(feature = "electrical") {
            "electrical"
        } else {
            "mock"
        }
        .to_string(),
        data_dir: PathBuf::from("data"),
        root: None,
        verbose: false,
        fast_forward_us: RunOptions::default().fast_forward_us,
        min_dt_us: 0,
        fine: false,
        seeds: 20,
        start_seed: 1,
        sim_s: 600,
        determinism: false,
    };
    let mut it = args.iter();
    while let Some(a) = it.next() {
        match a.as_str() {
            "--report" => o.report = Some(PathBuf::from(it.next().ok_or("--report needs a path")?)),
            "--target" => o.target = it.next().ok_or("--target needs a name")?.clone(),
            "--data-dir" => o.data_dir = PathBuf::from(it.next().ok_or("--data-dir needs a path")?),
            "--root" => o.root = Some(PathBuf::from(it.next().ok_or("--root needs a path")?)),
            "--ff-ms" => {
                let v: u64 = it
                    .next()
                    .ok_or("--ff-ms needs a number")?
                    .parse()
                    .map_err(|_| "--ff-ms needs an integer".to_string())?;
                o.fast_forward_us = v * 1000;
            }
            "--min-dt-ms" => {
                let v: u64 = it
                    .next()
                    .ok_or("--min-dt-ms needs a number")?
                    .parse()
                    .map_err(|_| "--min-dt-ms needs an integer".to_string())?;
                o.min_dt_us = v * 1000;
            }
            "--fine" => o.fine = true,
            "--determinism" => o.determinism = true,
            "--seeds" | "--start-seed" | "--sim-s" => {
                let v: u64 = it
                    .next()
                    .ok_or(format!("{a} needs a number"))?
                    .parse()
                    .map_err(|_| format!("{a} needs an integer"))?;
                match a.as_str() {
                    "--seeds" => o.seeds = v,
                    "--start-seed" => o.start_seed = v,
                    _ => o.sim_s = v,
                }
            }
            "-v" | "--verbose" => o.verbose = true,
            s if s.starts_with("--") => return Err(format!("unknown option {s}")),
            s => {
                if o.command.is_empty() {
                    o.command = s.to_string();
                } else {
                    o.paths.push(PathBuf::from(s));
                }
            }
        }
    }
    if o.command.is_empty() {
        return Err("no command".to_string());
    }
    Ok(o)
}

pub fn usage() -> &'static str {
    "usage: sim-cli <scenario <file|dir>... | facts <file>... | all | render <out.png>> [--report <path>] [--target mock|electrical] [--data-dir <dir>] [--root <dir>] [--ff-ms <n>] [--min-dt-ms <n>] [--fine] [-v]"
}

fn make_target(o: &Options) -> Result<Box<dyn Target>, String> {
    match o.target.as_str() {
        "mock" => Ok(Box::new(MockTarget::new())),
        #[cfg(feature = "electrical")]
        "electrical" => Ok(Box::new(crate::electrical::ElectricalTarget::new(&o.data_dir)?)),
        other => Err(format!("unknown target '{other}'")),
    }
}

pub fn find_tests_root(start: &Path) -> Option<PathBuf> {
    let mut cur = Some(start);
    while let Some(d) = cur {
        if d.join("tests").join("scenarios").is_dir() {
            return Some(d.join("tests"));
        }
        cur = d.parent();
    }
    None
}

pub fn run_scenario_files(o: &Options, files: &[PathBuf]) -> Result<Vec<ScenarioResult>, String> {
    let mut out = Vec::new();
    for f in files {
        let sc = load_scenario(f)?;
        let mut t = make_target(o)?;
        let opts = RunOptions {
            fast_forward_us: o.fast_forward_us,
            min_dt_us: o.min_dt_us,
            fine: o.fine,
        };
        out.push(run_scenario(t.as_mut(), &sc, &f.display().to_string(), &opts));
    }
    Ok(out)
}

pub fn run_fact_files(o: &Options, files: &[PathBuf]) -> Result<Vec<(String, Vec<FactResult>)>, String> {
    let mut out = Vec::new();
    for f in files {
        let facts = load_facts(f)?;
        let t = make_target(o)?;
        out.push((f.display().to_string(), run_facts(t.as_ref(), &facts)));
    }
    Ok(out)
}

pub fn render_tablet(path: &Path) -> Result<(), String> {
    let mut tablet = tablet::Tablet::new();
    tablet.register(Box::new(tablet::DemoElectricalPage::new()));
    let canvas = tablet.render();
    render::write_png(path, canvas.width(), canvas.height(), canvas.rgba())
        .map_err(|e| format!("{}: {e}", path.display()))
}

fn execute(o: &Options) -> Result<i32, String> {
    #[cfg(feature = "electrical")]
    if o.command == "stress" {
        let seeds: Vec<u64> = (o.start_seed..o.start_seed + o.seeds).collect();
        let threads = std::thread::available_parallelism()
            .map(|n| n.get())
            .unwrap_or(4)
            .max(1);
        let mut reports: Vec<crate::stress::StressReport> = Vec::new();
        for chunk in seeds.chunks(threads) {
            let mut part: Vec<crate::stress::StressReport> = std::thread::scope(|sc| {
                let hs: Vec<_> = chunk
                    .iter()
                    .map(|&seed| {
                        let dir = o.data_dir.clone();
                        let (sim_s, det) = (o.sim_s, o.determinism);
                        sc.spawn(move || crate::stress::run_seed(&dir, seed, sim_s, det))
                    })
                    .collect();
                hs.into_iter().filter_map(|h| h.join().ok()).collect()
            });
            reports.append(&mut part);
        }
        let text = crate::stress::summary(&reports);
        print!("{text}");
        if let Some(p) = &o.report {
            fs::write(p, &text).map_err(|e| format!("{}: {e}", p.display()))?;
        }
        let failed = reports
            .iter()
            .any(|r| r.panicked.is_some() || !r.violations.is_empty() || r.deterministic == Some(false));
        return Ok(if failed { 1 } else { 0 });
    }
    #[cfg(feature = "electrical")]
    if o.command == "topology" {
        let out = o.paths.first().ok_or("topology needs an output file")?;
        let net = sys_electrical::net::ElectricalNet::from_data(&o.data_dir)?;
        let mut text = String::from("kind,name,a,b,value\n");
        for r in net.topology() {
            let cells: Vec<String> = r.iter().map(|c| format!("\"{}\"", c.replace('"', "'"))).collect();
            text.push_str(&cells.join(","));
            text.push('\n');
        }
        fs::write(out, text).map_err(|e| format!("{}: {e}", out.display()))?;
        println!("wrote {}", out.display());
        return Ok(0);
    }
    #[cfg(feature = "electrical")]
    if o.command == "render" && o.target == "electrical" {
        let out = o.paths.first().ok_or("render needs an output directory")?;
        for p in crate::viewer::render_viewer(out, &o.data_dir, o.paths.get(1).map(|p| p.as_path()))? {
            println!("wrote {}", p.display());
        }
        return Ok(0);
    }
    if o.command == "render" {
        let p = o.paths.first().ok_or("render needs an output path")?;
        render_tablet(p)?;
        println!("wrote {}", p.display());
        return Ok(0);
    }
    let mut scenario_files = Vec::new();
    let mut fact_files = Vec::new();
    match o.command.as_str() {
        "scenario" => {
            if o.paths.is_empty() {
                return Err("scenario needs a file or directory".to_string());
            }
            for p in &o.paths {
                collect_files(p, "toml", &mut scenario_files)?;
            }
        }
        "facts" => {
            if o.paths.is_empty() {
                return Err("facts needs a file".to_string());
            }
            for p in &o.paths {
                collect_files(p, "toml", &mut fact_files)?;
            }
        }
        "all" => {
            let root = match &o.root {
                Some(r) => r.clone(),
                None => {
                    let cwd = std::env::current_dir().map_err(|e| e.to_string())?;
                    find_tests_root(&cwd).ok_or("tests/scenarios not found; use --root")?
                }
            };
            collect_files(&root.join("scenarios"), "toml", &mut scenario_files)?;
            if root.join("facts").is_dir() {
                collect_files(&root.join("facts"), "toml", &mut fact_files)?;
            }
        }
        other => return Err(format!("unknown command '{other}'")),
    }
    let scenarios = run_scenario_files(o, &scenario_files)?;
    let facts = run_fact_files(o, &fact_files)?;
    print!("{}", report::console(&scenarios, &facts, o.verbose));
    if let Some(p) = &o.report {
        let tname = make_target(o)?.name().to_string();
        fs::write(p, report::markdown(&tname, &scenarios, &facts))
            .map_err(|e| format!("{}: {e}", p.display()))?;
        println!("report written to {}", p.display());
    }
    Ok(if report::failed(&scenarios, &facts) { 1 } else { 0 })
}

pub fn run(args: &[String]) -> i32 {
    let o = match parse_args(args) {
        Ok(o) => o,
        Err(e) => {
            eprintln!("{e}\n{}", usage());
            return 2;
        }
    };
    match execute(&o) {
        Ok(code) => code,
        Err(e) => {
            eprintln!("error: {e}");
            2
        }
    }
}
