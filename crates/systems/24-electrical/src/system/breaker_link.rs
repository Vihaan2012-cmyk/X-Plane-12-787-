use super::Electrical;
use crate::protection::elcu::{Elcc, ElccInput};
use crate::protection::thermal_breaker::{BreakerInput, ThermalBreaker};

#[derive(Clone, Debug)]
pub(crate) enum Device {
    Thermal(ThermalBreaker),
    Elcc(Elcc),
}

#[derive(Clone, Debug)]
pub(crate) struct CircuitProt {
    pub(crate) id: String,
    pub(crate) load: String,
    pub(crate) load_index: usize,
    pub(crate) base_w: f64,
    pub(crate) device: Device,
    pub(crate) closed: bool,
}

pub(crate) fn build(net: &crate::net::ElectricalNet, pdu: &super::pdu_link::PduLink) -> Vec<CircuitProt> {
    net.breaker_circuits()
        .iter()
        .filter(|c| !c.sspc || pdu.channel(&c.id).is_none())
        .map(|c| CircuitProt {
            id: c.id.clone(),
            load: c.load.clone(),
            load_index: c.load_index,
            base_w: net.load_demand_w(&c.load).unwrap_or(0.0),
            device: if c.kind == "ELCF" {
                Device::Elcc(Elcc::new(c.rating_a))
            } else {
                Device::Thermal(ThermalBreaker::new(c.rating_a))
            },
            closed: true,
        })
        .collect()
}

impl Electrical {
    pub(crate) fn step_circuit_protection(&mut self, dt_us: u64) {
        let Some(snap) = self.plant.snapshot() else {
            return;
        };
        let ambient = self.inputs.ambient_c;
        let mut changes: Vec<(String, f64)> = Vec::new();
        for c in self.circuit_prot.iter_mut() {
            let current = snap.loads.get(c.load_index).map(|l| l.current_a).unwrap_or(0.0);
            let line_v = snap.loads.get(c.load_index).map(|l| l.voltage_v).unwrap_or(0.0);
            let closed = match &mut c.device {
                Device::Thermal(t) => {
                    t.step(&BreakerInput {
                        dt_us,
                        current_a: current,
                        ambient_c: ambient,
                        pull: false,
                        push: false,
                    })
                    .closed
                }
                Device::Elcc(e) => {
                    e.step(&ElccInput {
                        dt_us,
                        close_cmd: true,
                        current_a: current,
                        line_v,
                        ambient_c: ambient,
                        elcu_powered: true,
                        pull: false,
                        push: false,
                    })
                    .closed
                }
            };
            if closed != c.closed {
                c.closed = closed;
                changes.push((c.load.clone(), if closed { c.base_w } else { 0.0 }));
            }
        }
        for (name, w) in changes {
            self.plant.set_load(&name, w);
        }
    }

    pub fn circuit_protection_open(&self) -> Vec<String> {
        self.circuit_prot
            .iter()
            .filter(|c| !c.closed)
            .map(|c| c.id.clone())
            .collect()
    }
}
