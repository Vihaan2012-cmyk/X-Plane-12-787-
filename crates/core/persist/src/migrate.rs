use crate::document::{CURRENT_VERSION, Document};
use crate::error::PersistError;
use crate::value::Value;

pub type MigrationFn = fn(&mut Document) -> Result<(), PersistError>;

pub const MIGRATIONS: [(u32, MigrationFn); 1] = [(1, v1_to_v2)];

pub fn migrate_to_current(doc: &mut Document) -> Result<u32, PersistError> {
    migrate_with(doc, &MIGRATIONS, CURRENT_VERSION)
}

pub fn migrate_with(
    doc: &mut Document,
    table: &[(u32, MigrationFn)],
    target: u32,
) -> Result<u32, PersistError> {
    let mut steps = 0;
    while doc.version < target {
        let from = doc.version;
        let Some((_, f)) = table.iter().find(|(v, _)| *v == from) else {
            return Err(PersistError::Migration(format!(
                "no migration from version {from}"
            )));
        };
        f(doc)?;
        if doc.version != from + 1 {
            doc.version = from + 1;
        }
        steps += 1;
    }
    if doc.version > target {
        return Err(PersistError::UnsupportedVersion(doc.version));
    }
    Ok(steps)
}

fn rename_key<F: Fn(f64) -> f64>(
    sec: &mut crate::document::Section,
    from: &str,
    to: &str,
    f: F,
) -> Result<(), PersistError> {
    if let Some(v) = sec.remove(from) {
        match v {
            Value::F64(x) => sec.set(to, Value::F64(f(x))),
            _ => return Err(PersistError::WrongType(from.to_string())),
        }
    }
    Ok(())
}

fn v1_to_v2(doc: &mut Document) -> Result<(), PersistError> {
    for (name, sec) in doc.sections.iter_mut() {
        if name == "airframe" {
            if !sec.contains("settings.gap_mode") {
                sec.set("settings.gap_mode", Value::Str("sim".to_string()));
            }
        } else if name.starts_with("part/") {
            rename_key(sec, "hours", "run_s", |h| h * 3600.0)?;
            if !sec.contains("powered_s") {
                let run = sec.f64_or("run_s", 0.0)?;
                sec.set("powered_s", Value::F64(run));
            }
            rename_key(sec, "temp_c", "soak_c", |t| t)?;
            if !sec.contains("unpowered_s") {
                sec.set("unpowered_s", Value::F64(0.0));
            }
            if !sec.contains("soak_tau_s") {
                sec.set("soak_tau_s", Value::F64(5400.0));
            }
        } else if name.starts_with("battery/") {
            rename_key(sec, "soc_pct", "soc", |p| p / 100.0)?;
            rename_key(sec, "temp_c", "soak_c", |t| t)?;
            if !sec.contains("self_discharge_per_day") {
                sec.set("self_discharge_per_day", Value::F64(0.002));
            }
        }
    }
    doc.version = 2;
    Ok(())
}
