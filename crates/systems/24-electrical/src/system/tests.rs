use super::*;

fn data_dir() -> std::path::PathBuf {
    std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../../aircraft/24-electrical")
}

fn new_sys() -> Electrical {
    Electrical::new(&data_dir()).expect("system builds")
}

fn run(e: &mut Electrical, seconds: f64, inputs: &ElectricalInputs) {
    let steps = (seconds * 100.0) as u64;
    for _ in 0..steps {
        e.step(10_000, inputs);
    }
}

fn gens_normal(e: &mut Electrical) {
    for i in 0..4 {
        e.panel_mut().gen_ctrl[i].set_latched(true);
    }
    for i in 0..2 {
        e.panel_mut().apu_gen[i].set_latched(true);
    }
}

fn engines_inputs() -> ElectricalInputs {
    ElectricalInputs {
        engine_rpm: [7600.0; 2],
        engine_running: [true; 2],
        on_ground: false,
        radio_alt_ft: 20000.0,
        ..ElectricalInputs::default()
    }
}

#[test]
fn cold_and_dark_is_dead() {
    let mut e = new_sys();
    run(&mut e, 2.0, &ElectricalInputs::default());
    assert_eq!(e.flag("elec.bus.L1_235.energized"), Some(false));
    assert_eq!(e.flag("elec.mbr.closed"), Some(false));
    assert_eq!(e.text("elec.mode").as_deref(), Some("cold_dark"));
}

#[test]
fn battery_on_powers_only_dc_battery_loads() {
    let mut e = new_sys();
    e.action("crew", "press", "BATTERY", Some("ON")).unwrap();
    run(&mut e, 2.0, &ElectricalInputs::default());
    assert_eq!(e.flag("elec.mbr.closed"), Some(true));
    assert_eq!(e.flag("elec.bus.hbb_main.energized"), Some(true));
    for b in ["L1_235", "L2_235", "R1_235", "R2_235"] {
        assert_eq!(e.flag(&format!("elec.bus.{b}.energized")), Some(false));
    }
    assert_eq!(e.text("elec.mode").as_deref(), Some("on_ground_battery_only"));
}

#[test]
fn forward_external_power_goes_through_ground_handling_to_on() {
    let mut e = new_sys();
    gens_normal(&mut e);
    e.action("crew", "press", "BATTERY", Some("ON")).unwrap();
    e.action(
        "ground",
        "connect",
        "EXT_PWR_FWD_L",
        Some("v_ll=115 hz=400 kva=90"),
    )
    .unwrap();
    let inputs = e.inputs().clone();
    run(&mut e, 2.0, &inputs);
    assert_eq!(e.flag("elec.ext.fwd_l.avail"), Some(true));
    assert_eq!(e.flag("elec.bus.L1_235.energized"), Some(false));
    e.action("crew", "press", "EXT_PWR_FWD_L", None).unwrap();
    run(&mut e, 3.0, &inputs);
    assert_eq!(e.flag("elec.epc.fwd_l.closed"), Some(true));
    assert_eq!(e.flag("elec.bus.L1_235.energized"), Some(true));
}

#[test]
fn engine_vfsg_comes_online() {
    let mut e = new_sys();
    gens_normal(&mut e);
    e.action("crew", "press", "BATTERY", Some("ON")).unwrap();
    let mut i = ElectricalInputs {
        engine_rpm: [7600.0; 2],
        engine_running: [true; 2],
        ..ElectricalInputs::default()
    };
    run(&mut e, 4.0, &i);
    assert_eq!(e.flag("elec.gcb.l1.closed"), Some(true));
    assert_eq!(e.flag("elec.bus.L1_235.energized"), Some(true));
    i.engine_rpm = [7600.0; 2];
    assert_eq!(e.text("elec.vfsg.l1.mode").as_deref(), Some("generator"));
    assert!(e.eicas_messages().iter().all(|m| !m.contains("AC BUS L1")));
}

#[test]
fn loss_of_vfsg_in_flight_is_picked_up_by_a_tie() {
    let mut e = new_sys();
    gens_normal(&mut e);
    e.action("crew", "press", "BATTERY", Some("ON")).unwrap();
    let i = engines_inputs();
    run(&mut e, 6.0, &i);
    assert_eq!(e.flag("elec.bus.L1_235.energized"), Some(true));
    e.action("fault", "gen_loss", "L1_VFSG", None).unwrap();
    run(&mut e, 6.0, &i);
    assert_eq!(e.flag("elec.bus.L1_235.energized"), Some(true));
    assert_eq!(e.flag("elec.gcb.l1.closed"), Some(false));
}

#[test]
fn all_ac_lost_requests_rat_after_15_s() {
    let mut e = new_sys();
    gens_normal(&mut e);
    e.action("crew", "press", "BATTERY", Some("ON")).unwrap();
    let i = engines_inputs();
    run(&mut e, 6.0, &i);
    e.action("fault", "gen_loss", "ALL_VFSG", None).unwrap();
    let dead = ElectricalInputs {
        engine_rpm: [0.0; 2],
        engine_running: [false; 2],
        ..i
    };
    run(&mut e, 5.0, &dead);
    assert_eq!(e.flag("elec.rat.deploy_requested"), Some(false));
    run(&mut e, 14.0, &dead);
    assert_eq!(e.flag("elec.rat.deploy_requested"), Some(true));
}

#[test]
fn ac_bus_caution_appears_when_bus_dies() {
    let mut e = new_sys();
    gens_normal(&mut e);
    e.action("crew", "press", "BATTERY", Some("ON")).unwrap();
    let i = engines_inputs();
    run(&mut e, 6.0, &i);
    e.action("fault", "bus_fail", "L1", None).unwrap();
    run(&mut e, 2.0, &i);
    assert!(e.eicas_messages().iter().any(|m| m.contains("AC BUS L1")));
    assert_eq!(e.flag("elec.msg.ELEC_AC_BUS_L1"), Some(true));
}

#[test]
fn deterministic() {
    let mut a = new_sys();
    let mut b = new_sys();
    for e in [&mut a, &mut b] {
        gens_normal(e);
        e.action("crew", "press", "BATTERY", Some("ON")).unwrap();
        run(e, 3.0, &engines_inputs());
        e.action("fault", "gen_loss", "L2_VFSG", None).unwrap();
        run(e, 3.0, &engines_inputs());
    }
    let sa = a.snapshot();
    let sb = b.snapshot();
    assert_eq!(sa.len(), sb.len());
    for (x, y) in sa.iter().zip(sb.iter()) {
        match (&x.1, &y.1) {
            (SignalValue::Num(p), SignalValue::Num(q)) => assert_eq!(p.to_bits(), q.to_bits()),
            (p, q) => assert_eq!(p, q),
        }
    }
    assert_eq!(a.eicas_messages(), b.eicas_messages());
}

#[test]
fn apu_gens_online_then_transfer_to_engines() {
    let mut e = new_sys();
    gens_normal(&mut e);
    e.action("crew", "press", "BATTERY", Some("ON")).unwrap();
    let mut i = ElectricalInputs {
        apu_rpm: 11_400.0,
        apu_running: true,
        ..ElectricalInputs::default()
    };
    run(&mut e, 6.0, &i);
    assert_eq!(e.flag("elec.asg.l.gen_on_bus"), Some(true));
    assert_eq!(e.flag("elec.bus.L1_235.energized"), Some(true));
    assert_eq!(e.flag("elec.bus.R2_235.energized"), Some(true));
    assert_eq!(
        e.text("elec.bus.L1_235.source")
            .as_deref()
            .map(|s| s.ends_with("_ASG")),
        Some(true)
    );
    i.engine_rpm = [7600.0; 2];
    i.engine_running = [true; 2];
    run(&mut e, 8.0, &i);
    assert_eq!(e.flag("elec.gcb.l1.closed"), Some(true));
    assert_eq!(e.text("elec.bus.L1_235.source").as_deref(), Some("L1_VFSG"));
    assert_eq!(e.flag("elec.bus.L1_235.energized"), Some(true));
}

fn ext_fwd(e: &mut Electrical, which: &str) {
    e.action("ground", "connect", which, Some("v_ll=115 hz=400 kva=90"))
        .unwrap();
}

#[test]
fn drive_disconnect_needs_50ms_of_solenoid_and_overheats_past_3s() {
    let mut e = new_sys();
    gens_normal(&mut e);
    e.action("crew", "press", "BATTERY", Some("ON")).unwrap();
    let i = ElectricalInputs {
        engine_rpm: [7600.0; 2],
        engine_running: [true; 2],
        ..ElectricalInputs::default()
    };
    run(&mut e, 3.0, &i);
    e.press_drive_disconnect_for(0, 40_000);
    run(&mut e, 0.2, &i);
    assert!(!e.drive_disconnected[0]);
    e.press_drive_disconnect_for(1, 60_000);
    run(&mut e, 0.2, &i);
    assert!(e.drive_disconnected[1]);
    e.arm_fault("VFSG_R1", "disconnect-jam", 1.0).unwrap();
    run(&mut e, 0.1, &i);
    e.press_drive_disconnect_for(2, 3_500_000);
    run(&mut e, 4.0, &i);
    assert!(!e.drive_disconnected[2]);
    assert!(e.drive_sol[2].damaged);
}

#[test]
fn engine_start_on_one_forward_cart_is_inhibited() {
    let mut e = new_sys();
    gens_normal(&mut e);
    e.action("crew", "press", "BATTERY", Some("ON")).unwrap();
    ext_fwd(&mut e, "EXT_PWR_FWD_L");
    let inputs = e.inputs().clone();
    run(&mut e, 2.0, &inputs);
    e.action("crew", "press", "EXT_PWR_FWD_L", None).unwrap();
    run(&mut e, 3.0, &inputs);
    e.action("crew", "set", "ENG_START_L", Some("START")).unwrap();
    let inputs = e.inputs().clone();
    run(&mut e, 1.0, &inputs);
    assert_eq!(e.flag("elec.engine.l.start_inhibited"), Some(true));
    assert!(!e.crew_in.engine_start_in_progress[0]);
}

#[test]
fn starter_cooldown_halves_torque_after_three_minutes() {
    let mut e = new_sys();
    e.rig.vfsg[0].start_contactor = true;
    e.step_vfsg_duty_for_test(0, true, crate::system::rig::CMSC_MES_LIMIT_S + 0.1);
    assert!(e.rig.vfsg[0].cooldown_s > 0.0);
    assert!(e.signal("elec.vfsg.l1.cooldown_s").unwrap() > 1700.0);
}

#[test]
fn ground_handling_refuses_cart_above_440_hz() {
    let mut e = new_sys();
    e.action(
        "ground",
        "connect",
        "EXT_PWR_FWD_L",
        Some("v_ll=115 hz=445 kva=90"),
    )
    .unwrap();
    let inputs = e.inputs().clone();
    run(&mut e, 2.0, &inputs);
    assert_ne!(e.text("elec.mode").as_deref(), Some("ground_handling"));
}

#[test]
#[ignore]
fn print_cruise_load_budget() {
    let mut e = new_sys();
    gens_normal(&mut e);
    e.action("crew", "press", "BATTERY", Some("ON")).unwrap();
    e.action("sim", "set", "airplane_state", Some("cruise")).unwrap();
    let mut i = engines_inputs();
    i.galley_power = true;
    i.cargo_heat = true;
    i.ground_speed_kt = 480.0;
    run(&mut e, 20.0, &i);
    let snap = e.plant.snapshot().unwrap().clone();
    let mut tot = std::collections::BTreeMap::<String, (f64, usize, f64)>::new();
    for (name, ac, vn, demand) in e.plant.net().load_classes() {
        let p = snap.load(&name).map(|l| l.power_w).unwrap_or(0.0);
        let class = format!("{}{}", if ac { "AC" } else { "DC" }, vn.round());
        let group = if name.starts_with("LM_") || name.starts_with("CMSC") {
            "managed".to_string()
        } else if name.starts_with("LOAD_") {
            "lumped".to_string()
        } else {
            "panel".to_string()
        };
        let t = tot.entry(format!("{class} {group}")).or_insert((0.0, 0, 0.0));
        t.0 += p;
        t.1 += 1;
        t.2 += demand;
        if group != "panel" {
            eprintln!("  {name} {class} demand {demand:.0} actual {p:.0}");
        }
    }
    for (k, v) in tot {
        eprintln!(
            "{k}: actual {:.1} kW demand {:.1} kW n {}",
            v.0 / 1000.0,
            v.2 / 1000.0,
            v.1
        );
    }
}
