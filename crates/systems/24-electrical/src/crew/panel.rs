use super::inputs::CrewInputs;
use super::switch::{Lamp, LampColor, Switch, SwitchKind};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum ApuSelector {
    #[default]
    Off,
    On,
    Start,
}

#[derive(Clone, Debug)]
pub struct OverheadPanel {
    pub battery: Switch,
    pub apu_gen: [Switch; 2],
    pub gen_ctrl: [Switch; 4],
    pub apu_selector: ApuSelector,
    pub ext_pwr_fwd: [Switch; 2],
    pub ext_pwr_aft: Switch,
    pub drive_disconnect: [Switch; 4],
    pub ground_service: Switch,
    pub cabin_util: Switch,
    pub ife_seats: Switch,
    pub towing_power: Switch,
    pub towing_test: Switch,
    ground_service_lamp: bool,
    cycled: [bool; 4],
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct PanelOutputs {
    pub battery_on: bool,
    pub apu_gen_on: [bool; 2],
    pub gen_ctrl_on: [bool; 4],
    pub gen_ctrl_cycled: [bool; 4],
    pub apu_selector: ApuSelector,
    pub ext_fwd_pulse: [bool; 2],
    pub ext_aft_pulse: bool,
    pub drive_disconnect_request: [bool; 4],
    pub ground_service_on: bool,
    pub cabin_util_on: bool,
    pub ife_on: bool,
    pub towing_power_on: bool,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct PanelLamps {
    pub battery_off: Lamp,
    pub apu_gen_off: [Lamp; 2],
    pub apu_gen_avail: [Lamp; 2],
    pub apu_gen_on: [Lamp; 2],
    pub gen_off: [Lamp; 4],
    pub drive: [Lamp; 4],
    pub ext_fwd_avail: [Lamp; 2],
    pub ext_fwd_on: [Lamp; 2],
    pub ext_aft_avail: Lamp,
    pub ext_aft_on: Lamp,
    pub cabin_util_off: Lamp,
    pub ife_off: Lamp,
    pub ground_service_on: Lamp,
    pub towing_on_bat: Lamp,
    pub towing_high: Lamp,
    pub towing_medium: Lamp,
    pub towing_low: Lamp,
}

impl Default for OverheadPanel {
    fn default() -> Self {
        Self::new()
    }
}

impl OverheadPanel {
    pub fn new() -> Self {
        Self {
            battery: Switch::new(SwitchKind::Alternate),
            apu_gen: [Switch::new(SwitchKind::Alternate); 2],
            gen_ctrl: [Switch::new(SwitchKind::Alternate); 4],
            apu_selector: ApuSelector::Off,
            ext_pwr_fwd: [Switch::new(SwitchKind::Momentary); 2],
            ext_pwr_aft: Switch::new(SwitchKind::Momentary),
            drive_disconnect: [Switch::new(SwitchKind::GuardedMomentary); 4],
            ground_service: Switch::new(SwitchKind::Momentary),
            towing_power: Switch::new(SwitchKind::Alternate),
            towing_test: Switch::new(SwitchKind::Momentary),
            ground_service_lamp: false,
            cabin_util: Switch::new(SwitchKind::Alternate),
            ife_seats: Switch::new(SwitchKind::Alternate),
            cycled: [false; 4],
        }
    }

    pub fn set_normal(&mut self) {
        self.battery.set_latched(true);
        for s in &mut self.apu_gen {
            s.set_latched(true);
        }
        for s in &mut self.gen_ctrl {
            s.set_latched(true);
        }
        self.cabin_util.set_latched(true);
        self.ife_seats.set_latched(true);
    }

    pub fn set_apu_selector(&mut self, pos: ApuSelector) {
        self.apu_selector = pos;
    }

    pub fn release_apu_selector(&mut self) {
        if self.apu_selector == ApuSelector::Start {
            self.apu_selector = ApuSelector::On;
        }
    }

    pub fn press_gen_ctrl(&mut self, i: usize) {
        let was_on = self.gen_ctrl[i].is_on();
        self.gen_ctrl[i].press();
        if !was_on {
            self.cycled[i] = true;
        }
    }

    pub fn press_drive_disconnect(&mut self, i: usize) -> bool {
        self.drive_disconnect[i].press()
    }

    pub fn release_drive_disconnect(&mut self, i: usize) {
        self.drive_disconnect[i].release();
    }

    pub fn drive_lamp_cmd(i: usize, inp: &CrewInputs) -> bool {
        inp.vfsg_lop[i] || inp.pmg_lost[i]
    }

    pub fn update(&mut self, inp: &CrewInputs) -> (PanelOutputs, PanelLamps) {
        let mut out = PanelOutputs {
            battery_on: self.battery.is_on(),
            apu_gen_on: [self.apu_gen[0].is_on(), self.apu_gen[1].is_on()],
            ground_service_on: false,
            towing_power_on: self.towing_power.is_on(),
            cabin_util_on: self.cabin_util.is_on(),
            ife_on: self.ife_seats.is_on(),
            apu_selector: self.apu_selector,
            ext_fwd_pulse: [self.ext_pwr_fwd[0].take_pulse(), self.ext_pwr_fwd[1].take_pulse()],
            ext_aft_pulse: self.ext_pwr_aft.take_pulse(),
            ..PanelOutputs::default()
        };
        if self.ground_service.take_pulse() {
            self.ground_service_lamp = !self.ground_service_lamp;
        }
        let gen_commanded = inp.gcb_closed.iter().any(|g| *g)
            || inp.apb_cmd_closed.iter().any(|b| *b)
            || inp.apb_closed.iter().any(|b| *b);
        let ext_on = inp.ext_fwd_epc_closed.iter().any(|b| *b) && !inp.ground_handling_mode;
        if gen_commanded || ext_on {
            self.ground_service_lamp = false;
        }
        out.ground_service_on = self.ground_service_lamp;
        for i in 0..4 {
            out.gen_ctrl_on[i] = self.gen_ctrl[i].is_on();
            out.gen_ctrl_cycled[i] = self.cycled[i];
            self.cycled[i] = false;
            self.drive_disconnect[i].take_pulse();
            if self.drive_disconnect[i].pressed {
                let engine = i / 2;
                let lamp = Self::drive_lamp_cmd(i, inp);
                let allowed = if inp.on_ground {
                    inp.engine_running[engine] || inp.fuel_run[engine]
                } else {
                    lamp
                };
                out.drive_disconnect_request[i] = allowed && !inp.drive_disconnected[i];
            }
        }

        let t = inp.lamp_test;
        let d = inp.dim;
        let mut gen_off = [Lamp::drive(LampColor::Amber, false, t, d); 4];
        let mut drive = gen_off;
        for i in 0..4 {
            gen_off[i] = Lamp::drive(LampColor::Amber, !inp.gcb_closed[i], t, d);
            drive[i] = Lamp::drive(LampColor::Amber, Self::drive_lamp_cmd(i, inp), t, d);
        }
        let mut apu_off = [Lamp::drive(LampColor::Amber, false, t, d); 2];
        let mut apu_avail = [Lamp::drive(LampColor::White, false, t, d); 2];
        let mut apu_on = [Lamp::drive(LampColor::White, false, t, d); 2];
        let mut fwd_avail = [Lamp::drive(LampColor::White, false, t, d); 2];
        let mut fwd_on = fwd_avail;
        for s in 0..2 {
            let off_cmd = !out.apu_gen_on[s] || (inp.apu_running && !inp.apb_closed[s]);
            apu_off[s] = Lamp::drive(LampColor::Amber, off_cmd, t, d);
            apu_avail[s] = Lamp::drive(
                LampColor::White,
                inp.apu_gen_available[s] && !inp.apb_closed[s],
                t,
                d,
            );
            apu_on[s] = Lamp::drive(LampColor::White, inp.apb_closed[s], t, d);
            fwd_avail[s] = Lamp::drive(
                LampColor::White,
                inp.ext_fwd_inserted[s] && inp.ext_fwd_quality_ok[s] && !inp.ext_fwd_epc_closed[s],
                t,
                d,
            );
            fwd_on[s] = Lamp::drive(LampColor::White, inp.ext_fwd_epc_closed[s], t, d);
        }
        let aft_avail = inp.ext_aft_inserted && inp.ext_aft_quality_ok && !inp.ext_aft_epc_closed;
        let lamps = PanelLamps {
            battery_off: Lamp::drive(LampColor::Amber, !out.battery_on, t, d),
            apu_gen_off: apu_off,
            apu_gen_avail: apu_avail,
            apu_gen_on: apu_on,
            gen_off,
            drive,
            ext_fwd_avail: fwd_avail,
            ext_fwd_on: fwd_on,
            ext_aft_avail: Lamp::drive(LampColor::White, aft_avail, t, d),
            ext_aft_on: Lamp::drive(LampColor::White, inp.ext_aft_epc_closed, t, d),
            cabin_util_off: Lamp::drive(LampColor::Amber, !out.cabin_util_on, t, d),
            ife_off: Lamp::drive(LampColor::Amber, !out.ife_on, t, d),
            ground_service_on: Lamp::drive(LampColor::White, self.ground_service_lamp, t, d),
            towing_on_bat: Lamp::drive(LampColor::White, out.towing_power_on, t, d),
            towing_high: Lamp::drive(
                LampColor::White,
                self.towing_test.pressed && inp.main_batt_volts > 30.1,
                t,
                d,
            ),
            towing_medium: Lamp::drive(
                LampColor::White,
                self.towing_test.pressed && inp.main_batt_volts <= 30.1 && inp.main_batt_volts >= 29.7,
                t,
                d,
            ),
            towing_low: Lamp::drive(
                LampColor::White,
                self.towing_test.pressed && inp.main_batt_volts < 29.7,
                t,
                d,
            ),
        };
        (out, lamps)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn base() -> (OverheadPanel, CrewInputs) {
        let mut p = OverheadPanel::new();
        p.set_normal();
        let inp = CrewInputs {
            gcb_closed: [true; 4],
            apb_closed: [false; 2],
            ..CrewInputs::default()
        };
        (p, inp)
    }

    #[test]
    fn battery_off_lamp_follows_switch() {
        let (mut p, inp) = base();
        let (_, l) = p.update(&inp);
        assert!(!l.battery_off.lit);
        p.battery.press();
        let (o, l) = p.update(&inp);
        assert!(!o.battery_on);
        assert!(l.battery_off.lit);
    }

    #[test]
    fn gen_off_lamp_follows_gcb() {
        let (mut p, mut inp) = base();
        inp.gcb_closed[2] = false;
        let (_, l) = p.update(&inp);
        assert!(l.gen_off[2].lit);
        assert!(!l.gen_off[0].lit);
    }

    #[test]
    fn lamp_test_lights_all() {
        let (mut p, mut inp) = base();
        inp.lamp_test = true;
        inp.dim = true;
        let (_, l) = p.update(&inp);
        assert!(l.battery_off.lit && l.drive.iter().all(|x| x.lit && x.brightness == 1.0));
        assert!(l.ext_fwd_avail[0].lit && l.ife_off.lit && l.cabin_util_off.lit);
        assert!(l.gen_off.iter().all(|x| x.lit));
    }

    #[test]
    fn dim_reduces_brightness() {
        let (mut p, mut inp) = base();
        inp.gcb_closed[0] = false;
        inp.dim = true;
        let (_, l) = p.update(&inp);
        assert!(l.gen_off[0].lit && l.gen_off[0].brightness < 1.0);
    }

    #[test]
    fn drive_disconnect_needs_guard() {
        let (mut p, mut inp) = base();
        inp.on_ground = true;
        inp.engine_running = [true, true];
        assert!(!p.press_drive_disconnect(0));
        let (o, _) = p.update(&inp);
        assert!(!o.drive_disconnect_request[0]);
        p.drive_disconnect[0].set_guard(true);
        p.press_drive_disconnect(0);
        let (o, _) = p.update(&inp);
        assert!(o.drive_disconnect_request[0]);
    }

    #[test]
    fn drive_disconnect_in_flight_needs_lamp() {
        let (mut p, mut inp) = base();
        inp.on_ground = false;
        inp.engine_running = [true, true];
        p.drive_disconnect[1].set_guard(true);
        p.press_drive_disconnect(1);
        let (o, _) = p.update(&inp);
        assert!(!o.drive_disconnect_request[1]);
        inp.vfsg_lop[1] = true;
        p.press_drive_disconnect(1);
        let (o, l) = p.update(&inp);
        assert!(l.drive[1].lit);
        assert!(o.drive_disconnect_request[1]);
    }

    #[test]
    fn drive_disconnect_ground_shutdown_needs_run() {
        let (mut p, mut inp) = base();
        inp.on_ground = true;
        p.drive_disconnect[0].set_guard(true);
        p.press_drive_disconnect(0);
        let (o, _) = p.update(&inp);
        assert!(!o.drive_disconnect_request[0]);
        inp.fuel_run[0] = true;
        p.press_drive_disconnect(0);
        let (o, _) = p.update(&inp);
        assert!(o.drive_disconnect_request[0]);
    }

    #[test]
    fn ext_pwr_lamps_and_pulse() {
        let (mut p, mut inp) = base();
        inp.ext_fwd_inserted[0] = true;
        inp.ext_fwd_quality_ok[0] = true;
        let (_, l) = p.update(&inp);
        assert!(l.ext_fwd_avail[0].lit && !l.ext_fwd_on[0].lit);
        assert!(!l.ext_fwd_avail[1].lit);
        p.ext_pwr_fwd[0].press();
        let (o, _) = p.update(&inp);
        assert!(o.ext_fwd_pulse[0] && !o.ext_fwd_pulse[1]);
        let (o, _) = p.update(&inp);
        assert!(!o.ext_fwd_pulse[0]);
        inp.ext_fwd_epc_closed[0] = true;
        let (_, l) = p.update(&inp);
        assert!(l.ext_fwd_on[0].lit && !l.ext_fwd_avail[0].lit);
    }

    #[test]
    fn gen_ctrl_cycle_reported_once() {
        let (mut p, inp) = base();
        p.press_gen_ctrl(1);
        let (o, _) = p.update(&inp);
        assert!(!o.gen_ctrl_on[1] && !o.gen_ctrl_cycled[1]);
        p.press_gen_ctrl(1);
        let (o, _) = p.update(&inp);
        assert!(o.gen_ctrl_on[1] && o.gen_ctrl_cycled[1]);
        let (o, _) = p.update(&inp);
        assert!(!o.gen_ctrl_cycled[1]);
    }

    #[test]
    fn apu_selector_start_springs_back() {
        let (mut p, inp) = base();
        p.set_apu_selector(ApuSelector::Start);
        let (o, _) = p.update(&inp);
        assert_eq!(o.apu_selector, ApuSelector::Start);
        p.release_apu_selector();
        let (o, _) = p.update(&inp);
        assert_eq!(o.apu_selector, ApuSelector::On);
    }

    #[test]
    fn ground_service_lamp_toggles_and_clears() {
        let (mut p, mut inp) = base();
        inp.gcb_closed = [false; 4];
        p.ground_service.press();
        let (o, l) = p.update(&inp);
        assert!(o.ground_service_on && l.ground_service_on.lit);
        p.ground_service.press();
        let (o, _) = p.update(&inp);
        assert!(!o.ground_service_on);
        p.ground_service.press();
        let (o, _) = p.update(&inp);
        assert!(o.ground_service_on);
        inp.ext_fwd_epc_closed[0] = true;
        let (o, l) = p.update(&inp);
        assert!(!o.ground_service_on && !l.ground_service_on.lit);
    }

    #[test]
    fn towing_battery_test_lamps() {
        let (mut p, mut inp) = base();
        inp.main_batt_volts = 30.5;
        let (_, l) = p.update(&inp);
        assert!(!l.towing_high.lit);
        p.towing_test.press();
        let (_, l) = p.update(&inp);
        assert!(l.towing_high.lit && !l.towing_medium.lit && !l.towing_low.lit);
        inp.main_batt_volts = 29.9;
        let (_, l) = p.update(&inp);
        assert!(l.towing_medium.lit && !l.towing_high.lit);
        inp.main_batt_volts = 29.0;
        let (_, l) = p.update(&inp);
        assert!(l.towing_low.lit);
        p.towing_test.release();
        let (_, l) = p.update(&inp);
        assert!(!l.towing_low.lit);
        p.towing_power.press();
        let (o, l) = p.update(&inp);
        assert!(o.towing_power_on && l.towing_on_bat.lit);
    }

    #[test]
    fn apu_gen_lamps() {
        let (mut p, mut inp) = base();
        inp.apu_running = true;
        inp.apu_gen_available = [true, true];
        let (_, l) = p.update(&inp);
        assert!(l.apu_gen_avail[0].lit && l.apu_gen_off[0].lit);
        inp.apb_closed[0] = true;
        let (_, l) = p.update(&inp);
        assert!(l.apu_gen_on[0].lit && !l.apu_gen_avail[0].lit && !l.apu_gen_off[0].lit);
    }
}
