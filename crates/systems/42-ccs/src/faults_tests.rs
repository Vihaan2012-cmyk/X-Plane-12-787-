use crate::ccs::{Ccs, CcsInputs, IFZ_NAMES};
use crate::faults::{self, CATALOG};
use crate::icd::Dataset;
use std::path::PathBuf;

fn power() -> CcsInputs {
    let mut i = CcsInputs::default();
    for b in ["CIB", "FIB", "DC28_L", "DC28_R", "HBB_MAIN"] {
        i.bus_v.insert(b.to_string(), 28.0);
    }
    i
}

fn ready() -> Ccs {
    let mut c = Ccs::new(
        &PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../../aircraft/42-ccs"),
        5,
    )
    .unwrap();
    c.set_elec_icd(&["ELEC GEN OFF L1".to_string(), "ELEC AC BUS L1".to_string()]);
    let p = c.encode_elec(&["ELEC GEN OFF L1".to_string()]).unwrap();
    c.bpcu_payload = [Some(p.clone()), Some(p)];
    for _ in 0..4000 {
        c.step(100_000, &power());
    }
    c
}

fn signature(c: &Ccs) -> Vec<String> {
    let mut s = c.crew_status_messages();
    for page in 1..=11u8 {
        for (k, v) in c.maint_page(page) {
            s.push(format!("maint{page}:{k}={v}"));
        }
    }
    s.push(format!("cas={:?}", c.cas_elec_messages()));
    s.push(format!(
        "crc={} police={} queue={} unknown={}",
        c.cdn.links.iter().map(|l| l.crc_errors).sum::<u64>(),
        c.cdn.police_drops(),
        c.cdn.queue_drops(),
        c.cdn.unknown_vl_drops
    ));
    let mut seq_err = 0u64;
    for v in &c.cdn.vls {
        for d in &v.dests {
            if let Some(r) = c.cdn.rx(*d, v.id) {
                seq_err += r.seq_errors;
            }
            let dest = c.cdn.devices[*d].name.clone();
            let got = c.read(&dest, &v.name);
            let valid = got.as_ref().is_some_and(|g| g.2);
            let ok_bit = got
                .as_ref()
                .map(|(data, _, _)| data.get(4).copied().unwrap_or(0) & 1);
            s.push(format!("{}>{}:{}:{:?}", v.name, dest, valid, ok_bit));
        }
    }
    s.push(format!("seq_errors>0={}", seq_err > 0));
    for r in &c.rdcs {
        if let Some((data, _, valid)) = c.read("GPM L1", &format!("{} DATA", r.def.id)) {
            let z: Vec<bool> = IFZ_NAMES
                .iter()
                .map(|n| valid && c.rdc_ds.decode(&data, n) == Some(1.0))
                .collect();
            s.push(format!(
                "{} ifz {:?} v{}",
                r.def.id,
                z,
                Dataset::version_of(&data)
            ));
        }
    }
    for g in crate::network::GPMS {
        s.push(format!(
            "{g} run={} lane1={}",
            c.gpm_running(g),
            c.sw_of(g).unwrap().single_lane()
        ));
    }
    for app in c.hosted.keys() {
        s.push(format!("{app}={}", c.app_available(app)));
    }
    s
}

#[test]
fn every_ccs_failure_mode_changes_something_observable() {
    let base = ready();
    let mut clean = base.clone();
    for _ in 0..100 {
        clean.step(100_000, &power());
    }
    let reference = signature(&clean);
    let mut silent = Vec::new();
    for (t, modes) in CATALOG {
        let parts = faults::instances(&base, t);
        assert!(!parts.is_empty(), "{t} has no instances");
        let part = if *t == "link" {
            parts
                .iter()
                .find(|p| p.contains("FOX L A|FOX R A"))
                .cloned()
                .unwrap()
        } else {
            parts[0].clone()
        };
        for m in modes.iter() {
            let mut f = base.clone();
            faults::arm(&mut f, &part, m, true).unwrap_or_else(|e| panic!("{t} {m}: {e}"));
            for _ in 0..100 {
                f.step(100_000, &power());
            }
            if signature(&f) == reference {
                silent.push(format!("{t}:{m} on {part}"));
            }
        }
    }
    assert!(silent.is_empty(), "unobservable: {silent:?}");
}

#[test]
fn failure_points_are_counted_across_every_instance() {
    let c = Ccs::new(
        &PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../../aircraft/42-ccs"),
        5,
    )
    .unwrap();
    let n = faults::failure_points(&c);
    let modes: usize = CATALOG.iter().map(|(_, m)| m.len()).sum();
    assert!(modes >= 30, "{modes}");
    assert_eq!(n, 693, "{n}");
}

#[test]
fn clearing_a_fault_restores_normal_operation() {
    let base = ready();
    let mut f = base.clone();
    faults::arm(&mut f, "ARS 5", "failed", true).unwrap();
    for _ in 0..50 {
        f.step(100_000, &power());
    }
    faults::arm(&mut f, "ARS 5", "failed", false).unwrap();
    for _ in 0..50 {
        f.step(100_000, &power());
    }
    assert!(f.crew_status_messages().iter().all(|m| !m.contains("ARS")));
}

#[test]
fn every_link_open_is_visible_on_a_maintenance_page() {
    let base = ready();
    let mut clean = base.clone();
    for _ in 0..10 {
        clean.step(100_000, &power());
    }
    let reference = signature(&clean);
    let mut silent = Vec::new();
    for part in faults::instances(&base, "link") {
        let mut f = base.clone();
        faults::arm(&mut f, &part, "open", true).unwrap();
        for _ in 0..10 {
            f.step(100_000, &power());
        }
        if signature(&f) == reference {
            silent.push(part);
        }
    }
    assert!(silent.is_empty(), "silent links: {silent:?}");
}
