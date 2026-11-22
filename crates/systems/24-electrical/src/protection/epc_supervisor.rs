pub const EPC_CHATTER_CHANGES: usize = 4;
pub const EPC_CHATTER_WINDOW_US: u64 = 5_000_000;
pub const FWD_OFF_CURRENT_A: f64 = 10.0;
pub const FWD_OFF_US: u64 = 50_000;
pub const FWD_OFF_VOLTAGE_RMS: f64 = 20.0;
pub const AFT_FAIL_CLOSE_CURRENT_A: f64 = 10.0;
pub const AFT_FAIL_CLOSE_US: u64 = 150_000;

#[derive(Clone, Debug, Default, PartialEq)]
pub struct EpcSupervisor {
    changes: Vec<u64>,
    last_aux: bool,
    t_off: u64,
    t_close: u64,
    t_dead: u64,
    pub fwd_off_cmd: bool,
    pub chatter_lockout: bool,
    pub failed_open_check: bool,
    pub aft_fail_to_close: bool,
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct EpcSense {
    pub now_us: u64,
    pub dt_us: u64,
    pub plugged: bool,
    pub commanded: bool,
    pub aux_closed: bool,
    pub current_a: f64,
    pub voltage_v: f64,
    pub aft: bool,
}

impl EpcSupervisor {
    pub fn step(&mut self, s: &EpcSense) {
        if !s.plugged {
            *self = EpcSupervisor::default();
            return;
        }
        if s.aux_closed != self.last_aux {
            self.changes.push(s.now_us);
            self.last_aux = s.aux_closed;
        }
        let horizon = s.now_us.saturating_sub(EPC_CHATTER_WINDOW_US);
        self.changes
            .retain(|t| *t > horizon || s.now_us < EPC_CHATTER_WINDOW_US);
        if self.changes.len() > EPC_CHATTER_CHANGES {
            self.chatter_lockout = true;
        }
        if !s.commanded && s.current_a >= FWD_OFF_CURRENT_A {
            self.t_off = self.t_off.saturating_add(s.dt_us);
            if self.t_off >= FWD_OFF_US && !s.aft {
                self.failed_open_check = true;
            }
        } else {
            self.t_off = 0;
        }
        if !s.aft && s.commanded && s.voltage_v < FWD_OFF_VOLTAGE_RMS && s.current_a < FWD_OFF_CURRENT_A {
            self.t_dead = self.t_dead.saturating_add(s.dt_us);
            self.fwd_off_cmd = self.t_dead >= FWD_OFF_US;
        } else {
            self.t_dead = 0;
            self.fwd_off_cmd = false;
        }
        if s.aft && s.commanded && s.current_a < AFT_FAIL_CLOSE_CURRENT_A {
            self.t_close = self.t_close.saturating_add(s.dt_us);
            if self.t_close >= AFT_FAIL_CLOSE_US {
                self.aft_fail_to_close = true;
            }
        } else {
            self.t_close = 0;
            if s.aft && s.commanded {
                self.aft_fail_to_close = false;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sense(now: u64, aux: bool) -> EpcSense {
        EpcSense {
            now_us: now,
            dt_us: 10_000,
            plugged: true,
            commanded: aux,
            aux_closed: aux,
            current_a: 50.0,
            voltage_v: 115.0,
            aft: false,
        }
    }

    #[test]
    fn fwd_dead_receptacle_commands_epc_off_after_50ms() {
        let mut s = EpcSupervisor::default();
        let mut x = sense(0, true);
        x.voltage_v = 5.0;
        x.current_a = 2.0;
        for k in 1..=4 {
            x.now_us = k * 10_000;
            s.step(&x);
        }
        assert!(!s.fwd_off_cmd);
        x.now_us = 50_000;
        s.step(&x);
        assert!(s.fwd_off_cmd);
        x.voltage_v = 115.0;
        s.step(&x);
        assert!(!s.fwd_off_cmd);
    }

    #[test]
    fn more_than_four_state_changes_in_5s_lock_out_and_unplug_resets() {
        let mut s = EpcSupervisor::default();
        for k in 0..4u64 {
            s.step(&sense(10_000_000 + k * 1_000_000, k % 2 == 0));
        }
        assert!(!s.chatter_lockout);
        s.step(&sense(14_500_000, true));
        assert!(s.chatter_lockout);
        let mut u = sense(15_000_000, false);
        u.plugged = false;
        s.step(&u);
        assert!(!s.chatter_lockout);
        let mut s = EpcSupervisor::default();
        for k in 0..6u64 {
            s.step(&sense(10_000_000 + k * 2_000_000, k % 2 == 0));
        }
        assert!(!s.chatter_lockout, "slow cycling stays below the window");
    }

    #[test]
    fn forward_epc_still_carrying_current_50ms_after_open_command_is_flagged() {
        let mut s = EpcSupervisor::default();
        let mut i = sense(1_000_000, true);
        i.commanded = false;
        for k in 0..4 {
            i.now_us += 10_000 * k;
            s.step(&i);
        }
        assert!(!s.failed_open_check);
        s.step(&i);
        assert!(s.failed_open_check);
    }

    #[test]
    fn aft_epc_with_no_current_150ms_after_close_is_flagged() {
        let mut s = EpcSupervisor::default();
        let mut i = sense(1_000_000, true);
        i.aft = true;
        i.current_a = 2.0;
        for _ in 0..14 {
            s.step(&i);
        }
        assert!(!s.aft_fail_to_close);
        s.step(&i);
        assert!(s.aft_fail_to_close);
        i.current_a = 40.0;
        s.step(&i);
        assert!(!s.aft_fail_to_close);
    }
}
