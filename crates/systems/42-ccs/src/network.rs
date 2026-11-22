#![allow(clippy::needless_range_loop)]

use crate::cdn::{Cdn, DevKind, RATE_10M, RATE_100M};
use crate::data::RdcDef;
use std::collections::BTreeMap;

pub const GPMS: [&str; 16] = [
    "L1", "L2", "L3", "L4", "L5", "L6", "L7", "L8", "R1", "R2", "R3", "R4", "R5", "R6", "R7", "R8",
];
pub const CAS_HOSTS: [&str; 5] = ["L2", "L3", "L7", "R2", "R3"];
pub const EPD_APP: &str = "ELECTRICAL POWER DISTRIBUTION";
pub const GATEWAY_RPDUS: [&str; 4] = ["RPDU 71", "RPDU 72", "RPDU 81", "RPDU 82"];
pub const RDC_BAG_US: u64 = 16_000;
pub const APP_BAG_US: u64 = 32_000;
pub const TIME_BAG_US: u64 = 128_000;
pub const APPROX_RDC_FRAME_BYTES: usize = 256;
pub const APPROX_APP_FRAME_BYTES: usize = 200;
pub const APPROX_TIME_FRAME_BYTES: usize = 64;
pub const DISPLAY_APPS: [&str; 5] = [
    "EICAS SYSTEM & AOB DISPLAYS",
    "MAINTENANCE DISPLAYS",
    "NAVIGATION & MINI MAP DISPLAYS",
    "PFD & HUD DISPLAYS",
    "DISPLAY CREW ALERTING FUNCTION & BITE",
];

pub fn ars_for(location: &str) -> (u8, u8) {
    if location.contains("FLT DK") || location.contains("FWD EE") {
        (5, 6)
    } else if location.contains("CROWN DOOR 1") || location.contains("CROWN DOOR 2") {
        (1, 2)
    } else {
        (3, 4)
    }
}

pub fn slug(s: &str) -> String {
    s.to_ascii_lowercase()
        .chars()
        .map(|c| if c.is_ascii_alphanumeric() { c } else { '_' })
        .collect::<String>()
        .split('_')
        .filter(|p| !p.is_empty())
        .collect::<Vec<_>>()
        .join("_")
}

pub fn build(rdcs: &[RdcDef], hosted: &BTreeMap<String, Vec<String>>, seed: u64) -> Cdn {
    let mut c = Cdn::new(seed);
    let mut acs = [[0usize; 2]; 2];
    let mut fox = [[0usize; 2]; 2];
    for (s, side) in ["L", "R"].iter().enumerate() {
        for (ch, chn) in ["A", "B"].iter().enumerate() {
            acs[s][ch] = c.add_device(&format!("ACS {side} {chn}"), DevKind::Switch);
            fox[s][ch] = c.add_device(&format!("FOX {side} {chn}"), DevKind::Fox);
            c.connect(acs[s][ch], fox[s][ch], RATE_100M, false, ch);
        }
    }
    for ch in 0..2 {
        c.connect(fox[0][ch], fox[1][ch], RATE_100M, true, ch);
    }
    let mut ars = BTreeMap::new();
    for n in 1..=6u8 {
        ars.insert(n, c.add_device(&format!("ARS {n}"), DevKind::Switch));
    }
    for (n, ch) in [(5u8, 0usize), (6, 1)] {
        c.connect(fox[0][ch], ars[&n], RATE_100M, true, ch);
    }
    for (n, ch) in [(1u8, 0usize), (2, 1)] {
        c.connect(fox[1][ch], ars[&n], RATE_100M, true, ch);
    }
    for (a, b, ch) in [(1u8, 3u8, 0usize), (5, 3, 0), (2, 4, 1), (6, 4, 1)] {
        c.connect(ars[&a], ars[&b], RATE_100M, true, ch);
    }
    let mut gpm = BTreeMap::new();
    for g in GPMS {
        let s = if g.starts_with('L') { 0 } else { 1 };
        let d = c.add_device(&format!("GPM {g}"), DevKind::EndSystem);
        for ch in 0..2 {
            c.connect(d, acs[s][ch], RATE_100M, false, ch);
        }
        gpm.insert(g.to_string(), d);
    }
    let mut gg = Vec::new();
    for (s, side) in ["L", "R"].iter().enumerate() {
        for k in 1..=2 {
            let d = c.add_device(&format!("GG {side}{k}"), DevKind::EndSystem);
            for ch in 0..2 {
                c.connect(d, acs[s][ch], RATE_100M, false, ch);
            }
            gg.push(d);
        }
    }
    let mut ext = BTreeMap::new();
    for (name, s) in [
        ("P300", 0usize),
        ("P400", 1),
        ("RPDU 71", 0),
        ("RPDU 81", 0),
        ("RPDU 72", 1),
        ("RPDU 82", 1),
    ] {
        let d = c.add_device(name, DevKind::EndSystem);
        for ch in 0..2 {
            c.connect(d, acs[s][ch], RATE_10M, false, ch);
        }
        ext.insert(name.to_string(), d);
    }
    let mut rdc = BTreeMap::new();
    for r in rdcs {
        let d = c.add_device(&r.id, DevKind::EndSystem);
        match r.id.as_str() {
            "RDC 1" | "RDC 2" => {
                let s = if r.id == "RDC 1" { 0 } else { 1 };
                for ch in 0..2 {
                    c.connect(d, acs[s][ch], RATE_10M, false, ch);
                }
            }
            _ => {
                let (a, b) = ars_for(&r.location);
                c.connect(d, ars[&a], RATE_10M, false, 0);
                c.connect(d, ars[&b], RATE_10M, false, 1);
            }
        }
        rdc.insert(r.id.clone(), d);
    }
    let all_gpm: Vec<usize> = gpm.values().copied().collect();
    for r in rdcs {
        c.add_vl(
            &format!("{} DATA", r.id),
            rdc[&r.id],
            all_gpm.clone(),
            RDC_BAG_US,
            APPROX_RDC_FRAME_BYTES,
            false,
        );
    }
    let cas: Vec<usize> = CAS_HOSTS.iter().map(|g| gpm[*g]).collect();
    for (app, hosts) in hosted {
        for h in hosts {
            let src = gpm[h];
            let mut dests: Vec<usize> = hosts.iter().filter(|o| *o != h).map(|o| gpm[o]).collect();
            for d in &cas {
                if *d != src && !dests.contains(d) {
                    dests.push(*d);
                }
            }
            if DISPLAY_APPS.contains(&app.as_str()) {
                dests.extend(gg.iter().copied());
            }
            if app == EPD_APP {
                for r in GATEWAY_RPDUS {
                    dests.push(ext[r]);
                }
            }
            if dests.is_empty() {
                continue;
            }
            c.add_vl(
                &format!("{} @ {h}", slug(app)),
                src,
                dests,
                APP_BAG_US,
                APPROX_APP_FRAME_BYTES,
                false,
            );
        }
    }
    let epd: Vec<usize> = hosted
        .get(EPD_APP)
        .map(|h| h.iter().map(|g| gpm[g]).collect())
        .unwrap_or_default();
    let mut to_epd_cas = epd.clone();
    for d in &cas {
        if !to_epd_cas.contains(d) {
            to_epd_cas.push(*d);
        }
    }
    for p in ["P300", "P400"] {
        c.add_vl(
            &format!("BPCU {p}"),
            ext[p],
            to_epd_cas.clone(),
            APP_BAG_US,
            APPROX_APP_FRAME_BYTES,
            true,
        );
    }
    for r in GATEWAY_RPDUS {
        c.add_vl(
            &format!("{r} STATUS"),
            ext[r],
            epd.clone(),
            APP_BAG_US,
            APPROX_APP_FRAME_BYTES,
            false,
        );
    }
    for g in ["L1", "R1"] {
        let src = gpm[g];
        let dests: Vec<usize> = (0..c.devices.len())
            .filter(|d| c.devices[*d].kind == DevKind::EndSystem && *d != src)
            .collect();
        c.add_vl(
            &format!("TIME {g}"),
            src,
            dests,
            TIME_BAG_US,
            APPROX_TIME_FRAME_BYTES,
            true,
        );
    }
    c.build_routes();
    c
}
