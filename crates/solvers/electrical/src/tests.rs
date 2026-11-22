use super::*;

fn res(v: f64) -> LoadKind {
    LoadKind::Impedance {
        z: Complex::new(v, 0.0),
    }
}

fn ac_source(node: NodeId, emf: f64, freq: f64) -> Element {
    Element::AcSource {
        node,
        emf,
        angle: 0.0,
        r: 0.01,
        x: 0.0,
        frequency_hz: freq,
        r_neutral_ground: 1e-4,
    }
}

fn close(a: f64, b: f64, tol: f64) -> bool {
    (a - b).abs() <= tol * (1.0 + b.abs())
}

#[test]
fn dc_divider() {
    let mut net = Network::new();
    let a = net.add_node(NodeKind::Dc);
    let b = net.add_node(NodeKind::Dc);
    net.add_element(Element::DcSource {
        node: a,
        e: 100.0,
        r: 1e-6,
    })
    .unwrap();
    net.add_element(Element::Resistor { a, b, r: 1000.0 }).unwrap();
    net.add_element(Element::DcLoad {
        node: b,
        kind: res(1000.0),
    })
    .unwrap();
    let sol = Solver::default().solve(&net).unwrap();
    assert!(close(sol.dc_voltage(b), 50.0, 1e-5));
    assert!(close(sol.dc_voltage(a), 100.0, 1e-5));
    assert!(sol.report.converged);
}

#[test]
fn battery_with_internal_resistance() {
    let mut net = Network::new();
    let n = net.add_node(NodeKind::Dc);
    let src = net
        .add_element(Element::DcSource {
            node: n,
            e: 28.0,
            r: 0.05,
        })
        .unwrap();
    let load = net
        .add_element(Element::DcLoad {
            node: n,
            kind: res(2.8),
        })
        .unwrap();
    let sol = Solver::default().solve(&net).unwrap();
    let i = 28.0 / 2.85;
    assert!(close(sol.dc_voltage(n), 28.0 - 0.05 * i, 1e-6));
    assert!(close(sol.element_currents(src)[0].re, i, 1e-6));
    assert!(close(sol.element_currents(load)[0].re, i, 1e-6));
    assert!(close(sol.element_power(load).re, i * i * 2.8, 1e-6));
    assert!(close(sol.element_power(src).re, 28.0 * i - i * i * 0.05, 1e-6));
}

#[test]
fn balanced_three_phase_load() {
    let mut net = Network::new();
    let n = net.add_node(NodeKind::Ac3);
    let src = net.add_element(ac_source(n, 115.0, 400.0)).unwrap();
    let load = net.add_element(Element::balanced_ac_load(res(10.0), n)).unwrap();
    let sol = Solver::default().solve(&net).unwrap();
    let i = 115.0 / 10.01;
    let c = sol.element_currents(load);
    for p in 0..3 {
        assert!(close(c[p].abs(), i, 1e-6));
    }
    assert!(c[3].abs() < 1e-6);
    assert!(close(sol.element_power(load).re, 3.0 * i * i * 10.0, 1e-6));
    assert!(sol.element_power(load).im.abs() < 1e-6);
    assert!(close(sol.element_power(src).re, 3.0 * i * i * 10.0, 1e-3));
    assert!(close(
        sol.line_voltage(n, 0, 1).abs(),
        3f64.sqrt() * i * 10.0,
        1e-6
    ));
    assert!(close(sol.frequency_of(n), 400.0, 1e-12));
}

#[test]
fn unbalanced_load_gives_neutral_current() {
    let mut net = Network::new();
    let n = net.add_node(NodeKind::Ac3);
    net.add_element(ac_source(n, 115.0, 400.0)).unwrap();
    let load = net
        .add_element(Element::AcLoad {
            node: n,
            phases: [res(10.0), res(20.0), res(40.0)],
        })
        .unwrap();
    let sol = Solver::default().solve(&net).unwrap();
    let c = sol.element_currents(load);
    assert!(c[0].abs() > c[1].abs() && c[1].abs() > c[2].abs());
    let expect = (c[0] + c[1] + c[2]).abs();
    assert!(expect > 2.0);
    assert!(close(c[3].abs(), expect, 1e-9));
}

#[test]
fn open_switch_isolates_island() {
    let mut net = Network::new();
    let a = net.add_node(NodeKind::Dc);
    let b = net.add_node(NodeKind::Dc);
    net.add_element(Element::DcSource {
        node: a,
        e: 28.0,
        r: 0.01,
    })
    .unwrap();
    let sw = net
        .add_element(Element::Switch {
            a,
            b,
            closed: true,
            r_closed: 0.001,
        })
        .unwrap();
    net.add_element(Element::DcLoad {
        node: b,
        kind: res(5.0),
    })
    .unwrap();
    let mut solver = Solver::default();
    let s1 = solver.solve(&net).unwrap();
    assert!(s1.dc_voltage(b) > 27.0);
    assert_eq!(s1.island_count(), 1);
    net.set_switch(sw, false).unwrap();
    let s2 = solver.solve(&net).unwrap();
    assert_eq!(s2.island_count(), 2);
    assert_eq!(s2.dc_voltage(b), 0.0);
    assert!(!s2.is_live(b));
    assert!(s2.is_live(a));
    assert!(s2.report.topology_rebuilt);
    net.set_switch(sw, true).unwrap();
    let s3 = solver.solve(&net).unwrap();
    assert!(s3.dc_voltage(b) > 27.0);
}

#[test]
fn separate_islands_different_frequencies() {
    let mut net = Network::new();
    let a = net.add_node(NodeKind::Ac3);
    let b = net.add_node(NodeKind::Ac3);
    net.add_element(ac_source(a, 115.0, 400.0)).unwrap();
    net.add_element(ac_source(b, 115.0, 360.0)).unwrap();
    net.add_element(Element::balanced_ac_load(res(10.0), a)).unwrap();
    net.add_element(Element::balanced_ac_load(res(10.0), b)).unwrap();
    let sol = Solver::default().solve(&net).unwrap();
    assert_ne!(sol.island_of(a), sol.island_of(b));
    assert!(close(sol.frequency_of(a), 400.0, 1e-12));
    assert!(close(sol.frequency_of(b), 360.0, 1e-12));
    assert!(!sol.island(sol.island_of(a)).frequency_conflict);
}

fn atru_net(load_r: f64) -> (Network, NodeId, NodeId, NodeId, ElementId, ElementId) {
    let mut net = Network::new();
    let ac = net.add_node(NodeKind::Ac3);
    let pos = net.add_node(NodeKind::Dc);
    let neg = net.add_node(NodeKind::Dc);
    let src = net.add_element(ac_source(ac, 235.0, 400.0)).unwrap();
    let rect = net
        .add_element(Element::Rectifier {
            ac,
            pos,
            neg: Some(neg),
            ratio: 540.0 / (235.0 * 3f64.sqrt()),
            r_out: 0.05,
            efficiency: 0.96,
            ac_v_nominal: 235.0,
        })
        .unwrap();
    net.add_element(Element::DcLoad {
        node: pos,
        kind: res(load_r),
    })
    .unwrap();
    net.add_element(Element::DcLoad {
        node: neg,
        kind: res(load_r),
    })
    .unwrap();
    (net, ac, pos, neg, src, rect)
}

#[test]
fn atru_chain_power_balance() {
    let (net, ac, pos, neg, src, rect) = atru_net(3.645);
    let sol = Solver::default().solve(&net).unwrap();
    assert!(sol.report.converged);
    let vp = sol.dc_voltage(pos);
    let vn = sol.dc_voltage(neg);
    assert!(vp > 255.0 && vp < 270.5);
    assert!(close(vp, -vn, 1e-9));
    let p_out = vp * vp / 3.645 + vn * vn / 3.645;
    assert!(close(sol.converter_output_power(rect), p_out, 1e-6));
    let p_in = sol.element_power(rect).re;
    assert!(close(p_in, p_out / 0.96, 1e-6));
    let p_src = sol.element_power(src).re;
    assert!(close(p_src, p_in, 1e-3));
    assert!(sol.line_voltage(ac, 0, 3).abs() > 220.0);
    assert!(sol.island(sol.island_of(pos)).live);
}

#[test]
fn tru_chain_28v() {
    let mut net = Network::new();
    let ac = net.add_node(NodeKind::Ac3);
    let dc = net.add_node(NodeKind::Dc);
    net.add_element(ac_source(ac, 235.0, 400.0)).unwrap();
    let rect = net
        .add_element(Element::Rectifier {
            ac,
            pos: dc,
            neg: None,
            ratio: 28.0 / (235.0 * 3f64.sqrt()),
            r_out: 0.01,
            efficiency: 0.9,
            ac_v_nominal: 235.0,
        })
        .unwrap();
    net.add_element(Element::DcLoad {
        node: dc,
        kind: res(1.0),
    })
    .unwrap();
    let sol = Solver::default().solve(&net).unwrap();
    let v = sol.dc_voltage(dc);
    assert!(close(v, 28.0 / 1.01, 5e-3));
    assert!(close(sol.converter_output_power(rect), v * v, 1e-6));
    assert!(close(sol.element_power(rect).re, v * v / 0.9, 1e-6));
}

#[test]
fn autotransformer_ratio_and_efficiency() {
    let mut net = Network::new();
    let hi = net.add_node(NodeKind::Ac3);
    let lo = net.add_node(NodeKind::Ac3);
    net.add_element(ac_source(hi, 235.0, 400.0)).unwrap();
    let atu = net
        .add_element(Element::Autotransformer {
            input: hi,
            output: lo,
            ratio: 115.0 / 235.0,
            r_leak: 0.02,
            x_leak: 0.02,
            efficiency: 0.98,
            in_v_nominal: 235.0,
            r_neutral_ground: 1e-4,
        })
        .unwrap();
    let load = net.add_element(Element::balanced_ac_load(res(5.0), lo)).unwrap();
    let sol = Solver::default().solve(&net).unwrap();
    assert!(sol.report.converged);
    let vl = sol.phase_voltages(lo)[0].abs();
    assert!(vl > 112.0 && vl < 115.0);
    assert!(close(sol.frequency_of(lo), 400.0, 1e-12));
    let p_load = sol.element_power(load).re;
    assert!(close(sol.converter_output_power(atu), p_load, 1e-6));
    assert!(close(sol.element_power(atu).re, p_load / 0.98, 1e-5));
}

#[test]
fn constant_power_load_converges() {
    let mut net = Network::new();
    let n = net.add_node(NodeKind::Dc);
    net.add_element(Element::DcSource {
        node: n,
        e: 28.0,
        r: 0.05,
    })
    .unwrap();
    let load = net
        .add_element(Element::DcLoad {
            node: n,
            kind: LoadKind::Power {
                p: 1000.0,
                q: 0.0,
                v_nominal: 28.0,
            },
        })
        .unwrap();
    let sol = Solver::default().solve(&net).unwrap();
    assert!(sol.report.converged);
    assert!(sol.report.max_inner_iterations > 2);
    let v = (28.0 + (784.0f64 - 200.0).sqrt()) / 2.0;
    assert!(close(sol.dc_voltage(n), v, 1e-7));
    assert!(close(sol.element_power(load).re, 1000.0, 1e-7));
}

#[test]
fn constant_power_low_voltage_fallback() {
    let mut net = Network::new();
    let n = net.add_node(NodeKind::Dc);
    net.add_element(Element::DcSource {
        node: n,
        e: 28.0,
        r: 0.05,
    })
    .unwrap();
    let load = net
        .add_element(Element::DcLoad {
            node: n,
            kind: LoadKind::Power {
                p: 100_000.0,
                q: 0.0,
                v_nominal: 28.0,
            },
        })
        .unwrap();
    let sol = Solver::default().solve(&net).unwrap();
    assert!(sol.report.converged);
    let v = sol.dc_voltage(n);
    assert!(v < 14.0 && v > 0.0);
    let y0 = 100_000.0 / (14.0 * 14.0);
    assert!(close(sol.element_power(load).re, y0 * v * v, 1e-6));
}

fn nose_net(p: f64) -> (Network, NodeId, ElementId) {
    let mut net = Network::new();
    let n = net.add_node(NodeKind::Dc);
    net.add_element(Element::DcSource {
        node: n,
        e: 31.0,
        r: 1.0,
    })
    .unwrap();
    let load = net
        .add_element(Element::DcLoad {
            node: n,
            kind: LoadKind::Power {
                p,
                q: 0.0,
                v_nominal: 28.0,
            },
        })
        .unwrap();
    (net, n, load)
}

#[test]
fn constant_power_converges_at_the_power_transfer_nose() {
    let (net, n, load) = nose_net(239.5);
    let sol = Solver::default().solve(&net).unwrap();
    assert!(sol.report.converged);
    let v = (31.0 + (961.0f64 - 4.0 * 239.5).sqrt()) / 2.0;
    assert!(close(sol.dc_voltage(n), v, 1e-7));
    assert!(close(sol.element_power(load).re, 239.5, 1e-6));
}

#[test]
fn constant_power_past_the_nose_drops_to_fallback() {
    let (net, n, load) = nose_net(241.0);
    let sol = Solver::default().solve(&net).unwrap();
    assert!(sol.report.converged);
    let v = sol.dc_voltage(n);
    assert!(v < 14.0 && v > 0.0);
    let y0 = 241.0 / (14.0 * 14.0);
    assert!(close(sol.element_power(load).re, y0 * v * v, 1e-6));
}

#[test]
fn value_change_refactors_without_rebuild() {
    let mut net = Network::new();
    let n = net.add_node(NodeKind::Dc);
    let src = net
        .add_element(Element::DcSource {
            node: n,
            e: 28.0,
            r: 0.05,
        })
        .unwrap();
    net.add_element(Element::DcLoad {
        node: n,
        kind: res(2.8),
    })
    .unwrap();
    let mut solver = Solver::default();
    let a = solver.solve(&net).unwrap();
    assert_eq!(a.report.islands_factored, 1);
    net.set_element(
        src,
        Element::DcSource {
            node: n,
            e: 24.0,
            r: 0.08,
        },
    )
    .unwrap();
    let b = solver.solve(&net).unwrap();
    assert!(!b.report.topology_rebuilt);
    assert_eq!(b.report.islands_refactored, 1);
    assert!(close(b.dc_voltage(n), 24.0 * 2.8 / 2.88, 1e-6));
}

struct Rng(u64);

impl Rng {
    fn next(&mut self) -> f64 {
        self.0 = self
            .0
            .wrapping_mul(6364136223846793005)
            .wrapping_add(1442695040888963407);
        ((self.0 >> 11) as f64) / ((1u64 << 53) as f64)
    }
    fn index(&mut self, n: usize) -> usize {
        ((self.next() * n as f64) as usize).min(n - 1)
    }
}

fn random_net(n: usize) -> Network {
    let mut rng = Rng(42);
    let mut net = Network::new();
    let nodes: Vec<NodeId> = (0..n).map(|_| net.add_node(NodeKind::Dc)).collect();
    net.add_element(Element::DcSource {
        node: nodes[0],
        e: 270.0,
        r: 0.01,
    })
    .unwrap();
    for i in 1..n {
        let j = rng.index(i);
        net.add_element(Element::Resistor {
            a: nodes[i],
            b: nodes[j],
            r: 0.05 + rng.next(),
        })
        .unwrap();
    }
    for _ in 0..n / 2 {
        let a = rng.index(n);
        let b = rng.index(n);
        if a != b {
            net.add_element(Element::Resistor {
                a: nodes[a],
                b: nodes[b],
                r: 0.1 + rng.next() * 5.0,
            })
            .unwrap();
        }
    }
    for _ in 0..n / 4 {
        net.add_element(Element::DcLoad {
            node: nodes[rng.index(n)],
            kind: res(200.0 + rng.next() * 800.0),
        })
        .unwrap();
    }
    for _ in 0..n / 10 {
        net.add_element(Element::DcLoad {
            node: nodes[rng.index(n)],
            kind: LoadKind::Power {
                p: 5.0 + rng.next() * 20.0,
                q: 0.0,
                v_nominal: 270.0,
            },
        })
        .unwrap();
    }
    net
}

#[test]
fn large_random_network_is_deterministic_and_conserves_power() {
    let net = random_net(2000);
    let mut solver = Solver::default();
    let a = solver.solve(&net).unwrap();
    let b = solver.solve(&net).unwrap();
    let c = Solver::default().solve(&net).unwrap();
    assert!(a.report.converged);
    assert_eq!(a.island_count(), 1);
    for k in 0..net.node_count() {
        let id = NodeId(k);
        let va = a.dc_voltage(id);
        assert!(va > 0.0 && va <= 270.0);
        assert_eq!(va.to_bits(), b.dc_voltage(id).to_bits());
        assert_eq!(va.to_bits(), c.dc_voltage(id).to_bits());
    }
    let mut delivered = 0.0;
    let mut absorbed = 0.0;
    for (k, e) in net.elements().iter().enumerate() {
        let p = a.element_power(ElementId(k)).re;
        match e.power_sense() {
            PowerSense::Delivered => delivered += p,
            PowerSense::Absorbed => absorbed += p,
        }
    }
    assert!(delivered > 0.0);
    assert!(close(delivered, absorbed, 1e-3));
}

fn ac_nose_net(p: f64) -> (Network, NodeId, ElementId) {
    let mut net = Network::new();
    let n = net.add_node(NodeKind::Ac3);
    net.add_element(Element::AcSource {
        node: n,
        emf: 115.0,
        angle: 0.0,
        r: 1.0,
        x: 0.5,
        frequency_hz: 400.0,
        r_neutral_ground: 1e-4,
    })
    .unwrap();
    let load = net
        .add_element(Element::balanced_ac_load(
            LoadKind::Power {
                p,
                q: 0.0,
                v_nominal: 115.0,
            },
            n,
        ))
        .unwrap();
    (net, n, load)
}

#[test]
fn ac_constant_power_converges_near_the_nose() {
    let (net, n, load) = ac_nose_net(3050.0);
    let sol = Solver::default().solve(&net).unwrap();
    assert!(sol.report.converged);
    let v = sol.phase_voltages(n)[0].abs();
    assert!(v > 57.5 && v < 115.0);
    assert!(close(sol.element_power(load).re, 3.0 * 3050.0, 1e-6));
}

#[test]
fn ac_constant_power_past_the_nose_drops_to_fallback() {
    let (net, n, _) = ac_nose_net(3300.0);
    let sol = Solver::default().solve(&net).unwrap();
    assert!(sol.report.converged);
    let v = sol.phase_voltages(n)[0].abs();
    assert!(v > 0.0 && v < 57.5);
}
