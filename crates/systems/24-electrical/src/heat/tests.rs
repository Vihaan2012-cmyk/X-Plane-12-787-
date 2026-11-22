use super::*;

const SEC: u64 = 1_000_000;

fn run(t: &mut ElecThermal, seconds: u64) {
    for _ in 0..seconds {
        t.step(SEC);
    }
}

#[test]
fn wire_at_limit_current_settles_at_conductor_limit() {
    let mut t = ElecThermal::new();
    let s = t.add_wire("W1", Zone::EeBayFwd, 20.0, 2.0).unwrap();
    assert_eq!(s.gauge.awg, "18");
    t.set_zone_temp(Zone::EeBayFwd, AMBIENT_REF_K);
    let amb = t.zone_temp_k(Zone::EeBayFwd);
    assert!((amb - AMBIENT_REF_K).abs() < 1e-9);
    let limit = t.ampacity_a("W1").unwrap();
    assert!(limit >= 20.0);
    t.set_current("W1", limit);
    run(&mut t, 1500);
    let temp = t.temperature_k("W1").unwrap();
    assert!((temp - MAX_CONDUCTOR_K).abs() < 3.0, "{temp}");
}

#[test]
fn wire_at_rating_stays_below_limit_and_above_ambient() {
    let mut t = ElecThermal::new();
    t.add_wire("W1", Zone::Crown, 20.0, 2.0).unwrap();
    t.set_current("W1", 20.0);
    run(&mut t, 1500);
    let temp = t.temperature_k("W1").unwrap();
    assert!(temp > t.zone_temp_k(Zone::Crown) + 5.0);
    assert!(t.margin_to_limit_k("W1").unwrap() > 0.0);
    assert!(t.resistance_factor("W1").unwrap() > 1.1);
}

#[test]
fn large_feeder_uses_parallel_conductors() {
    let s = size_wire(1200.0);
    assert!(s.parallel >= 3);
    assert!(s.limit_a >= 1200.0);
}

#[test]
fn contactor_heats_with_current_and_resistance_rises() {
    let mut t = ElecThermal::new();
    t.add_contactor("GCB", Zone::EeBayAft, 225.0).unwrap();
    let r0 = t.resistance_ohm("GCB").unwrap();
    let t0 = t.temperature_k("GCB").unwrap();
    t.set_current("GCB", 225.0);
    run(&mut t, 1800);
    assert!(t.temperature_k("GCB").unwrap() > t0 + 5.0);
    assert!(t.resistance_ohm("GCB").unwrap() > r0);
    assert!(t.secondary_temperature_k("GCB").unwrap() < t.temperature_k("GCB").unwrap());
}

#[test]
fn high_resistance_multiplier_heats_more() {
    let mut a = ElecThermal::new();
    let mut b = ElecThermal::new();
    a.add_contactor("C", Zone::EeBayAft, 100.0).unwrap();
    b.add_contactor("C", Zone::EeBayAft, 100.0).unwrap();
    b.set_resistance_multiplier("C", 10.0);
    a.set_current("C", 100.0);
    b.set_current("C", 100.0);
    run(&mut a, 600);
    run(&mut b, 600);
    assert!(b.temperature_k("C").unwrap() > a.temperature_k("C").unwrap() + 10.0);
}

#[test]
fn battery_warms_under_discharge() {
    let mut t = ElecThermal::new();
    t.add_battery("MAIN", Zone::EeBayFwd, 8, 2.0, 0.0012).unwrap();
    let t0 = t.temperature_k("MAIN").unwrap();
    t.set_current("MAIN", 200.0);
    run(&mut t, 600);
    assert!(t.temperature_k("MAIN").unwrap() > t0 + 3.0);
}

#[test]
fn cold_soak_chills_parts_and_raises_battery_resistance() {
    let mut t = ElecThermal::new();
    t.add_battery("MAIN", Zone::WheelWell, 8, 2.0, 0.0012).unwrap();
    t.add_wire("W", Zone::WheelWell, 10.0, 1.0).unwrap();
    t.set_zone_temp(Zone::WheelWell, 243.15);
    for _ in 0..720 {
        t.step(60 * SEC);
    }
    let tb = t.temperature_k("MAIN").unwrap();
    assert!(tb < 250.0, "{tb}");
    assert!(t.resistance_factor("MAIN").unwrap() > 1.5);
    assert!(t.resistance_factor("W").unwrap() < 1.0);
}

#[test]
fn cold_soak_sets_nodes_directly() {
    let mut t = ElecThermal::new();
    t.add_module("RPDU", Zone::Crown, ModuleKind::Rpdu, 15.0).unwrap();
    t.cold_soak(Zone::Crown, 233.15);
    assert!((t.temperature_k("RPDU").unwrap() - 233.15).abs() < 1e-9);
}

#[test]
fn module_dissipation_and_cooling_air() {
    let mut t = ElecThermal::new();
    t.add_module("RPDU", Zone::Crown, ModuleKind::Rpdu, 30.0).unwrap();
    run(&mut t, 3000);
    let hot = t.temperature_k("RPDU").unwrap();
    assert!(hot > t.zone_temp_k(Zone::Crown) + 5.0);
    t.set_cooling_flow("RPDU", 0.02);
    run(&mut t, 3000);
    assert!(t.temperature_k("RPDU").unwrap() < hot);
}

#[test]
fn generator_oil_heats_and_cooler_closes() {
    let mut t = ElecThermal::new();
    t.add_generator("VFSG", Zone::Nacelle, 225.0, 9000.0).unwrap();
    t.set_current("VFSG", 225.0);
    run(&mut t, 3000);
    let warm = t.temperature_k("VFSG").unwrap();
    t.set_oil_cooling("VFSG", 0.0);
    run(&mut t, 3000);
    assert!(t.temperature_k("VFSG").unwrap() > warm + 20.0);
    assert!(t.resistance_factor("VFSG").unwrap() > 1.2);
}

#[test]
fn energy_balances() {
    let mut t = ElecThermal::new();
    t.add_wire("W", Zone::Cabin, 20.0, 2.0).unwrap();
    t.add_battery("B", Zone::Cabin, 8, 2.0, 0.0012).unwrap();
    t.set_current("W", 20.0);
    t.set_current("B", 100.0);
    run(&mut t, 300);
    let e = t.energy_report();
    assert!(e.source_j > 0.0);
    assert!(e.imbalance_j.abs() < 1e-3 * e.source_j.max(1.0));
    assert!((e.elapsed_s - 300.0).abs() < 1e-9);
}

#[test]
fn from_parts_and_duplicates() {
    let specs = vec![
        PartSpec::Wire {
            id: "W".into(),
            zone: Zone::Crown,
            rating_a: 5.0,
            length_m: 3.0,
        },
        PartSpec::Contactor {
            id: "K".into(),
            zone: Zone::Crown,
            rating_a: 50.0,
        },
    ];
    let t = ElecThermal::from_parts(&specs).unwrap();
    assert_eq!(t.len(), 2);
    assert_eq!(t.kind_of("K"), Some(PartKind::Contactor));
    let dup = vec![specs[0].clone(), specs[0].clone()];
    assert!(matches!(
        ElecThermal::from_parts(&dup),
        Err(HeatError::DuplicateId(_))
    ));
}

#[test]
fn deterministic() {
    let build = || {
        let mut t = ElecThermal::new();
        t.add_wire("W", Zone::Crown, 20.0, 2.0).unwrap();
        t.set_current("W", 20.0);
        run(&mut t, 200);
        t.temperature_k("W").unwrap().to_bits()
    };
    assert_eq!(build(), build());
}
