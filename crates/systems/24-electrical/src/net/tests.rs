use super::*;

fn run(net: &mut ElectricalNet, steps: usize) -> Snapshot {
    let mut last = net.step(10_000).expect("solve");
    for _ in 1..steps {
        last = net.step(10_000).expect("solve");
    }
    last
}

fn close(net: &mut ElectricalNet, names: &[&str]) {
    for n in names {
        assert!(net.set_contactor(n, true), "{n}");
    }
}

fn asg_left(net: &mut ElectricalNet) {
    assert!(net.set_machine_rpm("ASG_L", 10_800.0));
    close(net, &["L APB", "L2 BTB"]);
}

#[test]
fn builds_from_data_with_full_counts() {
    let net = ElectricalNet::new();
    assert!(net.bus_names().len() >= 50, "{}", net.bus_names().len());
    assert!(net.contactor_names().len() >= 100);
    assert_eq!(net.machine_names().len(), 7);
    assert!(net.load_names().len() >= 100);
    assert!(net.sspc_ids().len() >= 100);
    for n in [
        "L1 GCB",
        "L3 BTB",
        "MBR",
        "CIT",
        "RCB",
        "SPUB",
        "SPU contactor",
        "LAEPC",
    ] {
        assert!(net.contactor_names().iter().any(|c| c == n), "{n}");
    }
    println!(
        "buses {} contactors {} machines {} loads {} sspcs {} nodes {}",
        net.bus_names().len(),
        net.contactor_names().len(),
        net.machine_names().len(),
        net.load_names().len(),
        net.sspc_ids().len(),
        net.node_count()
    );
}

#[test]
fn battery_only_powers_hot_buses() {
    let mut net = ElectricalNet::new();
    let s = run(&mut net, 5);
    for name in ["HBB_MAIN", "HBB_APU"] {
        let b = s.bus(name).unwrap();
        assert!(b.live, "{name}");
        assert!(b.voltage_v > 24.0 && b.voltage_v < 36.0, "{name} {}", b.voltage_v);
    }
    for name in [
        "L1", "L2", "R1", "R2", "LTB", "RTB", "BB", "DC28_L", "DC28_R", "CIB", "FIB", "HVDC_L1", "AC115_L",
    ] {
        assert!(!s.bus(name).unwrap().live, "{name}");
    }
    close(&mut net, &["MBR"]);
    let s = run(&mut net, 5);
    assert!(s.bus("CIB").unwrap().live);
    assert!(s.bus("FIB").unwrap().live);
}

#[test]
fn forward_external_power_through_atu_reaches_235_buses() {
    let mut net = ElectricalNet::new();
    assert!(net.set_external_power("L FWD", true));
    close(&mut net, &["L EPC", "L BSB", "L ATUC"]);
    let s = run(&mut net, 5);
    assert!(s.bus("AC115_L").unwrap().live);
    let l1 = s.bus("L1").unwrap();
    assert!(l1.live);
    assert!((l1.voltage_v - 235.0).abs() < 8.0, "{}", l1.voltage_v);
    assert!(!s.bus("L2").unwrap().live);
    close(&mut net, &["L3 BTB"]);
    let s = run(&mut net, 5);
    assert!(s.bus("L2").unwrap().live);
}

#[test]
fn asg_tie_bus_to_main_bus() {
    let mut net = ElectricalNet::new();
    asg_left(&mut net);
    let s = run(&mut net, 400);
    for name in ["LTB", "L2"] {
        let b = s.bus(name).unwrap();
        assert!(b.live, "{name}");
        assert!((b.voltage_v - 235.0).abs() < 8.0, "{name} {}", b.voltage_v);
        assert!((b.freq_hz - 360.0).abs() < 1.0);
    }
    for name in ["L1", "R1", "R2", "RTB"] {
        assert!(!s.bus(name).unwrap().live, "{name}");
    }
}

#[test]
fn tru_makes_28v_and_atru_makes_270v() {
    let mut net = ElectricalNet::new();
    asg_left(&mut net);
    close(&mut net, &["L3 BTB", "L TRU RLY", "L1 ATRUC", "L2 ATRUC"]);
    let s = run(&mut net, 400);
    let dc = s.bus("DC28_L").unwrap();
    assert!(dc.live);
    assert!((dc.voltage_v - 28.0).abs() < 3.0, "{}", dc.voltage_v);
    for name in ["HVDC_L1", "HVDC_L2"] {
        let h = s.bus(name).unwrap();
        assert!(h.live, "{name}");
        assert!((h.voltage_v - 270.0).abs() < 15.0, "{name} {}", h.voltage_v);
    }
    assert!(!s.bus("HVDC_R1").unwrap().live);
    assert!(net.set_load("CMSC_CAC_L1", 50_000.0));
    let s = run(&mut net, 20);
    assert!(s.load("CMSC_CAC_L1").unwrap().current_a > 100.0);
}

#[test]
fn rat_feeds_backup_bus() {
    let mut net = ElectricalNet::new();
    assert!(net.set_machine_rpm("RAT", 8000.0));
    close(&mut net, &["RCB"]);
    let s = run(&mut net, 400);
    let bb = s.bus("BB").unwrap();
    assert!(bb.live);
    assert!((bb.voltage_v - 230.0).abs() < 15.0, "{}", bb.voltage_v);
    close(&mut net, &["C1 TRU RELAY"]);
    let s = run(&mut net, 20);
    assert!(s.bus("CIB").unwrap().live);
    assert!(s.bus("FIB").unwrap().live);
}

#[test]
fn rpdu_channel_draws_when_sspc_closes() {
    let mut net = ElectricalNet::new();
    asg_left(&mut net);
    close(
        &mut net,
        &["L3 BTB", "L TRU RLY", "L ATUC", "L BSB", "AC FEED RPDU 71"],
    );
    let id = net
        .sspc_ids()
        .iter()
        .find(|i| i.starts_with("RPDU71."))
        .cloned()
        .expect("rpdu 71 channel");
    let s = run(&mut net, 300);
    assert!(s.bus("RPDU 71 DC").unwrap().live);
    assert!(s.sspc(&id).unwrap().current_a < 1e-9);
    assert!(net.set_sspc(&id, true));
    let s = run(&mut net, 5);
    assert!(
        s.sspc(&id).unwrap().current_a > 0.1,
        "{}",
        s.sspc(&id).unwrap().current_a
    );
    assert!((s.sspc(&id).unwrap().voltage_v - 28.0).abs() < 3.0);
}

#[test]
fn spu_starts_from_apu_battery() {
    let mut net = ElectricalNet::new();
    close(&mut net, &["SPU contactor", "SPUB"]);
    let s = run(&mut net, 10);
    let h = s.bus("HVDC_R2").unwrap();
    assert!(h.live);
    assert!((h.voltage_v - 270.0).abs() < 10.0, "{}", h.voltage_v);
}

#[test]
fn repeated_runs_are_bit_identical() {
    let go = || {
        let mut net = ElectricalNet::new();
        close(&mut net, &["MBR"]);
        asg_left(&mut net);
        close(&mut net, &["L3 BTB", "L TRU RLY", "L1 ATRUC"]);
        run(&mut net, 100)
    };
    let a = go();
    let b = go();
    assert_eq!(a, b);
    for (x, y) in a.buses.iter().zip(b.buses.iter()) {
        assert_eq!(x.voltage_v.to_bits(), y.voltage_v.to_bits());
    }
}

#[test]
fn unknown_names_are_rejected() {
    let mut net = ElectricalNet::new();
    assert!(!net.set_contactor("NOPE", true));
    assert!(!net.set_machine_rpm("NOPE", 1.0));
    assert!(!net.set_machine_field("NOPE", 1.0));
    assert!(!net.set_external_power("NOPE", true));
    assert!(!net.set_load("NOPE", 1.0));
    assert!(!net.set_sspc("NOPE", true));
}

#[test]
#[allow(clippy::disallowed_methods)]
fn full_network_solve_time() {
    let mut net = ElectricalNet::new();
    assert!(net.set_external_power("L FWD", true));
    close(
        &mut net,
        &[
            "MBR",
            "L EPC",
            "L BSB",
            "L ATUC",
            "L3 BTB",
            "L TRU RLY",
            "L1 ATRUC",
        ],
    );
    asg_left(&mut net);
    let s = run(&mut net, 50);
    assert!(s.converged);
    let start = std::time::Instant::now();
    let n = 200;
    for _ in 0..n {
        net.step(10_000).expect("solve");
    }
    let per = start.elapsed().as_secs_f64() * 1000.0 / n as f64;
    println!("full net step: {per:.3} ms");
}

#[test]
fn every_located_load_from_the_table_is_in_the_network() {
    let net = ElectricalNet::from_data(&ElectricalNet::default_data_dir()).unwrap();
    let mut per: std::collections::BTreeMap<&str, (usize, f64)> = std::collections::BTreeMap::new();
    for (name, bus) in net.circuits() {
        let e = per.entry(bus.as_str()).or_insert((0, 0.0));
        e.0 += 1;
        e.1 += net.load_demand_w(name).unwrap_or(0.0);
    }
    eprintln!(
        "loads {} circuits {} {:?}",
        net.load_names().len(),
        net.circuits().len(),
        per
    );
    assert!(net.load_names().len() > 1000, "{}", net.load_names().len());
}
