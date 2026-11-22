use crate::network::{LinkId, Network, NodeId};

const REFERENCE_TEMPERATURE_K: f64 = 293.15;
const AIR_CP: f64 = 1005.0;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Conductor {
    Copper,
    Aluminium,
}

impl Conductor {
    pub fn resistivity_20c(self) -> f64 {
        match self {
            Conductor::Copper => 1.724e-8,
            Conductor::Aluminium => 2.82e-8,
        }
    }

    pub fn alpha_per_k(self) -> f64 {
        match self {
            Conductor::Copper => 0.00393,
            Conductor::Aluminium => 0.00403,
        }
    }

    pub fn density(self) -> f64 {
        match self {
            Conductor::Copper => 8960.0,
            Conductor::Aluminium => 2700.0,
        }
    }

    pub fn specific_heat(self) -> f64 {
        match self {
            Conductor::Copper => 385.0,
            Conductor::Aluminium => 897.0,
        }
    }

    pub fn resistivity_at(self, temperature_k: f64) -> f64 {
        let f = 1.0 + self.alpha_per_k() * (temperature_k - REFERENCE_TEMPERATURE_K);
        self.resistivity_20c() * f.max(0.05)
    }
}

pub fn conductor_resistance(conductor: Conductor, length_m: f64, area_m2: f64, temperature_k: f64) -> f64 {
    conductor.resistivity_at(temperature_k) * length_m / area_m2.max(1e-12)
}

#[derive(Clone, Copy, Debug)]
pub struct WireSpec {
    pub conductor: Conductor,
    pub length_m: f64,
    pub area_m2: f64,
    pub outer_diameter_m: f64,
    pub surface_h_w_m2k: f64,
    pub extra_capacity_j_k: f64,
    pub initial_temperature_k: f64,
}

impl WireSpec {
    pub fn capacity_j_k(&self) -> f64 {
        self.conductor.density() * self.conductor.specific_heat() * self.area_m2 * self.length_m
            + self.extra_capacity_j_k
    }

    pub fn surface_conductance_w_k(&self) -> f64 {
        self.surface_h_w_m2k * std::f64::consts::PI * self.outer_diameter_m * self.length_m
    }
}

pub fn wire_ampacity(spec: &WireSpec, ambient_k: f64, max_temperature_k: f64) -> f64 {
    let r = conductor_resistance(spec.conductor, spec.length_m, spec.area_m2, max_temperature_k);
    let q = spec.surface_conductance_w_k() * (max_temperature_k - ambient_k).max(0.0);
    (q / r).sqrt()
}

#[derive(Clone, Copy, Debug)]
pub struct WireSegment {
    node: NodeId,
    conductor: Conductor,
    length_m: f64,
    area_m2: f64,
}

impl WireSegment {
    pub fn add(net: &mut Network, spec: &WireSpec, sink: NodeId) -> Self {
        let node = net.add_node(spec.capacity_j_k(), spec.initial_temperature_k);
        net.add_conductance(node, sink, spec.surface_conductance_w_k());
        Self {
            node,
            conductor: spec.conductor,
            length_m: spec.length_m,
            area_m2: spec.area_m2,
        }
    }

    pub fn node(&self) -> NodeId {
        self.node
    }

    pub fn temperature(&self, net: &Network) -> f64 {
        net.temperature(self.node)
    }

    pub fn resistance(&self, net: &Network) -> f64 {
        conductor_resistance(
            self.conductor,
            self.length_m,
            self.area_m2,
            net.temperature(self.node),
        )
    }

    pub fn heat_w(&self, net: &Network, current_a: f64) -> f64 {
        current_a * current_a * self.resistance(net)
    }

    pub fn apply(&self, net: &mut Network, current_a: f64) -> f64 {
        let q = self.heat_w(net, current_a);
        net.set_source(self.node, q);
        q
    }
}

#[derive(Clone, Copy, Debug)]
pub struct BundleSpec {
    pub diameter_m: f64,
    pub length_m: f64,
    pub surface_h_w_m2k: f64,
    pub capacity_j_k: f64,
    pub initial_temperature_k: f64,
}

#[derive(Clone, Debug)]
pub struct WireBundle {
    node: NodeId,
    wires: Vec<WireSegment>,
}

impl WireBundle {
    pub fn add(
        net: &mut Network,
        bundle: &BundleSpec,
        wire: &WireSpec,
        count: usize,
        ambient: NodeId,
    ) -> Self {
        let node = net.add_node(bundle.capacity_j_k, bundle.initial_temperature_k);
        let g = bundle.surface_h_w_m2k * std::f64::consts::PI * bundle.diameter_m * bundle.length_m;
        net.add_conductance(node, ambient, g);
        let wires = (0..count).map(|_| WireSegment::add(net, wire, node)).collect();
        Self { node, wires }
    }

    pub fn node(&self) -> NodeId {
        self.node
    }

    pub fn wires(&self) -> &[WireSegment] {
        &self.wires
    }

    pub fn temperature(&self, net: &Network) -> f64 {
        net.temperature(self.node)
    }

    pub fn apply_uniform(&self, net: &mut Network, current_a: f64) -> f64 {
        self.wires.iter().map(|w| w.apply(net, current_a)).sum()
    }

    pub fn apply(&self, net: &mut Network, currents_a: &[f64]) -> f64 {
        self.wires
            .iter()
            .zip(currents_a.iter())
            .map(|(w, &i)| w.apply(net, i))
            .sum()
    }
}

#[derive(Clone, Copy, Debug)]
pub struct ContactSpec {
    pub resistance_ohm_20c: f64,
    pub alpha_per_k: f64,
    pub capacity_j_k: f64,
    pub conductance_to_case_w_k: f64,
    pub initial_temperature_k: f64,
}

#[derive(Clone, Copy, Debug)]
pub struct ContactorContact {
    node: NodeId,
    resistance_ohm_20c: f64,
    alpha_per_k: f64,
}

impl ContactorContact {
    pub fn add(net: &mut Network, spec: &ContactSpec, case: NodeId) -> Self {
        let node = net.add_node(spec.capacity_j_k, spec.initial_temperature_k);
        net.add_conductance(node, case, spec.conductance_to_case_w_k);
        Self {
            node,
            resistance_ohm_20c: spec.resistance_ohm_20c,
            alpha_per_k: spec.alpha_per_k,
        }
    }

    pub fn node(&self) -> NodeId {
        self.node
    }

    pub fn temperature(&self, net: &Network) -> f64 {
        net.temperature(self.node)
    }

    pub fn resistance(&self, net: &Network) -> f64 {
        let f = 1.0 + self.alpha_per_k * (net.temperature(self.node) - REFERENCE_TEMPERATURE_K);
        self.resistance_ohm_20c * f.max(0.05)
    }

    pub fn apply(&self, net: &mut Network, current_a: f64, coil_w: f64) -> f64 {
        let q = current_a * current_a * self.resistance(net) + coil_w;
        net.set_source(self.node, q);
        q
    }
}

#[derive(Clone, Copy, Debug)]
pub struct BatterySpec {
    pub mass_kg: f64,
    pub specific_heat_j_kg_k: f64,
    pub resistance_ohm_20c: f64,
    pub resistance_cold_coeff_per_k: f64,
    pub entropic_v_per_k: f64,
    pub conductance_to_sink_w_k: f64,
    pub initial_temperature_k: f64,
}

#[derive(Clone, Copy, Debug)]
pub struct BatteryCell {
    node: NodeId,
    spec: BatterySpec,
}

impl BatteryCell {
    pub fn add(net: &mut Network, spec: &BatterySpec, sink: NodeId) -> Self {
        let node = net.add_node(
            spec.mass_kg * spec.specific_heat_j_kg_k,
            spec.initial_temperature_k,
        );
        net.add_conductance(node, sink, spec.conductance_to_sink_w_k);
        Self { node, spec: *spec }
    }

    pub fn node(&self) -> NodeId {
        self.node
    }

    pub fn temperature(&self, net: &Network) -> f64 {
        net.temperature(self.node)
    }

    pub fn resistance_at(&self, temperature_k: f64) -> f64 {
        let dt = (REFERENCE_TEMPERATURE_K - temperature_k).clamp(-40.0, 80.0);
        self.spec.resistance_ohm_20c * (self.spec.resistance_cold_coeff_per_k * dt).exp()
    }

    pub fn generation_w(&self, net: &Network, current_a: f64) -> f64 {
        let t = net.temperature(self.node);
        current_a * current_a * self.resistance_at(t) + current_a * t * self.spec.entropic_v_per_k
    }

    pub fn apply(&self, net: &mut Network, current_a: f64) -> f64 {
        let q = self.generation_w(net, current_a);
        net.set_source(self.node, q);
        q
    }
}

#[derive(Clone, Copy, Debug)]
pub struct LruSpec {
    pub core_capacity_j_k: f64,
    pub shell_capacity_j_k: f64,
    pub core_to_shell_w_k: f64,
    pub shell_to_ambient_w_k: f64,
    pub cooling_ua_w_k: f64,
    pub dissipation_w: f64,
    pub initial_temperature_k: f64,
}

#[derive(Clone, Copy, Debug)]
pub struct LruBox {
    core: NodeId,
    shell: NodeId,
    cooling_link: LinkId,
    cooling_ua_w_k: f64,
}

impl LruBox {
    pub fn add(net: &mut Network, spec: &LruSpec, ambient: NodeId, cooling_inlet: NodeId) -> Self {
        let core = net.add_node(spec.core_capacity_j_k, spec.initial_temperature_k);
        let shell = net.add_node(spec.shell_capacity_j_k, spec.initial_temperature_k);
        net.add_conductance(core, shell, spec.core_to_shell_w_k);
        net.add_conductance(shell, ambient, spec.shell_to_ambient_w_k);
        let cooling_link = net.add_conductance(core, cooling_inlet, 0.0);
        net.set_source(core, spec.dissipation_w);
        Self {
            core,
            shell,
            cooling_link,
            cooling_ua_w_k: spec.cooling_ua_w_k,
        }
    }

    pub fn core(&self) -> NodeId {
        self.core
    }

    pub fn shell(&self) -> NodeId {
        self.shell
    }

    pub fn core_temperature(&self, net: &Network) -> f64 {
        net.temperature(self.core)
    }

    pub fn set_dissipation(&self, net: &mut Network, watts: f64) {
        net.set_source(self.core, watts);
    }

    pub fn set_cooling_flow(&self, net: &mut Network, mass_flow_kg_s: f64) {
        let c = mass_flow_kg_s * AIR_CP;
        let g = if c > 1e-9 {
            c * (1.0 - (-self.cooling_ua_w_k / c).exp())
        } else {
            0.0
        };
        net.set_conductance(self.cooling_link, g);
    }

    pub fn cooling_conductance(&self, net: &Network) -> f64 {
        net.link_conductance(self.cooling_link)
    }
}

pub fn soak_temperature(initial_k: f64, ambient_k: f64, tau_s: f64, time_s: f64) -> f64 {
    ambient_k + (initial_k - ambient_k) * (-time_s / tau_s).exp()
}

pub fn soak_time_constant(capacity_j_k: f64, conductance_w_k: f64) -> f64 {
    capacity_j_k / conductance_w_k
}
