use super::{ElectricalNet, R_SWITCH};
use solver_electrical::{Complex, Element, ElementId, LoadKind, NodeId, NodeKind};
use std::collections::BTreeMap;

const R_SHUNT_OFF: f64 = 1.0e9;
const R_OPEN_AC: f64 = 1.0e4;
const R_OPEN_RECT: f64 = 1.0e3;
const R_SHUNT_MIN: f64 = 1.0e-3;

#[derive(Clone, Debug, Default)]
pub(super) struct Overlay {
    shunts: BTreeMap<String, ElementId>,
    bases: BTreeMap<usize, Element>,
    pub(super) diode_force: BTreeMap<usize, bool>,
}

impl ElectricalNet {
    pub fn shunt_site(&self, target: &str) -> Option<String> {
        if let Some(c) = self.contactors.iter().find(|c| c.name == target)
            && let Some(e) = c.element
            && let Element::Switch { a, .. } = *self.network.element(e)
        {
            return self.node_name(a);
        }
        let s = self.sspcs.iter().find(|s| s.id == target)?;
        let Element::Switch { b, .. } = *self.network.element(s.switch?) else {
            return None;
        };
        self.node_name(b)
    }

    fn node_name(&self, n: NodeId) -> Option<String> {
        self.nodes.iter().find(|(_, v)| **v == n).map(|(k, _)| k.clone())
    }

    pub fn set_phase_shunt(&mut self, key: &str, node: &str, ohm: Option<f64>) -> bool {
        let Some(&n) = self.nodes.get(node) else {
            return false;
        };
        if self.network.node_kind(n) != NodeKind::Ac3 {
            return self.set_shunt(key, node, ohm);
        }
        if ohm.is_none() && !self.overlay.shunts.contains_key(key) {
            return true;
        }
        let z = Complex::new(ohm.unwrap_or(R_SHUNT_OFF).max(R_SHUNT_MIN), 0.0);
        let off = LoadKind::Impedance {
            z: Complex::new(R_SHUNT_OFF, 0.0),
        };
        let el = Element::AcLoad {
            node: n,
            phases: [LoadKind::Impedance { z }, off, off],
        };
        match self.overlay.shunts.get(key).copied() {
            Some(id) => self.network.set_element(id, el).is_ok(),
            None => match self.network.add_element(el) {
                Ok(id) => {
                    self.overlay.shunts.insert(key.to_string(), id);
                    true
                }
                Err(_) => false,
            },
        }
    }

    pub fn set_shunt(&mut self, key: &str, node: &str, ohm: Option<f64>) -> bool {
        let Some(&n) = self.nodes.get(node) else {
            return false;
        };
        if ohm.is_none() && !self.overlay.shunts.contains_key(key) {
            return true;
        }
        let z = Complex::new(ohm.unwrap_or(R_SHUNT_OFF).max(R_SHUNT_MIN), 0.0);
        let el = if self.network.node_kind(n) == NodeKind::Ac3 {
            Element::AcLoad {
                node: n,
                phases: [LoadKind::Impedance { z }; 3],
            }
        } else {
            Element::DcLoad {
                node: n,
                kind: LoadKind::Impedance { z },
            }
        };
        match self.overlay.shunts.get(key).copied() {
            Some(id) => self.network.set_element(id, el).is_ok(),
            None => match self.network.add_element(el) {
                Ok(id) => {
                    self.overlay.shunts.insert(key.to_string(), id);
                    true
                }
                Err(_) => false,
            },
        }
    }

    pub fn set_converter_mod(&mut self, node: &str, gain: f64, z_scale: f64, open: bool) -> bool {
        let Some(&n) = self.nodes.get(node) else {
            return false;
        };
        let neutral = !open && (gain - 1.0).abs() < 1.0e-12 && (z_scale - 1.0).abs() < 1.0e-12;
        let mut hit = false;
        for i in 0..self.network.element_count() {
            let el = *self.network.element(ElementId(i));
            let matches = match el {
                Element::Autotransformer { input, .. } => input == n,
                Element::Rectifier { ac, .. } => ac == n,
                _ => false,
            };
            if !matches {
                continue;
            }
            hit = true;
            if neutral {
                if let Some(base) = self.overlay.bases.remove(&i) {
                    let _ = self.network.set_element(ElementId(i), base);
                }
                continue;
            }
            let base = *self.overlay.bases.entry(i).or_insert(el);
            let next = match base {
                Element::Autotransformer {
                    input,
                    output,
                    ratio,
                    r_leak,
                    x_leak,
                    efficiency,
                    in_v_nominal,
                    r_neutral_ground,
                } => Element::Autotransformer {
                    input,
                    output,
                    ratio: ratio * gain,
                    r_leak: if open { R_OPEN_AC } else { r_leak * z_scale },
                    x_leak: if open { R_OPEN_AC } else { x_leak * z_scale },
                    efficiency,
                    in_v_nominal,
                    r_neutral_ground,
                },
                Element::Rectifier {
                    ac,
                    pos,
                    neg,
                    ratio,
                    r_out,
                    efficiency,
                    ac_v_nominal,
                } => Element::Rectifier {
                    ac,
                    pos,
                    neg,
                    ratio: ratio * gain,
                    r_out: if open { R_OPEN_RECT } else { r_out * z_scale },
                    efficiency,
                    ac_v_nominal,
                },
                other => other,
            };
            let _ = self.network.set_element(ElementId(i), next);
        }
        hit
    }

    pub fn set_battery_diode_force(&mut self, battery: &str, state: Option<bool>) -> bool {
        let Some(b) = self.batteries.iter().find(|b| b.name == battery) else {
            return false;
        };
        let node = b.node;
        let Some(i) = self.diodes.iter().position(|d| d.inner == node) else {
            return false;
        };
        match state {
            Some(s) => {
                self.overlay.diode_force.insert(i, s);
            }
            None => {
                self.overlay.diode_force.remove(&i);
            }
        }
        true
    }

    pub fn set_battery_diode_drop_ohm(&mut self, battery: &str, extra_ohm: f64) -> bool {
        let Some(b) = self.batteries.iter().find(|b| b.name == battery) else {
            return false;
        };
        let node = b.node;
        let Some(d) = self.diodes.iter().find(|d| d.inner == node) else {
            return false;
        };
        let id = d.switch;
        self.series_switch(id, extra_ohm)
    }

    fn series_switch(&mut self, id: ElementId, extra_ohm: f64) -> bool {
        let Element::Switch { a, b, closed, .. } = *self.network.element(id) else {
            return false;
        };
        self.network
            .set_element(
                id,
                Element::Switch {
                    a,
                    b,
                    closed,
                    r_closed: R_SWITCH + extra_ohm.max(0.0),
                },
            )
            .is_ok()
    }

    pub fn set_rpdu_feed_series_ohm(&mut self, rpdu_number: &str, extra_ohm: f64) -> bool {
        let Some(&dc) = self.nodes.get(&format!("RPDU {rpdu_number} DC")) else {
            return false;
        };
        let ids: Vec<ElementId> = self
            .diodes
            .iter()
            .filter(|d| d.far == dc)
            .map(|d| d.switch)
            .collect();
        let mut any = false;
        for id in ids {
            any |= self.series_switch(id, extra_ohm);
        }
        any
    }

    pub fn rpdu_numbers(&self) -> Vec<String> {
        let mut out: Vec<String> = self
            .nodes
            .keys()
            .filter_map(|k| k.strip_prefix("RPDU ")?.strip_suffix(" DC").map(str::to_string))
            .collect();
        out.sort();
        out
    }

    pub fn rpdu_has_ac(&self, rpdu_number: &str) -> bool {
        self.nodes.contains_key(&format!("RPDU {rpdu_number} AC"))
    }

    pub fn feed_sspc_ids(&self) -> Vec<String> {
        self.sspcs
            .iter()
            .filter(|s| s.switch.is_some())
            .map(|s| s.id.clone())
            .collect()
    }

    pub fn sspc_rpdu(&self, id: &str) -> Option<String> {
        self.sspcs.iter().find(|s| s.id == id).map(|s| s.rpdu.clone())
    }
}
