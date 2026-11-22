//! Project checks.
//!
//! `cargo xtask layers` checks that local crate dependencies only point the allowed way:
//! systems -> solvers -> core, tablet -> core, and only the hosts (sim-cli, xp-host) see everything.
//! `cargo xtask check` runs the layering check, then fmt, clippy and the tests.

use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, ExitCode};

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Layer {
    Core,
    Solver,
    System,
    Tablet,
    Host,
    Tool,
}

fn layer_of(path: &str) -> Option<Layer> {
    let p = path.replace('\\', "/");
    if p.starts_with("crates/core/") {
        Some(Layer::Core)
    } else if p.starts_with("crates/solvers/") {
        Some(Layer::Solver)
    } else if p.starts_with("crates/systems/") {
        Some(Layer::System)
    } else if p == "crates/tablet" {
        Some(Layer::Tablet)
    } else if p == "crates/sim-cli" || p == "crates/xp-host" {
        Some(Layer::Host)
    } else if p.starts_with("tools/") {
        Some(Layer::Tool)
    } else {
        None
    }
}

/// May a crate in `from` depend on `to` (named `dep`)?
fn allowed(from: Layer, to: Layer, dep: &str) -> bool {
    use Layer::*;
    match from {
        Core => to == Core,
        // Solvers know nothing about the aircraft; they may share only the common maths crate.
        Solver => to == Core || (to == Solver && dep == "b789-solver-common"),
        // Systems never depend on each other.
        System => to == Core || to == Solver,
        Tablet => to == Core,
        Host => to != Tool,
        Tool => false,
    }
}

fn root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()
        .expect("workspace root")
}

/// Local packages from `[workspace.dependencies]` in the root manifest: name -> relative path.
fn local_packages(root: &Path) -> BTreeMap<String, String> {
    let text = fs::read_to_string(root.join("Cargo.toml")).expect("root Cargo.toml");
    let mut out = BTreeMap::new();
    let mut in_deps = false;
    for line in text.lines() {
        let l = line.trim();
        if l.starts_with('[') {
            in_deps = l == "[workspace.dependencies]";
            continue;
        }
        if !in_deps {
            continue;
        }
        if let (Some((name, _)), Some(start)) = (l.split_once('='), l.find("path = \"")) {
            let rest = &l[start + 8..];
            if let Some(end) = rest.find('"') {
                out.insert(name.trim().to_string(), rest[..end].to_string());
            }
        }
    }
    out
}

/// Names of local (`b789-`) crates a manifest depends on, in any dependency section.
fn local_deps(manifest: &str) -> Vec<String> {
    let mut out = Vec::new();
    let mut in_deps = false;
    for line in manifest.lines() {
        let l = line.trim();
        if l.starts_with('[') {
            in_deps = l.ends_with("dependencies]");
            continue;
        }
        if in_deps && l.starts_with("b789-") {
            let name: String = l
                .chars()
                .take_while(|c| c.is_ascii_alphanumeric() || *c == '-')
                .collect();
            out.push(name);
        }
    }
    out
}

fn layers() -> bool {
    let root = root();
    let pkgs = local_packages(&root);
    let mut members: Vec<(String, String)> = pkgs.iter().map(|(n, p)| (n.clone(), p.clone())).collect();
    members.push(("b789-sim-cli".into(), "crates/sim-cli".into()));
    members.push(("b789-xp-host".into(), "crates/xp-host".into()));
    members.push(("xtask".into(), "tools/xtask".into()));
    let mut ok = true;
    let mut edges = 0;
    for (name, path) in &members {
        let Some(from) = layer_of(path) else {
            eprintln!("layers: {name} at {path} is not in a known layer folder");
            ok = false;
            continue;
        };
        let manifest = fs::read_to_string(root.join(path).join("Cargo.toml")).unwrap_or_default();
        for dep in local_deps(&manifest) {
            edges += 1;
            let Some(to) = pkgs.get(&dep).and_then(|p| layer_of(p)) else {
                eprintln!("layers: {name} depends on {dep}, which is not a known local crate");
                ok = false;
                continue;
            };
            if !allowed(from, to, &dep) {
                eprintln!("layers: {name} ({from:?}) may not depend on {dep} ({to:?})");
                ok = false;
            }
        }
    }
    println!(
        "layers: {} crates, {edges} local dependencies, {}",
        members.len(),
        if ok { "ok" } else { "FAILED" }
    );
    ok
}

fn cargo(args: &[&str]) -> bool {
    let cargo = std::env::var("CARGO").unwrap_or_else(|_| "cargo".into());
    println!("> cargo {}", args.join(" "));
    Command::new(cargo)
        .args(args)
        .current_dir(root())
        .status()
        .map(|s| s.success())
        .unwrap_or(false)
}

fn main() -> ExitCode {
    let cmd = std::env::args().nth(1).unwrap_or_default();
    let ok = match cmd.as_str() {
        "layers" => layers(),
        "check" => {
            layers()
                && cargo(&["fmt", "--all", "--check"])
                && cargo(&["clippy", "--workspace", "--all-targets", "--", "-D", "warnings"])
                && cargo(&["test", "--workspace", "--quiet"])
        }
        _ => {
            eprintln!("usage: cargo xtask <layers|check>");
            false
        }
    };
    if ok { ExitCode::SUCCESS } else { ExitCode::FAILURE }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn systems_cannot_depend_on_systems() {
        assert!(!allowed(Layer::System, Layer::System, "b789-sys-fuel"));
        assert!(allowed(Layer::System, Layer::Solver, "b789-solver-electrical"));
    }

    #[test]
    fn solvers_share_only_common() {
        assert!(allowed(Layer::Solver, Layer::Solver, "b789-solver-common"));
        assert!(!allowed(Layer::Solver, Layer::Solver, "b789-solver-fluid"));
        assert!(!allowed(Layer::Solver, Layer::System, "b789-sys-electrical"));
    }

    #[test]
    fn core_depends_only_on_core() {
        assert!(!allowed(Layer::Core, Layer::Solver, "b789-solver-common"));
        assert!(allowed(Layer::Core, Layer::Core, "b789-units"));
    }

    #[test]
    fn reads_local_deps() {
        let m = "[package]\nname = \"x\"\n[dependencies]\nb789-units.workspace = true\nserde = \"1\"\n\
                 [dev-dependencies]\nb789-clock = { workspace = true }\n";
        assert_eq!(local_deps(m), vec!["b789-units", "b789-clock"]);
    }
}
