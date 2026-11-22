#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SwitchKind {
    Alternate,
    Momentary,
    GuardedMomentary,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Switch {
    pub kind: SwitchKind,
    pub latched: bool,
    pub pressed: bool,
    pub guard_open: bool,
    pulse: bool,
}

impl Switch {
    pub fn new(kind: SwitchKind) -> Self {
        Self {
            kind,
            latched: false,
            pressed: false,
            guard_open: false,
            pulse: false,
        }
    }

    pub fn press(&mut self) -> bool {
        match self.kind {
            SwitchKind::Alternate => {
                self.latched = !self.latched;
                true
            }
            SwitchKind::Momentary => {
                self.pressed = true;
                self.pulse = true;
                true
            }
            SwitchKind::GuardedMomentary => {
                if self.guard_open {
                    self.pressed = true;
                    self.pulse = true;
                    true
                } else {
                    false
                }
            }
        }
    }

    pub fn release(&mut self) {
        self.pressed = false;
    }

    pub fn set_guard(&mut self, open: bool) {
        self.guard_open = open;
        if !open {
            self.pressed = false;
        }
    }

    pub fn set_latched(&mut self, on: bool) {
        self.latched = on;
    }

    pub fn is_on(&self) -> bool {
        self.latched
    }

    pub fn take_pulse(&mut self) -> bool {
        let p = self.pulse;
        self.pulse = false;
        p
    }

    pub fn has_pulse(&self) -> bool {
        self.pulse
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LampColor {
    White,
    Amber,
    Green,
    Blue,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Lamp {
    pub color: LampColor,
    pub lit: bool,
    pub brightness: f64,
}

pub const BRIGHT_LEVEL: f64 = 1.0;
pub const DIM_LEVEL: f64 = 0.4;

impl Lamp {
    pub fn drive(color: LampColor, command: bool, lamp_test: bool, dim: bool) -> Self {
        let lit = command || lamp_test;
        let brightness = if !lit {
            0.0
        } else if lamp_test || !dim {
            BRIGHT_LEVEL
        } else {
            DIM_LEVEL
        };
        Self {
            color,
            lit,
            brightness,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn alternate_toggles() {
        let mut s = Switch::new(SwitchKind::Alternate);
        assert!(!s.is_on());
        assert!(s.press());
        assert!(s.is_on());
        s.press();
        assert!(!s.is_on());
    }

    #[test]
    fn momentary_pulse_once() {
        let mut s = Switch::new(SwitchKind::Momentary);
        s.press();
        assert!(s.pressed);
        assert!(s.take_pulse());
        assert!(!s.take_pulse());
        s.release();
        assert!(!s.pressed);
    }

    #[test]
    fn guarded_needs_guard() {
        let mut s = Switch::new(SwitchKind::GuardedMomentary);
        assert!(!s.press());
        assert!(!s.has_pulse());
        s.set_guard(true);
        assert!(s.press());
        assert!(s.take_pulse());
        s.set_guard(false);
        assert!(!s.pressed);
    }

    #[test]
    fn lamp_levels() {
        let l = Lamp::drive(LampColor::Amber, false, false, false);
        assert!(!l.lit && l.brightness == 0.0);
        let l = Lamp::drive(LampColor::Amber, true, false, true);
        assert!(l.lit && l.brightness == DIM_LEVEL);
        let l = Lamp::drive(LampColor::Amber, false, true, true);
        assert!(l.lit && l.brightness == BRIGHT_LEVEL);
    }
}
