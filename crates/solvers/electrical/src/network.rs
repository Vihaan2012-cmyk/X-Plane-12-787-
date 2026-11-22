use solver_common::Complex;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct NodeId(pub usize);

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct ElementId(pub usize);

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum NodeKind {
    Dc,
    Ac3,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PowerSense {
    Absorbed,
    Delivered,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum LoadKind {
    Impedance {
        z: Complex,
    },
    Current {
        amps: f64,
        power_factor: f64,
        v_nominal: f64,
    },
    Power {
        p: f64,
        q: f64,
        v_nominal: f64,
    },
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Element {
    Resistor {
        a: NodeId,
        b: NodeId,
        r: f64,
    },
    Impedance {
        a: NodeId,
        b: NodeId,
        z_phase: Complex,
        z_neutral: Complex,
    },
    Switch {
        a: NodeId,
        b: NodeId,
        closed: bool,
        r_closed: f64,
    },
    DcSource {
        node: NodeId,
        e: f64,
        r: f64,
    },
    AcSource {
        node: NodeId,
        emf: f64,
        angle: f64,
        r: f64,
        x: f64,
        frequency_hz: f64,
        r_neutral_ground: f64,
    },
    DcCurrentSource {
        node: NodeId,
        amps: f64,
    },
    AcCurrentSource {
        node: NodeId,
        amps: f64,
        angle: f64,
    },
    DcLoad {
        node: NodeId,
        kind: LoadKind,
    },
    AcLoad {
        node: NodeId,
        phases: [LoadKind; 3],
    },
    Rectifier {
        ac: NodeId,
        pos: NodeId,
        neg: Option<NodeId>,
        ratio: f64,
        r_out: f64,
        efficiency: f64,
        ac_v_nominal: f64,
    },
    Autotransformer {
        input: NodeId,
        output: NodeId,
        ratio: f64,
        r_leak: f64,
        x_leak: f64,
        efficiency: f64,
        in_v_nominal: f64,
        r_neutral_ground: f64,
    },
}

impl Element {
    pub fn balanced_ac_load(kind: LoadKind, node: NodeId) -> Element {
        Element::AcLoad {
            node,
            phases: [kind; 3],
        }
    }

    pub fn power_sense(&self) -> PowerSense {
        match self {
            Element::DcSource { .. }
            | Element::AcSource { .. }
            | Element::DcCurrentSource { .. }
            | Element::AcCurrentSource { .. } => PowerSense::Delivered,
            _ => PowerSense::Absorbed,
        }
    }

    fn topology_key(&self) -> (u8, [usize; 3], bool) {
        match *self {
            Element::Resistor { a, b, .. } => (0, [a.0, b.0, 0], true),
            Element::Impedance { a, b, .. } => (1, [a.0, b.0, 0], true),
            Element::Switch { a, b, closed, .. } => (2, [a.0, b.0, 0], closed),
            Element::DcSource { node, .. } => (3, [node.0, 0, 0], true),
            Element::AcSource { node, .. } => (4, [node.0, 0, 0], true),
            Element::DcCurrentSource { node, .. } => (5, [node.0, 0, 0], true),
            Element::AcCurrentSource { node, .. } => (6, [node.0, 0, 0], true),
            Element::DcLoad { node, .. } => (7, [node.0, 0, 0], true),
            Element::AcLoad { node, .. } => (8, [node.0, 0, 0], true),
            Element::Rectifier { ac, pos, neg, .. } => {
                (9, [ac.0, pos.0, neg.map_or(usize::MAX, |n| n.0)], true)
            }
            Element::Autotransformer { input, output, .. } => (10, [input.0, output.0, 0], true),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum NetworkError {
    UnknownNode,
    UnknownElement,
    KindMismatch,
}

#[derive(Clone, Debug, Default)]
pub struct Network {
    kinds: Vec<NodeKind>,
    elements: Vec<Element>,
    version: u64,
}

impl Network {
    pub fn new() -> Self {
        Network::default()
    }

    pub fn add_node(&mut self, kind: NodeKind) -> NodeId {
        self.kinds.push(kind);
        self.version += 1;
        NodeId(self.kinds.len() - 1)
    }

    pub fn node_count(&self) -> usize {
        self.kinds.len()
    }

    pub fn element_count(&self) -> usize {
        self.elements.len()
    }

    pub fn node_kind(&self, n: NodeId) -> NodeKind {
        self.kinds[n.0]
    }

    pub fn element(&self, e: ElementId) -> &Element {
        &self.elements[e.0]
    }

    pub fn elements(&self) -> &[Element] {
        &self.elements
    }

    pub fn topology_version(&self) -> u64 {
        self.version
    }

    fn check(&self, e: &Element) -> Result<(), NetworkError> {
        let kind = |n: NodeId| self.kinds.get(n.0).copied().ok_or(NetworkError::UnknownNode);
        let want = |n: NodeId, k: NodeKind| {
            if kind(n)? == k {
                Ok(())
            } else {
                Err(NetworkError::KindMismatch)
            }
        };
        match *e {
            Element::Resistor { a, b, .. }
            | Element::Impedance { a, b, .. }
            | Element::Switch { a, b, .. } => {
                if kind(a)? == kind(b)? {
                    Ok(())
                } else {
                    Err(NetworkError::KindMismatch)
                }
            }
            Element::DcSource { node, .. }
            | Element::DcCurrentSource { node, .. }
            | Element::DcLoad { node, .. } => want(node, NodeKind::Dc),
            Element::AcSource { node, .. }
            | Element::AcCurrentSource { node, .. }
            | Element::AcLoad { node, .. } => want(node, NodeKind::Ac3),
            Element::Rectifier { ac, pos, neg, .. } => {
                want(ac, NodeKind::Ac3)?;
                want(pos, NodeKind::Dc)?;
                match neg {
                    Some(n) => want(n, NodeKind::Dc),
                    None => Ok(()),
                }
            }
            Element::Autotransformer { input, output, .. } => {
                want(input, NodeKind::Ac3)?;
                want(output, NodeKind::Ac3)
            }
        }
    }

    pub fn add_element(&mut self, e: Element) -> Result<ElementId, NetworkError> {
        self.check(&e)?;
        self.elements.push(e);
        self.version += 1;
        Ok(ElementId(self.elements.len() - 1))
    }

    pub fn set_element(&mut self, id: ElementId, e: Element) -> Result<(), NetworkError> {
        self.check(&e)?;
        let slot = self.elements.get_mut(id.0).ok_or(NetworkError::UnknownElement)?;
        if slot.topology_key() != e.topology_key() {
            self.version += 1;
        }
        *slot = e;
        Ok(())
    }

    pub fn set_switch(&mut self, id: ElementId, closed: bool) -> Result<(), NetworkError> {
        match self.elements.get_mut(id.0) {
            Some(Element::Switch { closed: c, .. }) => {
                if *c != closed {
                    *c = closed;
                    self.version += 1;
                }
                Ok(())
            }
            Some(_) => Err(NetworkError::KindMismatch),
            None => Err(NetworkError::UnknownElement),
        }
    }
}
