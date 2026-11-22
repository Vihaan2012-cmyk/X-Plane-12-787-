use super::{Electrical, MACHINES};
use crate::sources::gcu::GcuFault;

pub(crate) const DRIVE_DISC_CONDITION_US: u64 = 50_000;
pub(crate) const DRIVE_SWITCH_MAX_PRESS_US: u64 = 3_000_000;
pub(crate) const DRIVE_SWITCH_WAIT_US: u64 = 60_000_000;
pub(crate) const LOP_AUTO_DISCONNECT_US: u64 = 500_000;
pub(crate) const APPROX_CREW_PRESS_US: u64 = 1_000_000;

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub(crate) struct DriveSolenoid {
    pub energized_us: u64,
    pub heat_us: u64,
    pub damaged: bool,
    pub lop_us: u64,
    pub release_at_us: Option<u64>,
}

impl Electrical {
    pub(crate) fn press_drive_disconnect_for(&mut self, k: usize, hold_us: u64) {
        self.panel.drive_disconnect[k].set_guard(true);
        self.panel.press_drive_disconnect(k);
        self.drive_sol[k].release_at_us = Some(self.now_us + hold_us);
    }

    pub(crate) fn release_due_drive_switches(&mut self) {
        for k in 0..4 {
            if self.drive_sol[k].release_at_us.is_some_and(|t| self.now_us >= t) {
                self.panel.release_drive_disconnect(k);
                self.drive_sol[k].release_at_us = None;
            }
        }
    }

    pub(crate) fn step_drive_disconnects(&mut self, dt_us: u64) {
        for k in 0..4 {
            let request = self.panel_out.drive_disconnect_request[k];
            let jam = self.fault_disconnect_jam(k);
            let uf_trip = self.gcu_out[k].trip_mask & GcuFault::UnderFreq.mask() != 0;
            let lop = self.crew_in.vfsg_lop[k];
            let thermal = self.plant.net().machine_disconnected(MACHINES[k]);
            let s = &mut self.drive_sol[k];
            let energized = request && !s.damaged;
            if energized {
                s.energized_us = s.energized_us.saturating_add(dt_us);
                s.heat_us = s.heat_us.saturating_add(dt_us);
                if s.heat_us > DRIVE_SWITCH_MAX_PRESS_US {
                    s.damaged = true;
                }
            } else {
                s.energized_us = 0;
                let cool = dt_us.saturating_mul(DRIVE_SWITCH_MAX_PRESS_US) / DRIVE_SWITCH_WAIT_US;
                s.heat_us = s.heat_us.saturating_sub(cool);
            }
            s.lop_us = if lop { s.lop_us.saturating_add(dt_us) } else { 0 };
            let commanded = energized && s.energized_us >= DRIVE_DISC_CONDITION_US && !jam;
            let auto = s.lop_us >= LOP_AUTO_DISCONNECT_US && uf_trip && !jam;
            if commanded || auto || thermal {
                self.drive_disconnected[k] = true;
            }
        }
    }
}
