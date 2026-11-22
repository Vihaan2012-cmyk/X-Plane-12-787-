mod catalog;
mod effects;
#[cfg(test)]
mod tests;

pub use catalog::*;
pub use effects::*;

use failures::{FailureError, FailureSet, Onset, Registry, seed_for};
use std::collections::BTreeMap;

#[derive(Clone, Debug)]
struct Instance {
    part_type: String,
    set: FailureSet,
}

#[derive(Clone, Debug)]
pub struct FaultBook {
    registry: Registry,
    parts: BTreeMap<String, Instance>,
}

impl Default for FaultBook {
    fn default() -> Self {
        Self::new()
    }
}

impl FaultBook {
    pub fn new() -> Self {
        Self {
            registry: catalog(),
            parts: BTreeMap::new(),
        }
    }

    pub fn registry(&self) -> &Registry {
        &self.registry
    }

    pub fn arm(
        &mut self,
        part: &str,
        part_type: &str,
        mode: &str,
        magnitude: f64,
        onset: Onset,
        now_us: u64,
    ) -> Result<(), FailureError> {
        if !self.registry.supports(part_type, mode) {
            return Err(FailureError::UnknownMode {
                part_type: part_type.to_string(),
                mode: mode.to_string(),
            });
        }
        let inst = self.parts.entry(part.to_string()).or_insert_with(|| Instance {
            part_type: part_type.to_string(),
            set: FailureSet::new(),
        });
        if inst.part_type != part_type {
            return Err(FailureError::UnknownMode {
                part_type: inst.part_type.clone(),
                mode: mode.to_string(),
            });
        }
        inst.set.arm(mode, magnitude, onset, now_us);
        Ok(())
    }

    pub fn arm_immediate(
        &mut self,
        part: &str,
        part_type: &str,
        mode: &str,
        magnitude: f64,
        now_us: u64,
    ) -> Result<(), FailureError> {
        self.arm(part, part_type, mode, magnitude, Onset::Immediate, now_us)
    }

    pub fn arm_intermittent(
        &mut self,
        part: &str,
        part_type: &str,
        mode: &str,
        magnitude: f64,
        pattern: (u64, f64),
        now_us: u64,
    ) -> Result<(), FailureError> {
        let (period_us, duty) = pattern;
        let onset = Onset::Intermittent {
            seed: seed_for(part, mode),
            period_us,
            duty,
        };
        self.arm(part, part_type, mode, magnitude, onset, now_us)
    }

    pub fn clear(&mut self, part: &str, mode: &str) -> bool {
        let Some(inst) = self.parts.get_mut(part) else {
            return false;
        };
        let removed = inst.set.clear(mode);
        if inst.set.is_empty() {
            self.parts.remove(part);
        }
        removed
    }

    pub fn clear_part(&mut self, part: &str) -> bool {
        self.parts.remove(part).is_some()
    }

    pub fn clear_all(&mut self) {
        self.parts.clear();
    }

    pub fn trigger(&mut self, trigger: &str, now_us: u64) -> usize {
        self.parts
            .values_mut()
            .map(|i| i.set.trigger(trigger, now_us))
            .sum()
    }

    pub fn part_type(&self, part: &str) -> Option<&str> {
        self.parts.get(part).map(|i| i.part_type.as_str())
    }

    pub fn magnitude(&self, part: &str, mode: &str, now_us: u64) -> f64 {
        self.parts
            .get(part)
            .map_or(0.0, |i| i.set.magnitude(mode, now_us))
    }

    pub fn snapshot(&self, part: &str, now_us: u64) -> Magnitudes {
        match self.parts.get(part) {
            Some(i) => Magnitudes::new(
                i.set
                    .armed_modes()
                    .into_iter()
                    .map(|m| (m.to_string(), i.set.magnitude(m, now_us)))
                    .collect(),
            ),
            None => Magnitudes::default(),
        }
    }

    pub fn active(&self, now_us: u64) -> Vec<(String, String, f64)> {
        let mut out = Vec::new();
        for (part, inst) in &self.parts {
            for mode in inst.set.active_modes(now_us) {
                out.push((part.clone(), mode.to_string(), inst.set.magnitude(mode, now_us)));
            }
        }
        out
    }

    pub fn active_observables(&self, part: &str, now_us: u64) -> Vec<String> {
        let Some(inst) = self.parts.get(part) else {
            return Vec::new();
        };
        let mut out: Vec<String> = Vec::new();
        for mode in inst.set.active_modes(now_us) {
            if let Some(m) = self.registry.mode(&inst.part_type, mode) {
                for o in &m.observables {
                    if !out.contains(o) {
                        out.push(o.clone());
                    }
                }
            }
        }
        out
    }

    pub fn len(&self) -> usize {
        self.parts.len()
    }

    pub fn is_empty(&self) -> bool {
        self.parts.is_empty()
    }

    pub fn contactor(&self, part: &str, now_us: u64) -> ContactorEffect {
        contactor_effect(&self.snapshot(part, now_us))
    }

    pub fn sspc(&self, part: &str, now_us: u64) -> SspcEffect {
        sspc_effect(&self.snapshot(part, now_us))
    }

    pub fn wire(&self, part: &str, now_us: u64) -> WireEffect {
        wire_effect(&self.snapshot(part, now_us))
    }

    pub fn connector(&self, part: &str, now_us: u64) -> ConnectorEffect {
        connector_effect(&self.snapshot(part, now_us))
    }

    pub fn battery_cell(&self, part: &str, now_us: u64) -> CellEffect {
        battery_cell_effect(&self.snapshot(part, now_us))
    }

    pub fn machine(&self, part: &str, now_us: u64) -> MachineEffect {
        machine_effect(&self.snapshot(part, now_us))
    }

    pub fn gcu(&self, part: &str, now_us: u64) -> GcuEffect {
        gcu_effect(&self.snapshot(part, now_us))
    }

    pub fn converter(&self, part: &str, now_us: u64) -> ConverterEffect {
        converter_effect(&self.snapshot(part, now_us))
    }

    pub fn indication(&self, part: &str, now_us: u64) -> IndicationEffect {
        indication_effect(&self.snapshot(part, now_us))
    }
}
