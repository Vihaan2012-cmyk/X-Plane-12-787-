use crate::*;

const TA: f64 = 293.15;

fn copper_wire() -> WireSpec {
    WireSpec {
        conductor: Conductor::Copper,
        length_m: 1.0,
        area_m2: 2.5e-6,
        outer_diameter_m: 0.0035,
        surface_h_w_m2k: 10.0,
        extra_capacity_j_k: 0.0,
        initial_temperature_k: TA,
    }
}

#[test]
fn rc_step_response_matches_analytic() {
    let mut net = Network::new();
    let amb = net.add_boundary(300.0);
    let n = net.add_node(100.0, 400.0);
    net.add_conductance(n, amb, 2.0);
    let dt = 0.005;
    for _ in 0..10_000 {
        net.step(dt);
    }
    let expected = 300.0 + 100.0 * (-50.0_f64 / 50.0).exp();
    assert!((net.temperature(n) - expected).abs() < 0.1);
}

#[test]
fn two_node_equilibrium() {
    let mut net = Network::new();
    let amb = net.add_boundary(300.0);
    let a = net.add_node(50.0, 300.0);
    let b = net.add_node(80.0, 300.0);
    net.add_conductance(a, b, 5.0);
    net.add_conductance(b, amb, 2.0);
    net.set_source(a, 10.0);
    for _ in 0..2000 {
        net.step(1.0);
    }
    assert!((net.temperature(b) - 305.0).abs() < 1e-6);
    assert!((net.temperature(a) - 307.0).abs() < 1e-6);
}

#[test]
fn energy_is_conserved_per_step_and_in_total() {
    let mut net = Network::new();
    let amb = net.add_boundary(250.0);
    let cool = net.add_boundary(280.0);
    let a = net.add_node(500.0, 350.0);
    let b = net.add_node(300.0, 320.0);
    let c = net.add_node(40.0, 290.0);
    net.add_conductance(a, b, 3.0);
    let conv = net.add_convection(b, amb, 0.5, 0.3);
    net.set_airflow(conv, 5.0, 80_000.0);
    net.add_radiation(a, amb, 0.8, 0.4);
    net.add_conductance(c, cool, 1.0);
    net.add_conductance(c, b, 0.7);
    net.set_source(a, 120.0);
    net.set_source(c, -5.0);
    let start = net.stored_energy_j();
    let mut worst: f64 = 0.0;
    for _ in 0..500 {
        let r = net.step(0.7);
        worst = worst.max(r.imbalance_j.abs());
        assert!(r.energy_in_j >= 0.0 && r.energy_out_j >= 0.0);
    }
    assert!(worst < 1e-6);
    let delta = net.stored_energy_j() - start;
    assert!(delta.is_finite());
    assert!((net.total_in_j() - net.total_out_j() - delta).abs() < 1e-5 * net.total_in_j());
}

#[test]
fn wire_reaches_steady_temperature() {
    let spec = copper_wire();
    let mut net = Network::new();
    let amb = net.add_boundary(TA);
    let wire = WireSegment::add(&mut net, &spec, amb);
    let current = 20.0;
    for _ in 0..5000 {
        wire.apply(&mut net, current);
        net.step(1.0);
    }
    let r20 = 1.724e-8 / 2.5e-6;
    let g = spec.surface_conductance_w_k();
    let q = current * current * r20;
    let dt_k = q / g / (1.0 - q * 0.00393 / g);
    assert!((wire.temperature(&net) - TA - dt_k).abs() < 0.05);
    assert!(dt_k > 10.0);
}

#[test]
fn wire_resistance_rises_with_temperature() {
    let cold = conductor_resistance(Conductor::Copper, 1.0, 1e-6, 293.15);
    let hot = conductor_resistance(Conductor::Copper, 1.0, 1e-6, 373.15);
    assert!((cold - 0.01724).abs() < 1e-9);
    assert!((hot / cold - (1.0 + 0.00393 * 80.0)).abs() < 1e-9);
    let al = conductor_resistance(Conductor::Aluminium, 1.0, 1e-6, 293.15);
    assert!(al > cold);
}

#[test]
fn ampacity_reaches_limit_temperature() {
    let spec = copper_wire();
    let limit = TA + 40.0;
    let i = wire_ampacity(&spec, TA, limit);
    let mut net = Network::new();
    let amb = net.add_boundary(TA);
    let wire = WireSegment::add(&mut net, &spec, amb);
    for _ in 0..5000 {
        wire.apply(&mut net, i);
        net.step(1.0);
    }
    assert!((wire.temperature(&net) - limit).abs() < 0.05);
}

#[test]
fn bundle_runs_hotter_than_single_wire() {
    let spec = copper_wire();
    let mut net = Network::new();
    let amb = net.add_boundary(TA);
    let single = WireSegment::add(&mut net, &spec, amb);
    let bundle_spec = BundleSpec {
        diameter_m: 0.012,
        length_m: 1.0,
        surface_h_w_m2k: 10.0,
        capacity_j_k: 5.0,
        initial_temperature_k: TA,
    };
    let bundle = WireBundle::add(&mut net, &bundle_spec, &spec, 12, amb);
    let mut isolated = Network::new();
    let iamb = isolated.add_boundary(TA);
    let alone = WireSegment::add(&mut isolated, &spec, iamb);
    for _ in 0..4000 {
        single.apply(&mut net, 10.0);
        bundle.apply_uniform(&mut net, 10.0);
        alone.apply(&mut isolated, 10.0);
        net.step(1.0);
        isolated.step(1.0);
    }
    let t_bundle = bundle.wires()[0].temperature(&net);
    assert!(t_bundle > alone.temperature(&isolated) + 1.0);
    assert!(bundle.temperature(&net) > TA);
    assert!(t_bundle > bundle.temperature(&net));
}

#[test]
fn cold_soak_follows_exponential() {
    let mut net = Network::new();
    let amb = net.add_boundary(253.15);
    let n = net.add_node(1000.0, 303.15);
    net.add_conductance(n, amb, 2.0);
    let tau = soak_time_constant(1000.0, 2.0);
    let mut t = 0.0;
    for _ in 0..3600 {
        net.step(0.5);
        t += 0.5;
    }
    let expected = soak_temperature(303.15, 253.15, tau, t);
    assert!((net.temperature(n) - expected).abs() < 0.1);
}

#[test]
fn stable_for_huge_dt() {
    let mut net = Network::new();
    let amb = net.add_boundary(250.0);
    let a = net.add_node(1.0e6, 600.0);
    let b = net.add_node(1.0e-3, 100.0);
    net.add_conductance(a, b, 1.0e4);
    net.add_conductance(b, amb, 5.0);
    net.add_radiation(a, amb, 0.9, 1.0);
    net.set_source(a, 50.0);
    for dt in [1.0e3, 1.0e6, 1.0e9, 1.0e12] {
        let r = net.step(dt);
        assert!(net.temperature(a).is_finite() && net.temperature(b).is_finite());
        assert!(net.temperature(a) > 250.0 && net.temperature(a) < 700.0);
        assert!(r.imbalance_j.abs() <= 1e-6 * (r.energy_in_j + r.energy_out_j + 1.0));
    }
    for _ in 0..30 {
        net.step(1.0e12);
    }
    let rad = 0.9 * STEFAN_BOLTZMANN * 1.0 * (net.temperature(a).powi(4) - 250.0_f64.powi(4));
    let cond = 5.0 * (net.temperature(b) - 250.0);
    assert!((rad + cond - 50.0).abs() < 1e-3);
}

#[test]
fn radiation_balances_source() {
    let mut net = Network::new();
    let sky = net.add_boundary(250.0);
    let n = net.add_node(200.0, 250.0);
    net.add_radiation(n, sky, 0.85, 0.5);
    net.set_source(n, 100.0);
    for _ in 0..3000 {
        net.step(2.0);
    }
    let rad = 0.85 * STEFAN_BOLTZMANN * 0.5 * (net.temperature(n).powi(4) - 250.0_f64.powi(4));
    assert!((rad - 100.0).abs() < 1e-3);
}

#[test]
fn convection_varies_with_flow_and_pressure() {
    let still = convection_coefficient(0.0, 101_325.0, 0.1, 300.0, 20.0);
    let slow = convection_coefficient(2.0, 101_325.0, 0.1, 300.0, 20.0);
    let fast = convection_coefficient(20.0, 101_325.0, 0.1, 300.0, 20.0);
    let thin = convection_coefficient(20.0, 25_000.0, 0.1, 300.0, 20.0);
    assert!(still > 1.0 && still < 20.0);
    assert!(slow > still && fast > slow);
    assert!(thin < fast);
    let none = convection_coefficient(0.0, 0.0, 0.1, 300.0, 20.0);
    assert!(none.abs() < 1e-12);
}

#[test]
fn airflow_cools_a_hot_part() {
    let run = |velocity: f64| {
        let mut net = Network::new();
        let amb = net.add_boundary(290.0);
        let n = net.add_node(100.0, 290.0);
        let l = net.add_convection(n, amb, 0.2, 0.2);
        net.set_airflow(l, velocity, 101_325.0);
        net.set_source(n, 30.0);
        for _ in 0..3000 {
            net.step(5.0);
        }
        net.temperature(n)
    };
    assert!(run(10.0) < run(0.0) - 5.0);
}

#[test]
fn lru_cooling_flow_lowers_core_temperature() {
    let run = |flow: f64| {
        let mut net = Network::new();
        let amb = net.add_boundary(300.0);
        let inlet = net.add_boundary(290.0);
        let spec = LruSpec {
            core_capacity_j_k: 800.0,
            shell_capacity_j_k: 1500.0,
            core_to_shell_w_k: 4.0,
            shell_to_ambient_w_k: 3.0,
            cooling_ua_w_k: 8.0,
            dissipation_w: 150.0,
            initial_temperature_k: 300.0,
        };
        let lru = LruBox::add(&mut net, &spec, amb, inlet);
        lru.set_cooling_flow(&mut net, flow);
        for _ in 0..2000 {
            net.step(10.0);
        }
        (lru.core_temperature(&net), lru.cooling_conductance(&net))
    };
    let (t0, g0) = run(0.0);
    let (t1, g1) = run(0.02);
    assert!(g0 == 0.0 && g1 > 0.0);
    assert!(t1 < t0 - 20.0);
    assert!(t0 > 300.0);
}

#[test]
fn battery_and_contact_heating() {
    let mut net = Network::new();
    let amb = net.add_boundary(TA);
    let spec = BatterySpec {
        mass_kg: 1.0,
        specific_heat_j_kg_k: 1000.0,
        resistance_ohm_20c: 0.01,
        resistance_cold_coeff_per_k: 0.03,
        entropic_v_per_k: 0.0,
        conductance_to_sink_w_k: 0.5,
        initial_temperature_k: TA,
    };
    let cell = BatteryCell::add(&mut net, &spec, amb);
    assert!((cell.resistance_at(TA) - 0.01).abs() < 1e-12);
    assert!(cell.resistance_at(TA - 30.0) > 0.02);
    let q = cell.apply(&mut net, 50.0);
    assert!((q - 25.0).abs() < 1e-9);
    for _ in 0..4000 {
        cell.apply(&mut net, 50.0);
        net.step(5.0);
    }
    assert!(cell.temperature(&net) > TA + 10.0);
    let contact_spec = ContactSpec {
        resistance_ohm_20c: 0.001,
        alpha_per_k: 0.0039,
        capacity_j_k: 20.0,
        conductance_to_case_w_k: 0.5,
        initial_temperature_k: TA,
    };
    let contact = ContactorContact::add(&mut net, &contact_spec, amb);
    let qc = contact.apply(&mut net, 100.0, 2.0);
    assert!((qc - 12.0).abs() < 1e-9);
    for _ in 0..2000 {
        contact.apply(&mut net, 100.0, 2.0);
        net.step(5.0);
    }
    let expected_min = TA + 12.0 / 0.5;
    assert!(contact.temperature(&net) > expected_min);
}

#[test]
fn determinism_bitwise() {
    let build = || {
        let mut net = Network::new();
        let amb = net.add_boundary(260.0);
        let a = net.add_node(300.0, 330.0);
        let b = net.add_node(120.0, 310.0);
        let l = net.add_convection(a, amb, 0.3, 0.2);
        net.set_airflow(l, 3.0, 90_000.0);
        net.add_conductance(a, b, 2.5);
        net.add_radiation(b, amb, 0.7, 0.2);
        net.set_source(b, 40.0);
        (net, a, b)
    };
    let (mut n1, a1, b1) = build();
    let (mut n2, a2, b2) = build();
    for _ in 0..1000 {
        n1.step(0.37);
        n2.step(0.37);
    }
    assert_eq!(n1.temperature(a1).to_bits(), n2.temperature(a2).to_bits());
    assert_eq!(n1.temperature(b1).to_bits(), n2.temperature(b2).to_bits());
}

#[test]
fn zero_capacity_isolated_node_is_held() {
    let mut net = Network::new();
    let n = net.add_node(0.0, 321.0);
    net.step(1.0);
    assert_eq!(net.temperature(n), 321.0);
    let r = net.step(-1.0);
    assert_eq!(r.dt_s, 0.0);
}
