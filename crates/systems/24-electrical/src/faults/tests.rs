use super::*;

#[test]
fn registry_audit_passes() {
    let r = catalog();
    assert!(r.audit().is_ok());
    assert!(r.unobservable().is_empty());
}

#[test]
fn no_duplicate_catalog_entries() {
    assert_eq!(catalog().total_modes(), CATALOG.len());
}

#[test]
fn observables_use_known_prefixes() {
    for (p, m, obs) in CATALOG {
        for o in *obs {
            assert!(
                OBSERVABLE_PREFIXES.iter().any(|x| o.starts_with(x)),
                "{p}/{m}: {o}"
            );
        }
    }
}

#[test]
fn every_part_type_has_modes() {
    let required = [
        VFSG,
        ASG,
        RAT_GEN,
        GCU,
        BPCU,
        BATTERY_CELL,
        BATTERY_PACK,
        BMU,
        CHARGER,
        DIODE_MODULE,
        ATRU,
        ATU,
        TRU,
        SPU,
        CMSC,
        CONTACTOR,
        ELCC,
        SSPC,
        TCB,
        RPDU_CONTROLLER,
        RPDU_MODULE,
        SPDU_CARD,
        WIRE_SEGMENT,
        CONNECTOR,
        GROUND_STUD,
        PANEL_SWITCH,
        LAMP,
        EICAS_INPUT,
    ];
    let r = catalog();
    for t in required {
        assert!(!r.modes_for(t).is_empty(), "{t}");
    }
    assert!(r.part_types().len() >= required.len());
    assert_eq!(r.part_types().len(), part_types().len());
}

#[test]
fn required_modes_present() {
    let r = catalog();
    for (p, m) in [
        (CONTACTOR, "weld"),
        (CONTACTOR, "coil-open"),
        (CONTACTOR, "aux-stuck"),
        (CONTACTOR, "high-resistance"),
        (SSPC, "stuck-open"),
        (SSPC, "stuck-closed"),
        (SSPC, "spurious-trip"),
        (SSPC, "arc-detector-false-trip"),
        (SSPC, "telemetry-bias"),
        (WIRE_SEGMENT, "chafe-to-ground"),
        (WIRE_SEGMENT, "open"),
        (WIRE_SEGMENT, "high-resistance"),
        (BATTERY_CELL, "internal-short"),
        (BATTERY_CELL, "capacity-fade"),
        (BATTERY_CELL, "thermal-runaway-precursor"),
        (VFSG, "bearing-wear"),
        (VFSG, "rotating-diode-short"),
        (VFSG, "pmg-failure"),
        (GCU, "regulator-drift-high"),
        (GCU, "cpu-fail"),
        (GCU, "counter-overflow-248d"),
    ] {
        assert!(r.supports(p, m), "{p}/{m}");
    }
}

#[test]
fn arm_rejects_unknown_mode_and_type_mix() {
    let mut b = FaultBook::new();
    assert!(b.arm_immediate("K1", CONTACTOR, "nonsense", 1.0, 0).is_err());
    assert!(b.arm_immediate("K1", CONTACTOR, "weld", 1.0, 0).is_ok());
    assert!(b.arm_immediate("K1", SSPC, "stuck-open", 1.0, 0).is_err());
}

#[test]
fn arm_clear_and_magnitude_per_instance() {
    let mut b = FaultBook::new();
    b.arm_immediate("L1 GCB", CONTACTOR, "weld", 1.0, 100).unwrap();
    assert_eq!(b.magnitude("L1 GCB", "weld", 99), 0.0);
    assert_eq!(b.magnitude("L1 GCB", "weld", 100), 1.0);
    assert_eq!(b.magnitude("L2 GCB", "weld", 100), 0.0);
    assert!(b.clear("L1 GCB", "weld"));
    assert!(!b.clear("L1 GCB", "weld"));
    assert!(b.is_empty());
}

#[test]
fn contactor_mapping() {
    let mut b = FaultBook::new();
    b.arm_immediate("K", CONTACTOR, "weld", 1.0, 0).unwrap();
    b.arm_immediate("K", CONTACTOR, "high-resistance", 0.5, 0)
        .unwrap();
    let e = b.contactor("K", 10);
    assert!(e.welded && !e.coil_open);
    assert!((e.resistance_mult - 50.5).abs() < 1e-9);
    let clean = b.contactor("other", 10);
    assert!(!clean.welded);
    assert_eq!(clean.resistance_mult, 1.0);
    assert_eq!(clean.operate_time_mult, 1.0);
}

#[test]
fn binary_threshold() {
    let weak = Magnitudes::from_pairs(&[("weld", 0.4)]);
    let strong = Magnitudes::from_pairs(&[("weld", 0.5)]);
    assert!(!contactor_effect(&weak).welded);
    assert!(contactor_effect(&strong).welded);
}

#[test]
fn wire_ground_fault_resistance_falls_with_magnitude() {
    assert_eq!(ground_fault_resistance_ohm(0.0), None);
    let a = ground_fault_resistance_ohm(0.2).unwrap();
    let b = ground_fault_resistance_ohm(0.9).unwrap();
    assert!(a > b);
    assert!((ground_fault_resistance_ohm(1.0).unwrap() - 0.05).abs() < 1e-9);
    let e = wire_effect(&Magnitudes::from_pairs(&[
        ("open", 1.0),
        ("chafe-to-ground", 0.5),
    ]));
    assert!(e.open && e.ground_fault_ohm.is_some());
}

#[test]
fn sspc_telemetry_bias_and_flags() {
    let e = sspc_effect(&Magnitudes::from_pairs(&[
        ("telemetry-bias", 1.0),
        ("stuck-open", 1.0),
    ]));
    assert!((e.current_telemetry_scale - 1.25).abs() < 1e-9);
    assert!(e.stuck_open && !e.stuck_closed);
}

#[test]
fn battery_cell_mapping() {
    let e = battery_cell_effect(&Magnitudes::from_pairs(&[
        ("capacity-fade", 0.5),
        ("thermal-runaway-precursor", 0.8),
        ("internal-short", 0.9),
    ]));
    assert!((e.capacity_scale - 0.7).abs() < 1e-9);
    assert!(e.r0_mult > 1.5);
    assert!(e.runaway_precursor && e.extra_heat_w > 50.0);
    assert!(e.internal_short_ohm.is_some());
}

#[test]
fn gcu_drift_and_counter_overflow() {
    let hi = gcu_effect(&Magnitudes::from_pairs(&[("regulator-drift-high", 0.5)]));
    let lo = gcu_effect(&Magnitudes::from_pairs(&[("regulator-drift-low", 0.5)]));
    assert!(hi.setpoint_offset_v > 0.0 && lo.setpoint_offset_v < 0.0);
    let full = gcu_effect(&Magnitudes::from_pairs(&[("counter-overflow-248d", 1.0)]));
    assert_eq!(full.powered_time_preload_us, COUNTER_LIMIT_TICKS * TICK_US);
    assert_eq!(gcu_effect(&Magnitudes::default()).powered_time_preload_us, 0);
}

#[test]
fn machine_mapping() {
    let e = machine_effect(&Magnitudes::from_pairs(&[
        ("bearing-wear", 0.4),
        ("pmg-failure", 1.0),
    ]));
    assert!(e.friction_loss_w > 0.0 && e.vibration > 0.0);
    assert_eq!(e.pmg_scale, 0.0);
}

#[test]
fn intermittent_is_deterministic_and_partial() {
    let mk = || {
        let mut b = FaultBook::new();
        b.arm_intermittent("W12", WIRE_SEGMENT, "chafe-to-ground", 1.0, (1000, 0.3), 0)
            .unwrap();
        b
    };
    let (a, b) = (mk(), mk());
    let mut on = 0;
    for slot in 0..4000u64 {
        let t = slot * 1000 + 10;
        let (x, y) = (
            a.magnitude("W12", "chafe-to-ground", t),
            b.magnitude("W12", "chafe-to-ground", t),
        );
        assert_eq!(x, y);
        if x > 0.0 {
            on += 1;
        }
    }
    let frac = f64::from(on) / 4000.0;
    assert!((frac - 0.3).abs() < 0.05, "{frac}");
    let mut other = FaultBook::new();
    other
        .arm_intermittent("W13", WIRE_SEGMENT, "chafe-to-ground", 1.0, (1000, 0.3), 0)
        .unwrap();
    let differs = (0..200u64).any(|s| {
        other.magnitude("W13", "chafe-to-ground", s * 1000) != a.magnitude("W12", "chafe-to-ground", s * 1000)
    });
    assert!(differs);
}

#[test]
fn latent_trigger_and_active_observables() {
    let mut b = FaultBook::new();
    b.arm(
        "K1",
        CONTACTOR,
        "stuck-open",
        1.0,
        Onset::Latent {
            trigger: "cold-soak".into(),
        },
        0,
    )
    .unwrap();
    assert!(b.active_observables("K1", 10).is_empty());
    assert_eq!(b.trigger("cold-soak", 20), 1);
    let obs = b.active_observables("K1", 30);
    assert!(obs.iter().any(|o| o == "eicas:ELEC GEN OFF"));
    assert_eq!(b.active(30).len(), 1);
}
