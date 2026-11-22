use super::*;
use crate::faults;
use std::collections::BTreeSet;

fn data_dir() -> std::path::PathBuf {
    std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../../aircraft/24-electrical")
}

fn new_sys() -> Electrical {
    Electrical::new(&data_dir()).expect("system builds")
}

fn flight() -> ElectricalInputs {
    ElectricalInputs {
        engine_rpm: [7600.0; 2],
        engine_running: [true; 2],
        on_ground: false,
        radio_alt_ft: 20000.0,
        ..ElectricalInputs::default()
    }
}

fn run(e: &mut Electrical, seconds: f64, inputs: &ElectricalInputs) {
    for _ in 0..(seconds * 100.0) as u64 {
        e.step(10_000, inputs);
    }
}

fn powered() -> Electrical {
    let mut e = new_sys();
    for i in 0..4 {
        e.panel_mut().gen_ctrl[i].set_latched(true);
    }
    for i in 0..2 {
        e.panel_mut().apu_gen[i].set_latched(true);
    }
    e.action("crew", "press", "BATTERY", Some("ON")).unwrap();
    run(&mut e, 6.0, &flight());
    e
}

fn round_floats(s: &str) -> String {
    let b: Vec<char> = s.chars().collect();
    let mut out = String::new();
    let mut i = 0;
    while i < b.len() {
        let start = i;
        let mut j = i;
        if b[j] == '-' {
            j += 1;
        }
        let digits_from = j;
        while j < b.len() && b[j].is_ascii_digit() {
            j += 1;
        }
        let int_end = j;
        if int_end > digits_from && j + 1 < b.len() && b[j] == '.' && b[j + 1].is_ascii_digit() {
            j += 1;
            while j < b.len() && b[j].is_ascii_digit() {
                j += 1;
            }
            if j < b.len() && (b[j] == 'e' || b[j] == 'E') {
                let mut k = j + 1;
                if k < b.len() && (b[k] == '-' || b[k] == '+') {
                    k += 1;
                }
                if k < b.len() && b[k].is_ascii_digit() {
                    while k < b.len() && b[k].is_ascii_digit() {
                        k += 1;
                    }
                    j = k;
                }
            }
            let tok: String = b[start..j].iter().collect();
            match tok.parse::<f64>() {
                Ok(v) => out.push_str(&format!("{:.3}", if v.abs() < 5.0e-4 { 0.0 } else { v })),
                Err(_) => out.push_str(&tok),
            }
            i = j;
        } else if int_end > digits_from {
            let tok: String = b[start..int_end].iter().collect();
            out.push_str(&tok);
            i = int_end;
        } else {
            out.push(b[i]);
            i += 1;
        }
    }
    out
}

fn fingerprint(e: &Electrical) -> Vec<String> {
    fingerprint_raw(e).iter().map(|s| round_floats(s)).collect()
}

fn fingerprint_raw(e: &Electrical) -> Vec<String> {
    let mut out = Vec::new();
    for (name, v) in e.snapshot() {
        if name.starts_with("elec.fault.") {
            continue;
        }
        match v {
            SignalValue::Num(n) => out.push(format!("{name}={n:.4}")),
            SignalValue::Bool(b) => out.push(format!("{name}={b}")),
            SignalValue::Text(t) => out.push(format!("{name}={t}")),
        }
    }
    for n in e.contactor_names() {
        out.push(format!("c.{n}={:?}", e.contactor_closed(&n)));
    }
    if let Some(s) = e.net_snapshot() {
        for b in &s.buses {
            out.push(format!("bus.{}={:.3}/{:.2}", b.name, b.voltage_v, b.freq_hz));
        }
        for c in &s.contactors {
            out.push(format!("cur.{}={:.2}", c.name, c.current_a));
        }
        for l in &s.loads {
            out.push(format!("load.{}={:.3}/{:.3}", l.name, l.voltage_v, l.current_a));
        }
        for q in &s.sspcs {
            out.push(format!("sspc.{}={:.3}/{:.3}", q.id, q.voltage_v, q.current_a));
        }
        for b in &s.batteries {
            out.push(format!(
                "batt.{}={:.3}/{:.3}/{:.5}/{:.3}",
                b.name, b.voltage_v, b.current_a, b.soc, b.temp_k
            ));
        }
        for m in &s.machines {
            out.push(format!("mach.{}={:.3}/{:.3}", m.name, m.terminal_v, m.current_a));
        }
    }
    out.push(format!("lamps={:?}", e.lamps()));
    out.push(format!("eicas={:?}", e.eicas_messages()));
    out.push(format!("cbic={:?}", e.cbic()));
    for g in 0..6 {
        if let Some(o) = e.gcu_output(g) {
            out.push(format!("gcu{g}={o:?}"));
        }
    }
    for k in 0..3 {
        if let Some(o) = e.ext_output(k) {
            out.push(format!("ext{k}={o:?}"));
        }
    }
    out.push(format!("bpcu={:?}", e.bpcu_output()));
    out.push(format!("bmu={:?}/{:?}", e.bmu_out, e.charger_out));
    out.push(format!("rpdu={:?}", e.rpdu_views()));
    out.push(format!(
        "sensed={:?}/{:?}/{:.3}/{:?}",
        e.crew_in.ac_bus_volts, e.crew_in.ac_bus_energized, e.crew_in.main_batt_volts, e.crew_in.dc28_volts
    ));
    for id in e.thermal_part_ids() {
        if let Some(SignalValue::Num(t)) = e.value(&format!("elec.temp.{id}")) {
            out.push(format!("t.{id}={t:.4}"));
        }
    }
    out
}

#[test]
fn currents_heat_wires_and_contacts_and_resistance_rises() {
    let mut e = powered();
    assert_eq!(e.flag("elec.gcb.l1.closed"), Some(true));
    e.plant.set_load("LOAD_L1", 150_000.0);
    run(&mut e, 2.0, &flight());
    let r0 = e.feeder_resistance_ohm("L1 GCB").unwrap();
    let c0 = e.contactor_temp_c("L1 GCB").unwrap();
    let w0 = e.wire_temp_c("L1 GCB").unwrap();
    let f0 = e.heat().unwrap().resistance_factor("L1 GCB").unwrap();
    e.heat_advance(1_800_000_000);
    let c1 = e.contactor_temp_c("L1 GCB").unwrap();
    let w1 = e.wire_temp_c("L1 GCB").unwrap();
    let f1 = e.heat().unwrap().resistance_factor("L1 GCB").unwrap();
    assert!(c1 > c0 + 2.0, "{c0} {c1}");
    assert!(w1 > w0 + 2.0, "{w0} {w1}");
    assert!(f1 > f0, "{f0} {f1}");
    let r1 = e.feeder_resistance_ohm("L1 GCB").unwrap();
    assert!(r1 > r0, "{r0} {r1}");
    assert!(e.signal("elec.temp.L1_GCB").unwrap() > c0);
    assert!(e.signal("elec.temp.W.L1_GCB").unwrap() > w0);
}

#[test]
fn high_resistance_contactor_runs_hotter_and_drops_more() {
    let mut a = powered();
    let mut b = powered();
    for e in [&mut a, &mut b] {
        e.plant.set_load("LOAD_L1", 120_000.0);
        run(e, 2.0, &flight());
    }
    b.action("fault", "arm", "L1 GCB", Some("high-resistance 1.0"))
        .unwrap();
    for e in [&mut a, &mut b] {
        run(e, 2.0, &flight());
        e.heat_advance(900_000_000);
    }
    assert!(b.contactor_temp_c("L1 GCB").unwrap() > a.contactor_temp_c("L1 GCB").unwrap() + 5.0);
    assert!(b.feeder_resistance_ohm("L1 GCB").unwrap() > 5.0 * a.feeder_resistance_ohm("L1 GCB").unwrap());
}

#[test]
fn cold_soak_chills_parts_and_battery_pack() {
    let mut e = new_sys();
    e.action("crew", "press", "BATTERY", Some("ON")).unwrap();
    run(&mut e, 2.0, &ElectricalInputs::default());
    let warm = e.signal("elec.battery.main.temp_c").unwrap();
    let r_warm = e.heat().unwrap().resistance_factor("BATT_MAIN").unwrap();
    e.cold_soak(238.15);
    e.heat_advance(600_000_000);
    run(&mut e, 2.0, &ElectricalInputs::default());
    let cold = e.signal("elec.battery.main.temp_c").unwrap();
    assert!(cold < warm - 30.0, "{warm} {cold}");
    assert!((e.part_temp_c("L1 GCB").unwrap() + 35.0).abs() < 2.0);
    let r_cold = e.heat().unwrap().resistance_factor("BATT_MAIN").unwrap();
    assert!(r_cold > r_warm * 1.5, "{r_warm} {r_cold}");
    let v_cold = e.signal("elec.battery.main.voltage").unwrap();
    assert!(v_cold > 10.0);
}

#[test]
fn oat_drives_zone_temperatures() {
    let mut cold = powered();
    let mut hot = powered();
    let c = ElectricalInputs {
        ambient_c: -40.0,
        ..flight()
    };
    let h = ElectricalInputs {
        ambient_c: 50.0,
        ..flight()
    };
    run(&mut cold, 2.0, &c);
    run(&mut hot, 2.0, &h);
    cold.heat_advance(3_600_000_000);
    hot.heat_advance(3_600_000_000);
    assert!(hot.part_temp_c("VFSG_L1").unwrap() > cold.part_temp_c("VFSG_L1").unwrap() + 20.0);
}

#[test]
fn fault_arm_and_clear_actions_round_trip() {
    let mut e = powered();
    assert!(e.action("fault", "arm", "NOPE", Some("weld")).is_err());
    assert!(e.action("fault", "arm", "L1 GCB", Some("bogus")).is_err());
    e.action("fault", "arm", "L1 GCB", Some("weld")).unwrap();
    run(&mut e, 0.1, &flight());
    assert_eq!(e.active_faults().len(), 1);
    assert_eq!(e.signal("elec.fault.count"), Some(1.0));
    e.action("fault", "clear", "L1 GCB", None).unwrap();
    run(&mut e, 0.1, &flight());
    assert_eq!(e.active_faults().len(), 0);
    assert!(e.action("fault", "clear", "L1 GCB", None).is_err());
}

#[test]
fn contactor_weld_keeps_gcb_closed_after_command_drops() {
    let mut e = powered();
    assert_eq!(e.flag("elec.gcb.l1.closed"), Some(true));
    e.action("fault", "arm", "L1 GCB", Some("weld")).unwrap();
    run(&mut e, 0.2, &flight());
    e.panel_mut().gen_ctrl[0].set_latched(false);
    run(&mut e, 3.0, &flight());
    assert_eq!(e.flag("elec.gcb.l1.closed"), Some(true));
}

#[test]
fn coil_open_prevents_gcb_closing() {
    let mut e = new_sys();
    for i in 0..4 {
        e.panel_mut().gen_ctrl[i].set_latched(true);
    }
    e.action("crew", "press", "BATTERY", Some("ON")).unwrap();
    e.action("fault", "arm", "L1 GCB", Some("coil-open")).unwrap();
    run(&mut e, 6.0, &flight());
    assert_eq!(e.flag("elec.gcb.l1.closed"), Some(false));
}

#[test]
fn pmg_failure_drops_generator_and_shows_message() {
    let mut e = powered();
    e.action("fault", "arm", "VFSG_L1", Some("pmg-failure")).unwrap();
    run(&mut e, 4.0, &flight());
    assert_eq!(e.flag("elec.gcb.l1.closed"), Some(false));
    assert!(
        e.eicas_messages().iter().any(|m| m.contains("GEN OFF")),
        "{:?}",
        e.eicas_messages()
    );
}

#[test]
fn bearing_wear_raises_oil_temperature_through_the_machine() {
    let mut a = powered();
    let mut b = powered();
    b.action("fault", "arm", "VFSG_L1", Some("bearing-wear 1.0"))
        .unwrap();
    for e in [&mut a, &mut b] {
        run(e, 2.0, &flight());
        e.heat_advance(1_800_000_000);
        run(e, 2.0, &flight());
    }
    assert!(b.part_temp_c("VFSG_L1").unwrap() > a.part_temp_c("VFSG_L1").unwrap() + 3.0);
    let oa = a.plant.net().machine_oil_temp_c("VFSG_L1").unwrap();
    let ob = b.plant.net().machine_oil_temp_c("VFSG_L1").unwrap();
    assert!(ob > oa, "{oa} {ob}");
}

#[test]
fn battery_cell_fade_and_short_change_pack() {
    let mut a = powered();
    let mut b = powered();
    b.action("fault", "arm", "BATT_MAIN.c3", Some("capacity-fade 1.0"))
        .unwrap();
    b.action("fault", "arm", "BATT_MAIN.c4", Some("internal-short 1.0"))
        .unwrap();
    for e in [&mut a, &mut b] {
        run(e, 5.0, &flight());
        e.heat_advance(600_000_000);
        run(e, 1.0, &flight());
    }
    assert!(b.plant.net().battery().cells[4].soc < a.plant.net().battery().cells[4].soc);
    assert!(b.plant.net().battery().cells[3].cap_scale < a.plant.net().battery().cells[3].cap_scale);
    assert!(b.part_temp_c("BATT_MAIN").unwrap() > a.part_temp_c("BATT_MAIN").unwrap());
}

#[test]
fn gcu_regulator_drift_moves_setpoint_and_overflow_preloads() {
    let mut e = powered();
    let base = e.gcus[0].params.regulated_v;
    e.action("fault", "arm", "GCU_VFSG_L1", Some("regulator-drift-high 0.5"))
        .unwrap();
    e.action("fault", "arm", "GCU_VFSG_L2", Some("counter-overflow-248d 1.0"))
        .unwrap();
    run(&mut e, 0.5, &flight());
    assert!((e.gcus[0].params.regulated_v - base - 20.0).abs() < 1e-9);
    assert!(e.gcus[1].powered_time_us() >= 248 * 24 * 3600 * 1_000_000 / 2);
    e.action("fault", "clear", "GCU_VFSG_L1", None).unwrap();
    run(&mut e, 0.5, &flight());
    assert!((e.gcus[0].params.regulated_v - base).abs() < 1e-9);
}

#[test]
fn wire_chafe_trips_feeder() {
    let mut e = powered();
    assert_eq!(e.flag("elec.gcb.l1.closed"), Some(true));
    e.action("fault", "arm", "W:L1 GCB", Some("chafe-to-ground 1.0"))
        .unwrap();
    run(&mut e, 1.0, &flight());
    assert_eq!(e.flag("elec.fault.tripped.W.L1_GCB"), Some(true));
    assert_eq!(e.flag("elec.gcb.l1.closed"), Some(false));
}

#[test]
fn legacy_fault_actions_route_through_the_book() {
    let mut e = powered();
    e.action("fault", "gen_loss", "L1_VFSG", None).unwrap();
    e.action("fault", "bpcu_fail", "L", None).unwrap();
    run(&mut e, 0.2, &flight());
    let parts: Vec<&str> = e.active_faults().iter().map(|f| f.0.as_str()).collect();
    assert!(parts.contains(&"VFSG_L1"));
    assert!(parts.contains(&"BPCU_L"));
}

#[test]
fn fault_runs_are_deterministic() {
    let mut a = powered();
    let mut b = powered();
    for e in [&mut a, &mut b] {
        e.action("fault", "arm", "VFSG_L2", Some("bearing-wear 0.7"))
            .unwrap();
        e.action("fault", "arm", "R1 GCB", Some("high-resistance 0.4"))
            .unwrap();
        run(e, 3.0, &flight());
        e.heat_advance(300_000_000);
    }
    assert_eq!(fingerprint(&a), fingerprint(&b));
}

fn audit_plan(e: &Electrical) -> Vec<(&'static str, String, &'static str, f64)> {
    let sspc = e.plant.net().sspc_ids().first().cloned().unwrap_or_default();
    vec![
        (faults::VFSG, "VFSG_L1".into(), "bearing-wear", 1.0),
        (faults::ASG, "ASG_L".into(), "bearing-wear", 1.0),
        (faults::RAT_GEN, "RAT".into(), "bearing-wear", 1.0),
        (faults::GCU, "GCU_VFSG_L1".into(), "regulator-drift-high", 1.0),
        (faults::BPCU, "BPCU_L".into(), "cpu-fail", 1.0),
        (faults::BATTERY_CELL, "BATT_MAIN.c2".into(), "internal-short", 1.0),
        (faults::BATTERY_PACK, "BATT_MAIN".into(), "thermal-runaway", 1.0),
        (faults::BMU, "BMU_MAIN".into(), "bmu-fail", 1.0),
        (faults::CHARGER, "CHG_MAIN".into(), "overtemp", 1.0),
        (faults::DIODE_MODULE, "DIODE_MAIN".into(), "short", 1.0),
        (faults::ATRU, "ATRU_L1".into(), "degraded-output", 1.0),
        (faults::ATU, "ATU_L".into(), "winding-short", 1.0),
        (faults::TRU, "TRU_C1".into(), "degraded-output", 1.0),
        (faults::SPU, "SPU".into(), "internal-fault", 1.0),
        (faults::CMSC, "CMSC_CAC_L1".into(), "internal-fault", 1.0),
        (faults::CONTACTOR, "L1 GCB".into(), "coil-open", 1.0),
        (faults::ELCC, "ELCC_1".into(), "stuck-open", 1.0),
        (faults::ELCU, "ELCU_1".into(), "internal-fault", 1.0),
        (faults::SSPC, sspc, "stuck-open", 1.0),
        (faults::TCB, "TCB_1".into(), "nuisance-trip", 1.0),
        (faults::RPDU_CONTROLLER, "RPDUC 21".into(), "gateway-fail", 1.0),
        (faults::RPDU_MODULE, "RPDU 21".into(), "overtemp", 1.0),
        (faults::SPDU_CARD, "SPDU_L".into(), "card-fail", 1.0),
        (faults::WIRE_SEGMENT, "W:L1 GCB".into(), "high-resistance", 1.0),
        (faults::CONNECTOR, "CONN:L1 GCB".into(), "high-resistance", 1.0),
        (faults::GROUND_STUD, "GND:P100".into(), "open", 1.0),
        (faults::PANEL_SWITCH, "SW:BATTERY".into(), "open-contact", 1.0),
        (faults::LAMP, "LAMP:BATTERY".into(), "short", 1.0),
        (faults::EICAS_INPUT, "EICASIN:ELEC".into(), "stuck-low", 1.0),
        (faults::OIL_AIR_VALVE, "OAV_L1".into(), "locked-closed", 1.0),
        (faults::RAT_HEATER, "RATHTR".into(), "thermostat-stuck", 1.0),
        (
            faults::EXT_POWER_RECEPTACLE,
            "EXTRCPT_1".into(),
            "pin-ef-open",
            1.0,
        ),
        (faults::PECS_PUMP, "PECS_1".into(), "pump-loss", 1.0),
    ]
}

fn audit_flight() -> ElectricalInputs {
    let mut i = flight();
    i.ext[0] = ExtSupply {
        connected: true,
        pin_ef_ok: true,
        ..ExtSupply::default()
    };
    i
}

#[test]
fn observability_audit_every_part_type_changes_real_system_state() {
    let probe = new_sys();
    let mut plan = audit_plan(&probe);
    assert_eq!(plan.len(), faults::part_types().len());
    let covered: BTreeSet<&str> = plan.iter().map(|p| p.0).collect();
    for t in faults::part_types() {
        assert!(covered.contains(t), "type {t} missing from plan");
    }
    let mut baseline = powered();
    run(&mut baseline, 2.0, &audit_flight());
    run(&mut baseline, 0.5, &audit_flight());
    baseline.heat_advance(30_000_000);
    let base_fp = fingerprint(&baseline);
    let feeds = baseline.plant.net().feed_sspc_ids();
    let loaded = baseline.net_snapshot().and_then(|s| {
        feeds
            .iter()
            .enumerate()
            .map(|(k, id)| (k, id.clone(), s.sspc(id).map_or(0.0, |q| q.current_a.abs())))
            .max_by(|a, b| a.2.total_cmp(&b.2))
    });
    if let Some((k, id, amps)) = loaded
        && amps > 0.05
    {
        for item in plan.iter_mut() {
            if item.0 == faults::SSPC {
                item.1 = id.clone();
            }
            if item.0 == faults::TCB {
                item.1 = format!("TCB_{}", k + 1);
            }
        }
    }
    let mut changed_types = Vec::new();
    let mut silent = Vec::new();
    for (ptype, part, mode, mag) in plan {
        let mut e = powered();
        run(&mut e, 2.0, &audit_flight());
        let arm = format!("{mode} {mag}");
        e.action("fault", "arm", &part, Some(&arm))
            .unwrap_or_else(|err| panic!("{ptype}/{part}/{mode}: arm failed {err}"));
        run(&mut e, 0.5, &audit_flight());
        e.heat_advance(30_000_000);
        let fp = fingerprint(&e);
        let diffs: Vec<&String> = fp.iter().filter(|x| !base_fp.contains(x)).collect();
        if diffs.is_empty() && fp.len() == base_fp.len() {
            silent.push(format!("{ptype}/{part}/{mode}"));
        } else {
            let first: Vec<String> = diffs
                .iter()
                .take(2)
                .map(|d| d.chars().take(70).collect())
                .collect();
            eprintln!(
                "audit {ptype}/{part}/{mode}: {} changed, e.g. {first:?}",
                diffs.len()
            );
            changed_types.push(ptype);
        }
    }
    eprintln!(
        "audit: {}/{} part types change real system state; silent: {silent:?}",
        changed_types.len(),
        faults::part_types().len()
    );
    assert!(silent.is_empty(), "{silent:#?}");
    assert_eq!(changed_types.len(), faults::part_types().len());
}

#[test]
fn audit_control_a_mode_with_no_model_effect_leaves_the_fingerprint_unchanged() {
    let mut a = powered();
    let mut b = powered();
    for e in [&mut a, &mut b] {
        run(e, 2.0, &audit_flight());
    }
    b.action("fault", "arm", "BPCU_L", Some("counter-overflow-248d 1.0"))
        .unwrap();
    for e in [&mut a, &mut b] {
        run(e, 0.5, &audit_flight());
        e.heat_advance(30_000_000);
    }
    assert_eq!(fingerprint(&a), fingerprint(&b));
}

#[test]
fn atu_winding_short_sags_the_115_bus_until_protection_and_ground_stud_open_drops_the_rpdu_feed() {
    let mut a = powered();
    let mut b = powered();
    let mut c = powered();
    for e in [&mut a, &mut b, &mut c] {
        run(e, 2.0, &flight());
    }
    b.action("fault", "arm", "ATU_L", Some("winding-short 1.0"))
        .unwrap();
    c.action("fault", "arm", "GND:RPDU 21", Some("open 1.0")).unwrap();
    for e in [&mut a, &mut b, &mut c] {
        run(e, 0.05, &flight());
    }
    assert!(b.bus_volts("AC115_L") < a.bus_volts("AC115_L") - 5.0);
    for e in [&mut a, &mut b, &mut c] {
        run(e, 1.0, &flight());
    }
    assert_eq!(b.contactor_closed("L ATUC"), Some(false));
    assert!((b.bus_volts("AC115_L") - a.bus_volts("AC115_L")).abs() < 5.0);
    let v = |e: &Electrical| e.net_snapshot().unwrap().sspc("RPDU21.01").unwrap().voltage_v;
    assert!(v(&c) < v(&a) - 1.0, "{} {}", v(&a), v(&c));
}

#[test]
fn elcc_and_elcu_faults_open_the_rpdu_ac_feeds_and_clear_restores_them() {
    let mut a = powered();
    let mut b = powered();
    run(&mut a, 2.0, &flight());
    run(&mut b, 2.0, &flight());
    assert_eq!(a.contactor_closed("AC FEED RPDU 21"), Some(true));
    b.action("fault", "arm", "ELCC_21", Some("stuck-open 1.0"))
        .unwrap();
    b.action("fault", "arm", "ELCU_2", Some("internal-fault 1.0"))
        .unwrap();
    run(&mut b, 1.0, &flight());
    assert_eq!(b.contactor_closed("AC FEED RPDU 21"), Some(false));
    assert_eq!(b.contactor_closed("AC FEED RPDU 22"), Some(false));
    b.action("fault", "clear", "ELCC_21", None).unwrap();
    b.action("fault", "clear", "ELCU_2", None).unwrap();
    run(&mut b, 1.0, &flight());
    assert_eq!(b.contactor_closed("AC FEED RPDU 21"), Some(true));
}

#[test]
fn eicas_input_stuck_low_reaches_eicas_while_the_real_bus_stays_live() {
    let mut a = powered();
    let mut b = powered();
    run(&mut a, 2.0, &flight());
    run(&mut b, 2.0, &flight());
    b.action("fault", "arm", "EICASIN:ACBUS_L1", Some("stuck-low 1.0"))
        .unwrap();
    run(&mut b, 8.0, &flight());
    run(&mut a, 8.0, &flight());
    assert!(b.bus_volts("L1") > 200.0);
    assert_ne!(a.eicas_messages(), b.eicas_messages());
}

#[test]
fn ground_fault_leak_draws_real_current_then_protection_trips_the_generator_feeder() {
    let mut e = powered();
    e.action("fault", "arm", "W:L1 GCB", Some("chafe-to-ground 1.0"))
        .unwrap();
    run(&mut e, 0.02, &flight());
    assert!(e.plant.net().shunt_site("L1 GCB").is_some());
    run(&mut e, 1.0, &flight());
    assert_eq!(e.contactor_closed("L1 GCB"), Some(false));
    assert!(e.fault_tripped("W:L1 GCB"));
}

#[test]
fn btb_fail_to_close_latches_after_150ms_and_bpcu_reroutes() {
    let mut e = powered();
    e.action("fault", "arm", "L3 BTB", Some("coil-open")).unwrap();
    e.panel_mut().gen_ctrl[0].set_latched(false);
    run(&mut e, 0.1, &flight());
    assert_eq!(e.flag("elec.bpcu.fail_to_close.l3_btb"), Some(false));
    run(&mut e, 0.2, &flight());
    assert_eq!(e.flag("elec.bpcu.fail_to_close.l3_btb"), Some(true));
    run(&mut e, 2.0, &flight());
    assert_eq!(e.contactor_closed("L3 BTB"), Some(false));
    assert!(e.bus_volts("L1") > 200.0, "L1 {}", e.bus_volts("L1"));
}

#[test]
fn bsb_fail_to_close_latches_after_190ms() {
    let mut e = new_sys();
    e.action("fault", "arm", "L BSB", Some("coil-open")).unwrap();
    for i in 0..4 {
        e.panel_mut().gen_ctrl[i].set_latched(true);
    }
    e.action("crew", "press", "BATTERY", Some("ON")).unwrap();
    run(&mut e, 6.0, &flight());
    assert_eq!(e.flag("elec.bpcu.fail_to_close.l_bsb"), Some(true));
    assert_eq!(e.flag("elec.bpcu.fail_to_close.r_bsb"), Some(false));
    assert_eq!(e.contactor_closed("L BSB"), Some(false));
}

#[test]
fn tru_overcurrent_trips_on_tier_three_and_latches_until_input_removed() {
    let mut e = powered();
    let base = e.value("elec.tru.l.output_a");
    e.plant.net_mut().set_shunt("test.dc28l", "DC28_L", Some(0.055));
    run(&mut e, 1.0, &flight());
    let amps = match e.value("elec.tru.l.output_a") {
        Some(SignalValue::Num(a)) => a,
        v => panic!("{v:?} base {base:?}"),
    };
    assert!(amps > 420.0, "TRU_L carries {amps} A");
    run(&mut e, 10.0, &flight());
    assert_eq!(e.flag("elec.tru.l.oc_tripped"), Some(false));
    run(&mut e, 1.5, &flight());
    assert_eq!(e.flag("elec.tru.l.oc_tripped"), Some(true));
    run(&mut e, 0.5, &flight());
    assert!(e.tru_output_a(0).abs() < 1.0, "{}", e.tru_output_a(0));
    e.plant.net_mut().set_shunt("test.dc28l", "DC28_L", None);
    run(&mut e, 2.0, &flight());
    assert_eq!(e.flag("elec.tru.l.oc_tripped"), Some(true));
    assert_eq!(e.flag("elec.tru.r.oc_tripped"), Some(false));
}

#[test]
fn winding_short_is_caught_by_gcu_differential_from_real_phase_currents() {
    let mut e = powered();
    e.action("fault", "arm", "VFSG_L1", Some("winding-short 1.0"))
        .unwrap();
    run(&mut e, 0.02, &flight());
    let (_, ia, _) = e.plant.machine_phases(0);
    let feeder = e.plant.contactor_phase_a("L1 GCB");
    assert!(
        ia[0] - feeder[0] > crate::sources::gcu::DIFF_THRESH_LOW_A,
        "{ia:?} {feeder:?}"
    );
    run(&mut e, 0.3, &flight());
    assert_ne!(
        e.gcus[0].latched_mask() & crate::sources::gcu::GcuFault::Differential.mask(),
        0
    );
    assert_eq!(e.contactor_closed("L1 GCB"), Some(false));
}

#[test]
fn feeder_chafe_to_ground_draws_residual_and_gcu_trips_the_generator() {
    let mut e = powered();
    e.action("fault", "arm", "W:L1 GCB", Some("chafe-to-ground 1.0"))
        .unwrap();
    run(&mut e, 0.02, &flight());
    let (_, _, residual) = e.plant.machine_phases(0);
    assert!(residual > crate::sources::gcu::GROUND_FAULT_A, "{residual}");
    run(&mut e, 0.5, &flight());
    assert!(e.gcu_out[0].trip_latched);
    assert_eq!(e.contactor_closed("L1 GCB"), Some(false));
    assert!(e.fault_tripped("W:L1 GCB"));
    assert!(e.bus_volts("L1") > 200.0, "L1 re-fed {}", e.bus_volts("L1"));
}

#[test]
fn shorted_rotating_diode_weakens_field_until_the_gcu_trips() {
    let mut e = powered();
    e.action("fault", "arm", "VFSG_L1", Some("rotating-diode-short 1.0"))
        .unwrap();
    run(&mut e, 15.0, &flight());
    assert!(e.gcu_out[0].trip_latched, "mask {}", e.gcus[0].latched_mask());
    assert_eq!(e.contactor_closed("L1 GCB"), Some(false));
}

#[test]
fn atu_winding_short_trips_atu_differential_and_115_bus_is_tied_from_the_other_side() {
    let mut e = powered();
    e.action("fault", "arm", "ATU_L", Some("winding-short 1.0"))
        .unwrap();
    run(&mut e, 0.05, &flight());
    assert!(e.zone_leak_a("ATU_L") > crate::protection::zone_diff::ATU_DIFF_A);
    assert_eq!(e.flag("elec.atu.l.diff_tripped"), Some(false));
    run(&mut e, 0.1, &flight());
    assert_eq!(e.flag("elec.atu.l.diff_tripped"), Some(true));
    run(&mut e, 1.0, &flight());
    assert_eq!(e.contactor_closed("L ATUC"), Some(false));
    assert!(e.bus_volts("AC115_L") > 100.0, "{}", e.bus_volts("AC115_L"));
    assert_eq!(e.flag("elec.atu.r.diff_tripped"), Some(false));
}

#[test]
fn tie_bus_fault_trips_tie_differential_after_260ms_and_isolates_the_tie() {
    let mut e = new_sys();
    for i in 0..2 {
        e.panel_mut().apu_gen[i].set_latched(true);
    }
    e.action("crew", "press", "BATTERY", Some("ON")).unwrap();
    let apu = ElectricalInputs {
        apu_running: true,
        apu_rpm: 12_000.0,
        on_ground: true,
        ..ElectricalInputs::default()
    };
    run(&mut e, 8.0, &apu);
    assert!(e.bus_volts("L1") > 200.0, "L1 {}", e.bus_volts("L1"));
    e.plant.net_mut().set_phase_shunt("test.ltb", "LTB", Some(2.0));
    run(&mut e, 0.05, &apu);
    assert!(e.zone_leak_a("LTB") > crate::protection::zone_diff::TIE_DIFF_A);
    for _ in 0..20 {
        run(&mut e, 0.01, &apu);
        assert_eq!(
            e.flag("elec.tie.l.diff_tripped"),
            Some(false),
            "leak {}",
            e.zone_leak_a("LTB")
        );
    }
    run(&mut e, 0.02, &apu);
    assert_eq!(e.flag("elec.tie.l.diff_tripped"), Some(true));
    run(&mut e, 1.0, &apu);
    assert_eq!(e.contactor_closed("L APB"), Some(false));
    assert_eq!(e.contactor_closed("L2 BTB"), Some(false));
    assert_eq!(e.contactor_closed("R1 BTB"), Some(false));
    assert!(e.zone_leak_a("LTB") < 1.0);
}

#[test]
fn high_resistance_strand_in_one_parallel_feeder_trips_unbalanced_feeder() {
    let mut e = powered();
    e.plant.net_mut().set_shunt("test.l1load", "L1", Some(2.0));
    e.action("fault", "arm", "W:L1 GCB", Some("high-resistance 1.0"))
        .unwrap();
    run(&mut e, 0.1, &flight());
    let ub = e.feeder_unbalance_a(0, "L1 GCB");
    assert!(ub > crate::sources::gcu::UNBALANCED_FEEDER_A, "{ub}");
    run(&mut e, 2.5, &flight());
    assert!(!e.gcu_out[0].trip_latched);
    run(&mut e, 0.3, &flight());
    assert_ne!(
        e.gcus[0].latched_mask() & crate::sources::gcu::GcuFault::UnbalancedFeeder.mask(),
        0,
        "unbalance {ub}"
    );
}

#[test]
fn shorted_field_driver_is_detected_field_relay_opens_and_breaker_trips() {
    let mut e = powered();
    e.action("fault", "arm", "GCU_VFSG_L1", Some("field-driver-short 1.0"))
        .unwrap();
    run(&mut e, 0.2, &flight());
    assert_ne!(
        e.gcus[0].latched_mask() & crate::sources::gcu::GcuFault::FieldDriver.mask(),
        0
    );
    assert!(e.gcu_out[0].field_off);
    assert_eq!(e.contactor_closed("L1 GCB"), Some(false));
    assert!(e.bus_volts("L1") > 200.0);
}

#[test]
fn current_transformer_offset_causes_a_differential_trip() {
    let mut e = powered();
    e.action(
        "fault",
        "arm",
        "GCU_VFSG_L1",
        Some("current-transformer-offset 1.0"),
    )
    .unwrap();
    run(&mut e, 0.3, &flight());
    assert_ne!(
        e.gcus[0].latched_mask() & crate::sources::gcu::GcuFault::Differential.mask(),
        0
    );
}

fn ground() -> ElectricalInputs {
    ElectricalInputs {
        on_ground: true,
        ..ElectricalInputs::default()
    }
}

#[test]
fn battery_off_keeps_ground_battery_only_mode_for_30s() {
    let mut e = new_sys();
    e.action("crew", "press", "BATTERY", Some("ON")).unwrap();
    run(&mut e, 5.0, &ground());
    assert_eq!(e.mode(), "on_ground_battery_only");
    e.action("crew", "press", "BATTERY", Some("OFF")).unwrap();
    run(&mut e, 29.0, &ground());
    assert!(e.bpcu_out.only_apu_start_loads);
    run(&mut e, 1.5, &ground());
    assert!(!e.bpcu_out.only_apu_start_loads);
}

#[test]
fn battery_discharge_drives_the_disch_message_and_inhibit_through_the_monitor() {
    let mut e = new_sys();
    e.action("crew", "press", "BATTERY", Some("ON")).unwrap();
    run(&mut e, 5.0, &ground());
    e.plant.net_mut().set_shunt("test.hbb", "HBB_MAIN", Some(0.8));
    run(&mut e, 1.0, &ground());
    let d = e.plant.battery_a(0);
    assert!(d >= crate::storage::monitor::DISCH_HIGH_A, "{d}");
    run(&mut e, 18.0, &ground());
    assert!(!e.batt_mon_out.discharge_inhibit);
    run(&mut e, 1.5, &ground());
    assert!(e.batt_mon_out.discharge_inhibit);
    run(&mut e, 2.0, &ground());
    assert!(
        e.eicas_messages().iter().any(|m| m == "MAIN BATTERY DISCH"),
        "{:?}",
        e.eicas_messages()
    );
}

#[test]
fn load_circuit_wiring_faults_act_on_the_real_load_and_bus() {
    let mut e = powered();
    let (name, bus) = e
        .plant
        .net()
        .circuits()
        .iter()
        .find(|(_, b)| b == "DC28_L")
        .cloned()
        .expect("a DC28_L circuit");
    let base = e.plant.net().load_demand_w(&name).unwrap();
    let part = format!("W:{name}");
    e.action("fault", "arm", &part, Some("open 1.0")).unwrap();
    run(&mut e, 0.1, &flight());
    assert_eq!(e.plant.net().load_demand_w(&name), Some(0.0));
    e.action("fault", "clear", &part, None).unwrap();
    run(&mut e, 0.1, &flight());
    assert_eq!(e.plant.net().load_demand_w(&name), Some(base));
    e.action("fault", "arm", &part, Some("high-resistance 1.0"))
        .unwrap();
    run(&mut e, 0.1, &flight());
    assert!(e.plant.net().load_demand_w(&name).unwrap() > base);
    e.action("fault", "clear", &part, None).unwrap();
    let before = e.bus_volts(&bus);
    e.action("fault", "arm", &part, Some("chafe-to-ground 1.0"))
        .unwrap();
    run(&mut e, 0.1, &flight());
    assert!(e.fault_tripped(&part), "circuit protection trips on a hard chafe");
    assert_eq!(e.plant.net().load_demand_w(&name), Some(0.0));
    assert!((e.bus_volts(&bus) - before).abs() < 1.0);
}

#[test]
fn pecs_coolant_settles_near_supply_and_warms_after_pump_loss() {
    let mut e = powered();
    run(&mut e, 300.0, &flight());
    let settled = e.pecs_c[0];
    assert!((29.4..39.4).contains(&settled), "{settled}");
    e.action("fault", "arm", "PECS_1", Some("pump-loss 1.0")).unwrap();
    run(&mut e, 120.0, &flight());
    assert!(e.pecs_c[0] > settled + 0.1, "{settled} {}", e.pecs_c[0]);
}

#[test]
fn every_located_breaker_is_a_protected_circuit_and_a_tcb_trips_on_real_overcurrent() {
    let mut e = powered();
    let n = e.plant.net().breaker_circuits().len();
    assert!(n > 200, "{n}");
    let tcb = e
        .circuit_prot
        .iter()
        .find(|c| matches!(c.device, super::breaker_link::Device::Thermal(_)) && c.load.contains("CT"))
        .map(|c| (c.id.clone(), c.load.clone()))
        .expect("a TCB circuit");
    run(&mut e, 1.0, &flight());
    assert!(
        e.circuit_protection_open().is_empty(),
        "{:?}",
        e.circuit_protection_open()
    );
    e.plant.set_load(&tcb.1, 20_000.0);
    run(&mut e, 30.0, &flight());
    assert!(e.circuit_protection_open().contains(&tcb.0));
    assert!(e.plant.net().load_demand_w(&tcb.1).unwrap() < 1.0);
}
