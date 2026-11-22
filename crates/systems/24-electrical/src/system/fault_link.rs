use super::{Electrical, MACHINES, SignalValue};
use crate::faults::{self, FaultBook};
use std::collections::{BTreeMap, BTreeSet};

pub(crate) fn norm(s: &str) -> String {
    s.replace(' ', "_").replace(':', ".")
}

#[derive(Clone, Debug, Default)]
pub(crate) struct FaultLink {
    pub(crate) book: FaultBook,
    active: Vec<(String, String, f64)>,
    had: bool,
    prev_gen: [bool; 4],
    prev_asg: [bool; 2],
    prev_bpcu: [bool; 2],
    jam: [bool; 4],
    overflow: [bool; 6],
    regulated_base: [f64; 6],
    drift_applied: [f64; 6],
    sspc_forced: BTreeMap<String, bool>,
    cell_base: BTreeMap<(usize, usize), (f64, f64)>,
    pub(crate) tripped: BTreeSet<String>,
    pub(crate) applied: super::causal::NetState,
    pub(crate) heater_on: bool,
    pub(crate) panel_hold: BTreeMap<String, bool>,
    pub(crate) exciter_scale: [f64; 6],
    pub(crate) ct_offset_a: [f64; 6],
    pub(crate) field_driver_short: [bool; 6],
    pub(crate) load_base: BTreeMap<String, f64>,
}

impl FaultLink {
    pub(crate) fn new() -> FaultLink {
        FaultLink {
            regulated_base: [f64::NAN; 6],
            exciter_scale: [1.0; 6],
            ..FaultLink::default()
        }
    }

    pub(crate) fn active_parts(&self) -> Vec<&str> {
        self.active.iter().map(|(p, _, _)| p.as_str()).collect()
    }

    pub(crate) fn jam(&self, i: usize) -> bool {
        self.jam.get(i).copied().unwrap_or(false)
    }
}

const SRD_FIELD_LOSS: f64 = 0.75;

fn machine_index(id: &str) -> Option<usize> {
    MACHINES.iter().position(|m| *m == id)
}

fn gcu_index(id: &str) -> Option<usize> {
    machine_index(id.strip_prefix("GCU_")?)
}

fn cell_key(id: &str) -> Option<(usize, usize)> {
    let (pack, cell) = id.split_once(".c")?;
    let b = match pack {
        "BATT_MAIN" => 0,
        "BATT_APU" => 1,
        _ => return None,
    };
    Some((b, cell.parse().ok()?))
}

const PREFIX_TYPES: [(&str, &str); 22] = [
    ("GCU_", faults::GCU),
    ("ATRU_", faults::ATRU),
    ("ATU_", faults::ATU),
    ("TRU_", faults::TRU),
    ("CMSC_", faults::CMSC),
    ("ELCC_", faults::ELCC),
    ("ELCU_", faults::ELCU),
    ("TCB_", faults::TCB),
    ("RPDUC ", faults::RPDU_CONTROLLER),
    ("RPDU ", faults::RPDU_MODULE),
    ("W:", faults::WIRE_SEGMENT),
    ("CONN:", faults::CONNECTOR),
    ("GND:", faults::GROUND_STUD),
    ("SW:", faults::PANEL_SWITCH),
    ("LAMP:", faults::LAMP),
    ("EICASIN:", faults::EICAS_INPUT),
    ("OAV_", faults::OIL_AIR_VALVE),
    ("RATHTR", faults::RAT_HEATER),
    ("EXTRCPT_", faults::EXT_POWER_RECEPTACLE),
    ("PECS_", faults::PECS_PUMP),
    ("BPCU_", faults::BPCU),
    ("BMU_", faults::BMU),
];

impl Electrical {
    pub fn fault_book(&self) -> &FaultBook {
        &self.fault_link.book
    }

    pub fn part_type_of(&self, id: &str) -> Option<&'static str> {
        if self.gear.iter().any(|g| g.name == id) {
            return Some(faults::CONTACTOR);
        }
        match id {
            "VFSG_L1" | "VFSG_L2" | "VFSG_R1" | "VFSG_R2" => return Some(faults::VFSG),
            "ASG_L" | "ASG_R" => return Some(faults::ASG),
            "RAT" => return Some(faults::RAT_GEN),
            "BATT_MAIN" | "BATT_APU" => return Some(faults::BATTERY_PACK),
            "CHG_MAIN" | "CHG_APU" => return Some(faults::CHARGER),
            "DIODE_MAIN" | "DIODE_APU" => return Some(faults::DIODE_MODULE),
            "SPU" => return Some(faults::SPU),
            "SPDU_L" | "SPDU_R" => return Some(faults::SPDU_CARD),
            _ => {}
        }
        if cell_key(id).is_some() {
            return Some(faults::BATTERY_CELL);
        }
        for (prefix, t) in PREFIX_TYPES {
            if id.starts_with(prefix) {
                return Some(t);
            }
        }
        if self.plant.net().sspc_ids().iter().any(|s| s == id) {
            return Some(faults::SSPC);
        }
        None
    }

    pub fn arm_fault(&mut self, part: &str, mode: &str, magnitude: f64) -> Result<(), String> {
        let t = self
            .part_type_of(part)
            .ok_or_else(|| format!("unknown part {part}"))?;
        let now = self.now_us;
        self.fault_link
            .book
            .arm_immediate(part, t, mode, magnitude.clamp(0.0, 1.0), now)
            .map_err(|e| format!("{e:?}"))
    }

    pub fn clear_fault(&mut self, part: &str, mode: Option<&str>) -> Result<(), String> {
        let done = match mode {
            Some(m) => self.fault_link.book.clear(part, m),
            None => self.fault_link.book.clear_part(part),
        };
        if done {
            self.fault_link.tripped.remove(part);
            Ok(())
        } else {
            Err(format!("no such fault on {part}"))
        }
    }

    pub fn active_faults(&self) -> &[(String, String, f64)] {
        &self.fault_link.active
    }

    pub fn fault_tripped(&self, part: &str) -> bool {
        self.fault_link.tripped.contains(part)
    }

    pub(crate) fn fault_value(&self, name: &str) -> Option<SignalValue> {
        let rest = name.strip_prefix("elec.fault.")?;
        if rest == "count" {
            return Some(SignalValue::Num(self.fault_link.active.len() as f64));
        }
        if let Some(rest) = rest.strip_prefix("tripped.") {
            return Some(SignalValue::Bool(
                self.fault_link.tripped.iter().any(|p| norm(p) == rest),
            ));
        }
        for (part, mode, mag) in &self.fault_link.active {
            if format!("{}.{}", norm(part), mode) == rest {
                return Some(SignalValue::Num(*mag));
            }
        }
        Some(SignalValue::Num(0.0))
    }

    pub(crate) fn fault_disconnect_jam(&self, i: usize) -> bool {
        self.fault_link.jam(i)
    }

    pub(crate) fn stage_fault_pre(&mut self) {
        if self.fault_link.book.is_empty() && !self.fault_link.had {
            return;
        }
        let now = self.now_us;
        let active = self.fault_link.book.active(now);
        self.fault_link.had = !self.fault_link.book.is_empty();
        let mut gen_lost = [false; 4];
        let mut asg_lost = [false; 2];
        let mut exciter_scale = [1.0; 6];
        let mut ct_offset_a = [0.0; 6];
        let mut field_driver_short = [false; 6];
        let mut bpcu = [false; 2];
        let mut jam = [false; 4];
        let mut drift = [0.0; 6];
        let mut overflow = [false; 6];
        let mut parts: Vec<&str> = active.iter().map(|(p, _, _)| p.as_str()).collect();
        parts.dedup();
        for part in parts {
            let Some(t) = self.fault_link.book.part_type(part) else {
                continue;
            };
            match t {
                faults::VFSG => {
                    if let Some(m) = machine_index(part).filter(|m| *m < 4) {
                        let e = self.fault_link.book.machine(part, now);
                        gen_lost[m] = e.phase_open || e.pmg_scale < 0.5;
                        exciter_scale[m] = 1.0 - SRD_FIELD_LOSS * e.exciter_ripple.min(1.0);
                        jam[m] = e.disconnect_jam;
                    }
                }
                faults::ASG => {
                    if let Some(m) = machine_index(part).filter(|m| *m >= 4) {
                        let e = self.fault_link.book.machine(part, now);
                        asg_lost[m - 4] = e.phase_open || e.pmg_scale < 0.5;
                        exciter_scale[m] = 1.0 - SRD_FIELD_LOSS * e.exciter_ripple.min(1.0);
                    }
                }
                faults::GCU => {
                    if let Some(m) = gcu_index(part) {
                        let e = self.fault_link.book.gcu(part, now);
                        if e.cpu_failed {
                            if m < 4 {
                                gen_lost[m] = true;
                            } else {
                                asg_lost[m - 4] = true;
                            }
                        }
                        drift[m] = e.setpoint_offset_v;
                        ct_offset_a[m] = e.ct_offset_a;
                        field_driver_short[m] = e.field_driver_short;
                        overflow[m] = e.powered_time_preload_us > 0;
                        if overflow[m] && !self.fault_link.overflow[m] {
                            self.gcus[m].set_powered_time_us(e.powered_time_preload_us);
                        }
                    }
                }
                faults::BPCU => {
                    let k = usize::from(part == "BPCU_R");
                    let down = ["cpu-fail", "ttpc-loss", "pin-error"]
                        .iter()
                        .any(|m| self.fault_link.book.magnitude(part, m, now) >= faults::BINARY_THRESHOLD);
                    if down {
                        bpcu[k] = true;
                    }
                }
                _ => {}
            }
        }
        self.fault_link.exciter_scale = exciter_scale;
        self.fault_link.ct_offset_a = ct_offset_a;
        self.fault_link.field_driver_short = field_driver_short;
        for k in 0..4 {
            if gen_lost[k] != self.fault_link.prev_gen[k] {
                self.faults.gen_loss[k] = gen_lost[k];
                self.fault_link.prev_gen[k] = gen_lost[k];
            }
        }
        for k in 0..2 {
            if asg_lost[k] != self.fault_link.prev_asg[k] {
                self.faults.asg_loss[k] = asg_lost[k];
                self.fault_link.prev_asg[k] = asg_lost[k];
            }
            if bpcu[k] != self.fault_link.prev_bpcu[k] {
                self.faults.bpcu_fail[k] = bpcu[k];
                self.fault_link.prev_bpcu[k] = bpcu[k];
            }
        }
        for m in 0..6 {
            if self.fault_link.regulated_base[m].is_nan() {
                self.fault_link.regulated_base[m] = self.gcus[m].params.regulated_v;
            }
            if (drift[m] - self.fault_link.drift_applied[m]).abs() > 1e-9 {
                self.gcus[m].params.regulated_v = self.fault_link.regulated_base[m] + drift[m];
                self.fault_link.drift_applied[m] = drift[m];
            }
        }
        self.fault_link.jam = jam;
        self.fault_link.overflow = overflow;
        self.fault_link.active = active;
    }

    pub(crate) fn force_gear(&mut self, name: &str, open: bool, weld: bool, aux_stuck: bool) {
        let Some(g) = self.gear.iter_mut().find(|g| g.name == name) else {
            return;
        };
        if weld {
            if !g.unit.welded() {
                g.unit.force_weld();
            }
            g.out.contacts_closed = true;
            g.out.armature_closed = true;
            g.out.welded = true;
        } else if open {
            g.out.contacts_closed = false;
            g.out.armature_closed = false;
        }
        if aux_stuck {
            g.out.aux_no = !g.out.contacts_closed;
            g.out.aux_nc = !g.out.aux_no;
        } else if open || weld {
            g.out.aux_no = g.out.contacts_closed;
            g.out.aux_nc = !g.out.aux_no;
        }
        let closed = g.out.contacts_closed;
        self.plant.set_contactor(name, closed);
    }

    pub(crate) fn force_sspc(&mut self, id: &str, closed: bool) {
        self.fault_link.sspc_forced.insert(id.to_string(), closed);
        self.plant.net_mut().set_sspc(id, closed);
        self.pdu.hold_net(id, true);
    }

    pub(crate) fn stage_fault_post(&mut self, dt_us: u64) {
        self.causal_heater();
        if self.fault_link.active.is_empty()
            && self.fault_link.sspc_forced.is_empty()
            && self.fault_link.cell_base.is_empty()
            && self.fault_link.applied.is_empty()
            && !self.tru_oc.iter().any(|t| t.tripped())
            && !self.atru_prot.iter().any(|p| p.shut_down())
        {
            return;
        }
        let now = self.now_us;
        let mut parts: Vec<String> = self.fault_link.active.iter().map(|(p, _, _)| p.clone()).collect();
        parts.dedup();
        let mut sspc_live: BTreeSet<String> = BTreeSet::new();
        let mut cell_live: BTreeSet<(usize, usize)> = BTreeSet::new();
        for part in &parts {
            let Some(t) = self.fault_link.book.part_type(part) else {
                continue;
            };
            match t {
                faults::CONTACTOR => {
                    let e = self.fault_link.book.contactor(part, now);
                    if e.welded || e.stuck_open || e.coil_open || e.aux_stuck {
                        self.force_gear(part, e.stuck_open || e.coil_open, e.welded, e.aux_stuck);
                    }
                }
                faults::SSPC => {
                    let e = self.fault_link.book.sspc(part, now);
                    if e.force_trip || e.arc_false_trip {
                        self.fault_link.tripped.insert(part.clone());
                    }
                    if e.stuck_open || e.force_trip || e.arc_false_trip {
                        self.force_sspc(part, false);
                        sspc_live.insert(part.clone());
                    } else if e.stuck_closed {
                        self.force_sspc(part, true);
                        sspc_live.insert(part.clone());
                    }
                }
                faults::WIRE_SEGMENT => {
                    let target = part.strip_prefix("W:").unwrap_or(part).to_string();
                    let e = self.fault_link.book.wire(part, now);
                    if e.open || e.arcing {
                        if e.arcing {
                            self.fault_link.tripped.insert(part.clone());
                        }
                        if self.gear.iter().any(|g| g.name == target) {
                            self.force_gear(&target, true, false, false);
                        } else if self.plant.net().sspc_ids().contains(&target) {
                            self.force_sspc(&target, false);
                            sspc_live.insert(target);
                        }
                    }
                }
                faults::BATTERY_CELL => {
                    if let Some((b, c)) = cell_key(part) {
                        cell_live.insert((b, c));
                        self.apply_cell_fault(part, b, c, dt_us);
                    }
                }
                _ => {}
            }
        }
        self.causal_post(&mut sspc_live);
        let stale: Vec<String> = self
            .fault_link
            .sspc_forced
            .keys()
            .filter(|k| !sspc_live.contains(*k))
            .cloned()
            .collect();
        for id in stale {
            let default_closed = !id.starts_with("RPDU");
            self.fault_link.sspc_forced.remove(&id);
            self.plant.net_mut().set_sspc(&id, default_closed);
            self.pdu.hold_net(&id, false);
        }
        let stale_cells: Vec<(usize, usize)> = self
            .fault_link
            .cell_base
            .keys()
            .filter(|k| !cell_live.contains(*k))
            .copied()
            .collect();
        for (b, c) in stale_cells {
            if let Some((cap, r)) = self.fault_link.cell_base.remove(&(b, c))
                && let Some(pack) = self.plant.net_mut().battery_pack_mut(b)
                && c < pack.params.cells
            {
                pack.cells[c].cap_scale = cap;
                pack.cells[c].r_scale = r;
            }
        }
    }

    fn apply_cell_fault(&mut self, part: &str, b: usize, c: usize, dt_us: u64) {
        let e = self.fault_link.book.battery_cell(part, self.now_us);
        let Some(pack) = self.plant.net_mut().battery_pack_mut(b) else {
            return;
        };
        if c >= pack.params.cells {
            return;
        }
        let base = *self
            .fault_link
            .cell_base
            .entry((b, c))
            .or_insert((pack.cells[c].cap_scale, pack.cells[c].r_scale));
        pack.cells[c].cap_scale = base.0 * e.capacity_scale;
        pack.cells[c].r_scale = base.1 * e.r0_mult;
        if let Some(r) = e.internal_short_ohm {
            let amps = 3.6 / r;
            let cap_as = pack.params.capacity_ah * 3600.0 * pack.cells[c].cap_scale;
            let drain = amps * dt_us as f64 * 1.0e-6 / cap_as;
            pack.cells[c].soc = (pack.cells[c].soc - drain).clamp(0.0, 1.0);
        }
    }
}
