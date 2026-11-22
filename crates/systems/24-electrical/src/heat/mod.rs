mod gauge;
#[cfg(test)]
mod tests;

pub use gauge::{
    AMBIENT_REF_K, BUNDLE_DERATE, GAUGES, Gauge, MAX_CONDUCTOR_K, Sizing, ambient_derate, size_wire,
};

use solver_thermal::{
    BatteryCell, BatterySpec, Conductor, ContactSpec, ContactorContact, LinkId, LruBox, LruSpec, Network,
    NodeId, StepReport, WireSegment, WireSpec, conductor_resistance, wire_ampacity,
};
use std::collections::BTreeMap;

const MAX_SUBSTEP_S: f64 = 1.0;
const COPPER_ALPHA_PER_K: f64 = 0.00393;
const COOLING_AIR_K: f64 = 295.15;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Zone {
    EeBayFwd,
    EeBayAft,
    Crown,
    WheelWell,
    Nacelle,
    Cabin,
}

pub const ZONES: [Zone; 6] = [
    Zone::EeBayFwd,
    Zone::EeBayAft,
    Zone::Crown,
    Zone::WheelWell,
    Zone::Nacelle,
    Zone::Cabin,
];

impl Zone {
    pub fn index(self) -> usize {
        match self {
            Zone::EeBayFwd => 0,
            Zone::EeBayAft => 1,
            Zone::Crown => 2,
            Zone::WheelWell => 3,
            Zone::Nacelle => 4,
            Zone::Cabin => 5,
        }
    }

    pub fn default_k(self) -> f64 {
        match self {
            Zone::EeBayFwd | Zone::EeBayAft => 298.15,
            Zone::Crown => 293.15,
            Zone::WheelWell => 288.15,
            Zone::Nacelle => 308.15,
            Zone::Cabin => 295.15,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ModuleKind {
    Sspc,
    Rpdu,
    Spdu,
    PanelBox,
}

impl ModuleKind {
    fn spec(self) -> (LruSpec, f64) {
        let (core, shell, cs, sa, cool, r_on) = match self {
            ModuleKind::Sspc => (60.0, 150.0, 1.5, 0.4, 0.8, 0.03),
            ModuleKind::Rpdu => (900.0, 1800.0, 6.0, 3.0, 8.0, 0.02),
            ModuleKind::Spdu => (2500.0, 5000.0, 12.0, 6.0, 15.0, 0.01),
            ModuleKind::PanelBox => (4000.0, 12000.0, 8.0, 8.0, 0.0, 0.005),
        };
        (
            LruSpec {
                core_capacity_j_k: core,
                shell_capacity_j_k: shell,
                core_to_shell_w_k: cs,
                shell_to_ambient_w_k: sa,
                cooling_ua_w_k: cool,
                dissipation_w: 0.0,
                initial_temperature_k: 0.0,
            },
            r_on,
        )
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PartKind {
    Wire,
    Contactor,
    Battery,
    Module,
    Generator,
}

#[derive(Clone, Debug, PartialEq)]
pub enum PartSpec {
    Wire {
        id: String,
        zone: Zone,
        rating_a: f64,
        length_m: f64,
    },
    Contactor {
        id: String,
        zone: Zone,
        rating_a: f64,
    },
    Battery {
        id: String,
        zone: Zone,
        cells: u32,
        cell_mass_kg: f64,
        cell_r_ohm_20c: f64,
    },
    Module {
        id: String,
        zone: Zone,
        kind: ModuleKind,
        dissipation_w: f64,
    },
    Generator {
        id: String,
        zone: Zone,
        rated_current_a: f64,
        rated_loss_w: f64,
    },
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum HeatError {
    DuplicateId(String),
    UnknownId(String),
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct EnergyReport {
    pub elapsed_s: f64,
    pub source_j: f64,
    pub boundary_j: f64,
    pub stored_delta_j: f64,
    pub imbalance_j: f64,
    pub stored_j: f64,
}

#[derive(Clone, Copy, Debug)]
enum Model {
    Wire {
        seg: WireSegment,
        spec: WireSpec,
        parallel: f64,
    },
    Contact {
        contact: ContactorContact,
        case: NodeId,
    },
    Battery {
        cell: BatteryCell,
    },
    Module {
        lru: LruBox,
        r_on_ohm: f64,
    },
    Generator {
        winding: NodeId,
        oil: NodeId,
        cooler: LinkId,
        cooler_g_w_k: f64,
        r_equiv_ohm: f64,
    },
}

#[derive(Clone, Copy, Debug)]
struct Part {
    zone: Zone,
    model: Model,
    current_a: f64,
    extra_w: f64,
    r_mult: f64,
    heat_w: f64,
}

impl Part {
    fn new(zone: Zone, model: Model, extra_w: f64) -> Self {
        Self {
            zone,
            model,
            current_a: 0.0,
            extra_w,
            r_mult: 1.0,
            heat_w: extra_w,
        }
    }
}

#[derive(Clone, Debug)]
pub struct ElecThermal {
    nets: Vec<Network>,
    ambient: Vec<NodeId>,
    cooling: Vec<NodeId>,
    parts: BTreeMap<String, Part>,
    totals: EnergyReport,
}

impl Default for ElecThermal {
    fn default() -> Self {
        Self::new()
    }
}

fn copper_factor(temp_k: f64) -> f64 {
    (1.0 + COPPER_ALPHA_PER_K * (temp_k - AMBIENT_REF_K)).max(0.05)
}

impl ElecThermal {
    pub fn new() -> Self {
        let mut nets = Vec::new();
        let mut ambient = Vec::new();
        let mut cooling = Vec::new();
        for z in ZONES {
            let mut n = Network::new();
            ambient.push(n.add_boundary(z.default_k()));
            cooling.push(n.add_boundary(COOLING_AIR_K));
            nets.push(n);
        }
        Self {
            nets,
            ambient,
            cooling,
            parts: BTreeMap::new(),
            totals: EnergyReport::default(),
        }
    }

    pub fn from_parts(specs: &[PartSpec]) -> Result<Self, HeatError> {
        let mut t = Self::new();
        for s in specs {
            match s {
                PartSpec::Wire {
                    id,
                    zone,
                    rating_a,
                    length_m,
                } => {
                    t.add_wire(id, *zone, *rating_a, *length_m)?;
                }
                PartSpec::Contactor { id, zone, rating_a } => t.add_contactor(id, *zone, *rating_a)?,
                PartSpec::Battery {
                    id,
                    zone,
                    cells,
                    cell_mass_kg,
                    cell_r_ohm_20c,
                } => t.add_battery(id, *zone, *cells, *cell_mass_kg, *cell_r_ohm_20c)?,
                PartSpec::Module {
                    id,
                    zone,
                    kind,
                    dissipation_w,
                } => t.add_module(id, *zone, *kind, *dissipation_w)?,
                PartSpec::Generator {
                    id,
                    zone,
                    rated_current_a,
                    rated_loss_w,
                } => t.add_generator(id, *zone, *rated_current_a, *rated_loss_w)?,
            }
        }
        Ok(t)
    }

    fn check_free(&self, id: &str) -> Result<(), HeatError> {
        if self.parts.contains_key(id) {
            Err(HeatError::DuplicateId(id.to_string()))
        } else {
            Ok(())
        }
    }

    fn zone_temp(&self, zone: Zone) -> f64 {
        self.nets[zone.index()].temperature(self.ambient[zone.index()])
    }

    pub fn add_wire(
        &mut self,
        id: &str,
        zone: Zone,
        rating_a: f64,
        length_m: f64,
    ) -> Result<Sizing, HeatError> {
        self.check_free(id)?;
        let sizing = size_wire(rating_a);
        let g = sizing.gauge;
        let od = g.outer_diameter_m();
        let r_max = conductor_resistance(Conductor::Copper, 1.0, g.area_m2, MAX_CONDUCTOR_K);
        let i = g.bundle_limit_a();
        let h = i * i * r_max / (std::f64::consts::PI * od * (MAX_CONDUCTOR_K - AMBIENT_REF_K));
        let spec = WireSpec {
            conductor: Conductor::Copper,
            length_m: length_m.max(0.01),
            area_m2: g.area_m2,
            outer_diameter_m: od,
            surface_h_w_m2k: h,
            extra_capacity_j_k: 0.0,
            initial_temperature_k: self.zone_temp(zone),
        };
        let zi = zone.index();
        let seg = WireSegment::add(&mut self.nets[zi], &spec, self.ambient[zi]);
        let model = Model::Wire {
            seg,
            spec,
            parallel: f64::from(sizing.parallel),
        };
        self.parts.insert(id.to_string(), Part::new(zone, model, 0.0));
        Ok(sizing)
    }

    pub fn add_contactor(&mut self, id: &str, zone: Zone, rating_a: f64) -> Result<(), HeatError> {
        self.check_free(id)?;
        let rating = rating_a.max(1.0);
        let zi = zone.index();
        let t0 = self.zone_temp(zone);
        let case = self.nets[zi].add_node(5.0 * rating + 50.0, t0);
        self.nets[zi].add_conductance(case, self.ambient[zi], 0.01 * rating + 0.5);
        let spec = ContactSpec {
            resistance_ohm_20c: 0.05 / rating,
            alpha_per_k: COPPER_ALPHA_PER_K,
            capacity_j_k: 0.5 * rating + 10.0,
            conductance_to_case_w_k: 0.006 * rating + 0.3,
            initial_temperature_k: t0,
        };
        let contact = ContactorContact::add(&mut self.nets[zi], &spec, case);
        self.parts.insert(
            id.to_string(),
            Part::new(zone, Model::Contact { contact, case }, 0.0),
        );
        Ok(())
    }

    pub fn add_battery(
        &mut self,
        id: &str,
        zone: Zone,
        cells: u32,
        cell_mass_kg: f64,
        cell_r_ohm_20c: f64,
    ) -> Result<(), HeatError> {
        self.check_free(id)?;
        let n = f64::from(cells.max(1));
        let mass = n * cell_mass_kg.max(0.01);
        let spec = BatterySpec {
            mass_kg: mass,
            specific_heat_j_kg_k: 900.0,
            resistance_ohm_20c: n * cell_r_ohm_20c.max(1e-6),
            resistance_cold_coeff_per_k: 0.025,
            entropic_v_per_k: 5e-5 * n,
            conductance_to_sink_w_k: 0.12 * mass,
            initial_temperature_k: self.zone_temp(zone),
        };
        let zi = zone.index();
        let cell = BatteryCell::add(&mut self.nets[zi], &spec, self.ambient[zi]);
        self.parts
            .insert(id.to_string(), Part::new(zone, Model::Battery { cell }, 0.0));
        Ok(())
    }

    pub fn add_module(
        &mut self,
        id: &str,
        zone: Zone,
        kind: ModuleKind,
        dissipation_w: f64,
    ) -> Result<(), HeatError> {
        self.check_free(id)?;
        let (mut spec, r_on_ohm) = kind.spec();
        spec.initial_temperature_k = self.zone_temp(zone);
        spec.dissipation_w = dissipation_w;
        let zi = zone.index();
        let lru = LruBox::add(&mut self.nets[zi], &spec, self.ambient[zi], self.cooling[zi]);
        self.parts.insert(
            id.to_string(),
            Part::new(zone, Model::Module { lru, r_on_ohm }, dissipation_w),
        );
        Ok(())
    }

    pub fn add_generator(
        &mut self,
        id: &str,
        zone: Zone,
        rated_current_a: f64,
        rated_loss_w: f64,
    ) -> Result<(), HeatError> {
        self.check_free(id)?;
        let zi = zone.index();
        let t0 = self.zone_temp(zone);
        let winding = self.nets[zi].add_node(12_000.0, t0);
        let oil = self.nets[zi].add_node(20_000.0, t0);
        self.nets[zi].add_conductance(winding, oil, 400.0);
        self.nets[zi].add_conductance(oil, self.ambient[zi], 8.0);
        let cooler_g_w_k = 300.0;
        let cooler = self.nets[zi].add_conductance(oil, self.cooling[zi], cooler_g_w_k);
        let i = rated_current_a.max(1.0);
        let model = Model::Generator {
            winding,
            oil,
            cooler,
            cooler_g_w_k,
            r_equiv_ohm: rated_loss_w.max(0.0) / (i * i),
        };
        self.parts.insert(id.to_string(), Part::new(zone, model, 0.0));
        Ok(())
    }

    pub fn set_current(&mut self, part_id: &str, amps: f64) -> bool {
        self.parts.get_mut(part_id).map(|p| p.current_a = amps).is_some()
    }

    pub fn set_dissipation(&mut self, part_id: &str, watts: f64) -> bool {
        self.parts.get_mut(part_id).map(|p| p.extra_w = watts).is_some()
    }

    pub fn set_resistance_multiplier(&mut self, part_id: &str, mult: f64) -> bool {
        self.parts
            .get_mut(part_id)
            .map(|p| p.r_mult = mult.max(0.0))
            .is_some()
    }

    pub fn set_cooling_flow(&mut self, part_id: &str, mass_flow_kg_s: f64) -> bool {
        let Some(p) = self.parts.get(part_id) else {
            return false;
        };
        let zi = p.zone.index();
        match p.model {
            Model::Module { lru, .. } => {
                lru.set_cooling_flow(&mut self.nets[zi], mass_flow_kg_s);
                true
            }
            _ => false,
        }
    }

    pub fn set_oil_cooling(&mut self, part_id: &str, fraction: f64) -> bool {
        let Some(p) = self.parts.get(part_id) else {
            return false;
        };
        let zi = p.zone.index();
        match p.model {
            Model::Generator {
                cooler, cooler_g_w_k, ..
            } => {
                self.nets[zi].set_conductance(cooler, cooler_g_w_k * fraction.clamp(0.0, 1.0));
                true
            }
            _ => false,
        }
    }

    pub fn set_zone_temp(&mut self, zone: Zone, kelvin: f64) {
        let zi = zone.index();
        self.nets[zi].set_temperature(self.ambient[zi], kelvin);
    }

    pub fn set_cooling_air_k(&mut self, zone: Zone, kelvin: f64) {
        let zi = zone.index();
        self.nets[zi].set_temperature(self.cooling[zi], kelvin);
    }

    pub fn zone_temp_k(&self, zone: Zone) -> f64 {
        self.zone_temp(zone)
    }

    pub fn cold_soak(&mut self, zone: Zone, kelvin: f64) {
        let zi = zone.index();
        self.set_zone_temp(zone, kelvin);
        self.set_cooling_air_k(zone, kelvin);
        for p in self.parts.values().filter(|p| p.zone == zone) {
            let nodes: [Option<NodeId>; 2] = match p.model {
                Model::Wire { seg, .. } => [Some(seg.node()), None],
                Model::Contact { contact, case } => [Some(contact.node()), Some(case)],
                Model::Battery { cell } => [Some(cell.node()), None],
                Model::Module { lru, .. } => [Some(lru.core()), Some(lru.shell())],
                Model::Generator { winding, oil, .. } => [Some(winding), Some(oil)],
            };
            for n in nodes.into_iter().flatten() {
                self.nets[zi].set_temperature(n, kelvin);
            }
        }
    }

    fn apply_sources(&mut self) {
        for p in self.parts.values_mut() {
            let net = &mut self.nets[p.zone.index()];
            let i = p.current_a;
            let q = match p.model {
                Model::Wire { seg, parallel, .. } => {
                    let per = i / parallel;
                    let q = per * per * seg.resistance(net) * p.r_mult + p.extra_w;
                    net.set_source(seg.node(), q);
                    q
                }
                Model::Contact { contact, .. } => {
                    let q = i * i * contact.resistance(net) * p.r_mult + p.extra_w;
                    net.set_source(contact.node(), q);
                    q
                }
                Model::Battery { cell } => {
                    let t = cell.temperature(net);
                    let q = cell.generation_w(net, i)
                        + i * i * cell.resistance_at(t) * (p.r_mult - 1.0)
                        + p.extra_w;
                    net.set_source(cell.node(), q);
                    q
                }
                Model::Module { lru, r_on_ohm } => {
                    let q = p.extra_w + i * i * r_on_ohm * p.r_mult;
                    lru.set_dissipation(net, q);
                    q
                }
                Model::Generator {
                    winding, r_equiv_ohm, ..
                } => {
                    let t = net.temperature(winding);
                    let q = i * i * r_equiv_ohm * copper_factor(t) * p.r_mult + p.extra_w;
                    net.set_source(winding, q);
                    q
                }
            };
            p.heat_w = q;
        }
    }

    pub fn step(&mut self, dt_us: u64) -> StepReport {
        let total_s = dt_us as f64 * 1e-6;
        let mut out = StepReport::default();
        if total_s <= 0.0 {
            return out;
        }
        let n = (total_s / MAX_SUBSTEP_S).ceil().max(1.0) as u32;
        let dt = total_s / f64::from(n);
        for _ in 0..n {
            self.apply_sources();
            for net in &mut self.nets {
                let r = net.step(dt);
                out.source_j += r.source_j;
                out.boundary_j += r.boundary_j;
                out.energy_in_j += r.energy_in_j;
                out.energy_out_j += r.energy_out_j;
                out.stored_delta_j += r.stored_delta_j;
                out.imbalance_j += r.imbalance_j;
            }
        }
        out.dt_s = total_s;
        self.totals.elapsed_s += total_s;
        self.totals.source_j += out.source_j;
        self.totals.boundary_j += out.boundary_j;
        self.totals.stored_delta_j += out.stored_delta_j;
        self.totals.imbalance_j += out.imbalance_j;
        self.totals.stored_j = self.nets.iter().map(Network::stored_energy_j).sum();
        out
    }

    pub fn energy_report(&self) -> EnergyReport {
        self.totals
    }

    pub fn contains(&self, part_id: &str) -> bool {
        self.parts.contains_key(part_id)
    }

    pub fn part_ids(&self) -> Vec<&str> {
        self.parts.keys().map(String::as_str).collect()
    }

    pub fn len(&self) -> usize {
        self.parts.len()
    }

    pub fn is_empty(&self) -> bool {
        self.parts.is_empty()
    }

    pub fn zone_of(&self, part_id: &str) -> Option<Zone> {
        self.parts.get(part_id).map(|p| p.zone)
    }

    pub fn kind_of(&self, part_id: &str) -> Option<PartKind> {
        self.parts.get(part_id).map(|p| match p.model {
            Model::Wire { .. } => PartKind::Wire,
            Model::Contact { .. } => PartKind::Contactor,
            Model::Battery { .. } => PartKind::Battery,
            Model::Module { .. } => PartKind::Module,
            Model::Generator { .. } => PartKind::Generator,
        })
    }

    pub fn temperature_k(&self, part_id: &str) -> Option<f64> {
        let p = self.parts.get(part_id)?;
        let net = &self.nets[p.zone.index()];
        Some(match p.model {
            Model::Wire { seg, .. } => net.temperature(seg.node()),
            Model::Contact { contact, .. } => net.temperature(contact.node()),
            Model::Battery { cell } => net.temperature(cell.node()),
            Model::Module { lru, .. } => net.temperature(lru.core()),
            Model::Generator { oil, .. } => net.temperature(oil),
        })
    }

    pub fn secondary_temperature_k(&self, part_id: &str) -> Option<f64> {
        let p = self.parts.get(part_id)?;
        let net = &self.nets[p.zone.index()];
        match p.model {
            Model::Contact { case, .. } => Some(net.temperature(case)),
            Model::Module { lru, .. } => Some(net.temperature(lru.shell())),
            Model::Generator { winding, .. } => Some(net.temperature(winding)),
            _ => None,
        }
    }

    pub fn heat_w(&self, part_id: &str) -> Option<f64> {
        self.parts.get(part_id).map(|p| p.heat_w)
    }

    pub fn resistance_factor(&self, part_id: &str) -> Option<f64> {
        let p = self.parts.get(part_id)?;
        let net = &self.nets[p.zone.index()];
        Some(match p.model {
            Model::Wire { seg, .. } => copper_factor(seg.temperature(net)),
            Model::Contact { contact, .. } => copper_factor(contact.temperature(net)),
            Model::Battery { cell } => {
                cell.resistance_at(cell.temperature(net)) / cell.resistance_at(AMBIENT_REF_K)
            }
            Model::Module { lru, .. } => copper_factor(lru.core_temperature(net)),
            Model::Generator { winding, .. } => copper_factor(net.temperature(winding)),
        })
    }

    pub fn resistance_ohm(&self, part_id: &str) -> Option<f64> {
        let p = self.parts.get(part_id)?;
        let net = &self.nets[p.zone.index()];
        match p.model {
            Model::Wire { seg, parallel, .. } => Some(seg.resistance(net) / parallel),
            Model::Contact { contact, .. } => Some(contact.resistance(net)),
            _ => None,
        }
    }

    pub fn ampacity_a(&self, part_id: &str) -> Option<f64> {
        let p = self.parts.get(part_id)?;
        match p.model {
            Model::Wire { spec, parallel, .. } => {
                Some(wire_ampacity(&spec, self.zone_temp(p.zone), MAX_CONDUCTOR_K) * parallel)
            }
            _ => None,
        }
    }

    pub fn margin_to_limit_k(&self, part_id: &str) -> Option<f64> {
        let p = self.parts.get(part_id)?;
        match p.model {
            Model::Wire { .. } => Some(MAX_CONDUCTOR_K - self.temperature_k(part_id)?),
            _ => None,
        }
    }
}
