pub const STEFAN_BOLTZMANN: f64 = 5.670_374_419e-8;

const AIR_GAS_CONSTANT: f64 = 287.052_87;
const PRANDTL: f64 = 0.71;
const MIN_TEMPERATURE: f64 = 1.0;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct NodeId(usize);

impl NodeId {
    pub fn index(self) -> usize {
        self.0
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct LinkId(usize);

impl LinkId {
    pub fn index(self) -> usize {
        self.0
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct StepReport {
    pub dt_s: f64,
    pub source_j: f64,
    pub boundary_j: f64,
    pub energy_in_j: f64,
    pub energy_out_j: f64,
    pub stored_delta_j: f64,
    pub imbalance_j: f64,
}

#[derive(Clone, Copy, Debug)]
struct Node {
    capacity: f64,
    temperature: f64,
    fixed: bool,
    source: f64,
}

#[derive(Clone, Copy, Debug)]
enum LinkKind {
    Conductance {
        g: f64,
    },
    Convection {
        area: f64,
        length: f64,
        velocity: f64,
        pressure: f64,
    },
    Radiation {
        emissivity: f64,
        area: f64,
    },
}

#[derive(Clone, Copy, Debug)]
struct Link {
    a: usize,
    b: usize,
    kind: LinkKind,
}

#[derive(Clone, Debug, Default)]
pub struct Network {
    nodes: Vec<Node>,
    links: Vec<Link>,
    total_source_j: f64,
    total_in_j: f64,
    total_out_j: f64,
}

fn air_conductivity(t: f64) -> f64 {
    0.0241 * (t / 273.15).powf(0.9)
}

fn air_viscosity(t: f64) -> f64 {
    1.716e-5 * (t / 273.15).powf(1.5) * (273.15 + 110.4) / (t + 110.4)
}

pub fn convection_coefficient(
    velocity_m_s: f64,
    pressure_pa: f64,
    length_m: f64,
    film_temperature_k: f64,
    delta_t_k: f64,
) -> f64 {
    let t = film_temperature_k.max(MIN_TEMPERATURE);
    let p = pressure_pa.max(0.0);
    let l = length_m.max(1e-6);
    let v = velocity_m_s.abs();
    let k = air_conductivity(t);
    let rho = p / (AIR_GAS_CONSTANT * t);
    let re = rho * v * l / air_viscosity(t);
    let pr13 = PRANDTL.powf(1.0 / 3.0);
    let nu = if re < 5.0e5 {
        0.664 * re.sqrt() * pr13
    } else {
        (0.037 * re.powf(0.8) - 871.0).max(0.0) * pr13
    };
    let forced = (nu * k / l).max(0.0);
    let natural = 1.42 * (delta_t_k.abs() / l).powf(0.25) * (p / 101_325.0).sqrt();
    (forced.powi(3) + natural.powi(3)).cbrt()
}

impl Network {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add_node(&mut self, capacity_j_k: f64, temperature_k: f64) -> NodeId {
        self.nodes.push(Node {
            capacity: capacity_j_k.max(0.0),
            temperature: temperature_k,
            fixed: false,
            source: 0.0,
        });
        NodeId(self.nodes.len() - 1)
    }

    pub fn add_boundary(&mut self, temperature_k: f64) -> NodeId {
        self.nodes.push(Node {
            capacity: 0.0,
            temperature: temperature_k,
            fixed: true,
            source: 0.0,
        });
        NodeId(self.nodes.len() - 1)
    }

    pub fn node_count(&self) -> usize {
        self.nodes.len()
    }

    pub fn link_count(&self) -> usize {
        self.links.len()
    }

    pub fn is_boundary(&self, node: NodeId) -> bool {
        self.nodes[node.0].fixed
    }

    pub fn temperature(&self, node: NodeId) -> f64 {
        self.nodes[node.0].temperature
    }

    pub fn set_temperature(&mut self, node: NodeId, temperature_k: f64) {
        self.nodes[node.0].temperature = temperature_k;
    }

    pub fn capacity(&self, node: NodeId) -> f64 {
        self.nodes[node.0].capacity
    }

    pub fn set_capacity(&mut self, node: NodeId, capacity_j_k: f64) {
        self.nodes[node.0].capacity = capacity_j_k.max(0.0);
    }

    pub fn source(&self, node: NodeId) -> f64 {
        self.nodes[node.0].source
    }

    pub fn set_source(&mut self, node: NodeId, watts: f64) {
        self.nodes[node.0].source = watts;
    }

    pub fn add_source(&mut self, node: NodeId, watts: f64) {
        self.nodes[node.0].source += watts;
    }

    fn push_link(&mut self, a: NodeId, b: NodeId, kind: LinkKind) -> LinkId {
        self.links.push(Link { a: a.0, b: b.0, kind });
        LinkId(self.links.len() - 1)
    }

    pub fn add_conductance(&mut self, a: NodeId, b: NodeId, g_w_k: f64) -> LinkId {
        self.push_link(a, b, LinkKind::Conductance { g: g_w_k.max(0.0) })
    }

    pub fn set_conductance(&mut self, link: LinkId, g_w_k: f64) {
        if let LinkKind::Conductance { g } = &mut self.links[link.0].kind {
            *g = g_w_k.max(0.0);
        }
    }

    pub fn add_convection(&mut self, a: NodeId, b: NodeId, area_m2: f64, length_m: f64) -> LinkId {
        self.push_link(
            a,
            b,
            LinkKind::Convection {
                area: area_m2.max(0.0),
                length: length_m.max(1e-6),
                velocity: 0.0,
                pressure: 101_325.0,
            },
        )
    }

    pub fn set_airflow(&mut self, link: LinkId, velocity_m_s: f64, pressure_pa: f64) {
        if let LinkKind::Convection {
            velocity, pressure, ..
        } = &mut self.links[link.0].kind
        {
            *velocity = velocity_m_s;
            *pressure = pressure_pa;
        }
    }

    pub fn add_radiation(&mut self, a: NodeId, b: NodeId, emissivity: f64, area_m2: f64) -> LinkId {
        self.push_link(
            a,
            b,
            LinkKind::Radiation {
                emissivity: emissivity.clamp(0.0, 1.0),
                area: area_m2.max(0.0),
            },
        )
    }

    pub fn link_conductance(&self, link: LinkId) -> f64 {
        self.effective_conductance(&self.links[link.0])
    }

    fn effective_conductance(&self, link: &Link) -> f64 {
        let ta = self.nodes[link.a].temperature.max(MIN_TEMPERATURE);
        let tb = self.nodes[link.b].temperature.max(MIN_TEMPERATURE);
        match link.kind {
            LinkKind::Conductance { g } => g,
            LinkKind::Convection {
                area,
                length,
                velocity,
                pressure,
            } => {
                let film = 0.5 * (ta + tb);
                area * convection_coefficient(velocity, pressure, length, film, ta - tb)
            }
            LinkKind::Radiation { emissivity, area } => {
                emissivity * STEFAN_BOLTZMANN * area * (ta * ta + tb * tb) * (ta + tb)
            }
        }
    }

    pub fn stored_energy_j(&self) -> f64 {
        self.nodes
            .iter()
            .filter(|n| !n.fixed)
            .map(|n| n.capacity * n.temperature)
            .sum()
    }

    pub fn total_source_j(&self) -> f64 {
        self.total_source_j
    }

    pub fn total_in_j(&self) -> f64 {
        self.total_in_j
    }

    pub fn total_out_j(&self) -> f64 {
        self.total_out_j
    }

    pub fn step(&mut self, dt_s: f64) -> StepReport {
        if !(dt_s.is_finite() && dt_s > 0.0) {
            return StepReport::default();
        }
        let n = self.nodes.len();
        let mut index = vec![usize::MAX; n];
        let mut m = 0;
        for (i, node) in self.nodes.iter().enumerate() {
            if !node.fixed {
                index[i] = m;
                m += 1;
            }
        }
        let gs: Vec<f64> = self.links.iter().map(|l| self.effective_conductance(l)).collect();
        let mut a = vec![0.0; m * m];
        let mut rhs = vec![0.0; m];
        for (i, node) in self.nodes.iter().enumerate() {
            if !node.fixed {
                let r = index[i];
                let c = node.capacity / dt_s;
                a[r * m + r] += c;
                rhs[r] += c * node.temperature + node.source;
            }
        }
        for (link, &g) in self.links.iter().zip(gs.iter()) {
            if link.a == link.b || g <= 0.0 || !g.is_finite() {
                continue;
            }
            let (ia, ib) = (index[link.a], index[link.b]);
            match (ia != usize::MAX, ib != usize::MAX) {
                (true, true) => {
                    a[ia * m + ia] += g;
                    a[ib * m + ib] += g;
                    a[ia * m + ib] -= g;
                    a[ib * m + ia] -= g;
                }
                (true, false) => {
                    a[ia * m + ia] += g;
                    rhs[ia] += g * self.nodes[link.b].temperature;
                }
                (false, true) => {
                    a[ib * m + ib] += g;
                    rhs[ib] += g * self.nodes[link.a].temperature;
                }
                (false, false) => {}
            }
        }
        for (i, node) in self.nodes.iter().enumerate() {
            if !node.fixed {
                let r = index[i];
                if a[r * m + r] <= 0.0 {
                    a[r * m + r] = 1.0;
                    rhs[r] = node.temperature;
                }
            }
        }
        let old_stored = self.stored_energy_j();
        cholesky_solve(&mut a, &mut rhs, m);
        for (i, node) in self.nodes.iter_mut().enumerate() {
            if !node.fixed {
                node.temperature = rhs[index[i]];
            }
        }
        let mut report = StepReport {
            dt_s,
            ..StepReport::default()
        };
        for node in self.nodes.iter().filter(|n| !n.fixed) {
            report.source_j += node.source * dt_s;
            if node.source >= 0.0 {
                report.energy_in_j += node.source * dt_s;
            } else {
                report.energy_out_j -= node.source * dt_s;
            }
        }
        for (link, &g) in self.links.iter().zip(gs.iter()) {
            if link.a == link.b || g <= 0.0 || !g.is_finite() {
                continue;
            }
            let fa = self.nodes[link.a].fixed;
            let fb = self.nodes[link.b].fixed;
            if fa == fb {
                continue;
            }
            let (fixed, free) = if fa { (link.a, link.b) } else { (link.b, link.a) };
            let q = g * (self.nodes[fixed].temperature - self.nodes[free].temperature) * dt_s;
            report.boundary_j += q;
            if q >= 0.0 {
                report.energy_in_j += q;
            } else {
                report.energy_out_j -= q;
            }
        }
        report.stored_delta_j = self.stored_energy_j() - old_stored;
        report.imbalance_j = report.stored_delta_j - report.source_j - report.boundary_j;
        self.total_source_j += report.source_j;
        self.total_in_j += report.energy_in_j;
        self.total_out_j += report.energy_out_j;
        report
    }

    pub fn advance(&mut self, duration_s: f64, max_dt_s: f64) -> StepReport {
        let mut total = StepReport::default();
        if !(duration_s.is_finite() && duration_s > 0.0 && max_dt_s > 0.0) {
            return total;
        }
        let steps = (duration_s / max_dt_s).ceil().max(1.0);
        let dt = duration_s / steps;
        let count = steps as u64;
        for _ in 0..count {
            let r = self.step(dt);
            total.dt_s += r.dt_s;
            total.source_j += r.source_j;
            total.boundary_j += r.boundary_j;
            total.energy_in_j += r.energy_in_j;
            total.energy_out_j += r.energy_out_j;
            total.stored_delta_j += r.stored_delta_j;
            total.imbalance_j += r.imbalance_j;
        }
        total
    }
}

fn cholesky_solve(a: &mut [f64], b: &mut [f64], m: usize) {
    for j in 0..m {
        let mut s = a[j * m + j];
        for k in 0..j {
            s -= a[j * m + k] * a[j * m + k];
        }
        let d = if s > 0.0 { s.sqrt() } else { 1e-150 };
        a[j * m + j] = d;
        for i in (j + 1)..m {
            let mut v = a[i * m + j];
            for k in 0..j {
                v -= a[i * m + k] * a[j * m + k];
            }
            a[i * m + j] = v / d;
        }
    }
    for i in 0..m {
        let mut v = b[i];
        for k in 0..i {
            v -= a[i * m + k] * b[k];
        }
        b[i] = v / a[i * m + i];
    }
    for i in (0..m).rev() {
        let mut v = b[i];
        for k in (i + 1)..m {
            v -= a[k * m + i] * b[k];
        }
        b[i] = v / a[i * m + i];
    }
}
