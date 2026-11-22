pub const BOUNCE_EDGES_US: [u64; 5] = [0, 400, 700, 1_000, 1_200];
pub const ARC_VOLTAGE_V: f64 = 20.0;
pub const ARC_TIME_S: f64 = 0.001;
pub const WEAR_PER_CYCLE: f64 = 2e-5;
pub const WEAR_PER_ARC_J: f64 = 5e-4;

#[derive(Clone, Copy, Debug)]
pub struct ContactorConfig {
    pub pickup_v: f64,
    pub dropout_v: f64,
    pub operate_us: u64,
    pub release_us: u64,
    pub r0_ohm: f64,
    pub weld_energy_j: f64,
}

impl Default for ContactorConfig {
    fn default() -> Self {
        Self {
            pickup_v: 18.0,
            dropout_v: 5.0,
            operate_us: 15_000,
            release_us: 8_000,
            r0_ohm: 0.001,
            weld_energy_j: 5_000.0,
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub struct ContactorInput {
    pub dt_us: u64,
    pub coil_v: f64,
    pub current_a: f64,
}

#[derive(Clone, Copy, Debug)]
pub struct ContactorOutput {
    pub contacts_closed: bool,
    pub armature_closed: bool,
    pub aux_no: bool,
    pub aux_nc: bool,
    pub resistance_ohm: f64,
    pub welded: bool,
    pub bouncing: bool,
}

#[derive(Clone, Debug)]
pub struct Contactor {
    pub config: ContactorConfig,
    coil_energized: bool,
    armature: bool,
    timer_us: u64,
    since_operate_us: u64,
    cycles: u64,
    arc_energy_j: f64,
    welded: bool,
}

impl Contactor {
    pub fn new(config: ContactorConfig) -> Self {
        Self {
            config,
            coil_energized: false,
            armature: false,
            timer_us: 0,
            since_operate_us: 0,
            cycles: 0,
            arc_energy_j: 0.0,
            welded: false,
        }
    }

    pub fn cycles(&self) -> u64 {
        self.cycles
    }

    pub fn arc_energy_j(&self) -> f64 {
        self.arc_energy_j
    }

    pub fn welded(&self) -> bool {
        self.welded
    }

    pub fn force_weld(&mut self) {
        self.welded = true;
    }

    pub fn resistance_ohm(&self) -> f64 {
        self.config.r0_ohm * (1.0 + WEAR_PER_CYCLE * self.cycles as f64 + WEAR_PER_ARC_J * self.arc_energy_j)
    }

    fn contacts_closed(&self) -> bool {
        if self.welded {
            return true;
        }
        if !self.armature {
            return false;
        }
        let n = BOUNCE_EDGES_US
            .iter()
            .filter(|e| **e <= self.since_operate_us)
            .count();
        n % 2 == 1
    }

    pub fn step(&mut self, input: &ContactorInput, line_v: f64) -> ContactorOutput {
        if input.coil_v >= self.config.pickup_v {
            self.coil_energized = true;
        } else if input.coil_v < self.config.dropout_v {
            self.coil_energized = false;
        }
        if self.coil_energized {
            if !self.armature {
                self.timer_us += input.dt_us;
                if self.timer_us >= self.config.operate_us {
                    self.armature = true;
                    self.timer_us = 0;
                    self.since_operate_us = self.timer_us;
                    self.cycles += 1;
                }
            } else {
                self.timer_us = 0;
                self.since_operate_us = self.since_operate_us.saturating_add(input.dt_us);
            }
        } else if self.armature {
            self.timer_us += input.dt_us;
            if self.timer_us >= self.config.release_us {
                self.armature = false;
                self.timer_us = 0;
                self.since_operate_us = 0;
                let e = input.current_a.abs()
                    * line_v.abs().clamp(ARC_VOLTAGE_V, ARC_VOLTAGE_V * 3.0)
                    * ARC_TIME_S;
                self.arc_energy_j += e;
                if self.arc_energy_j >= self.config.weld_energy_j {
                    self.welded = true;
                }
            }
        } else {
            self.timer_us = 0;
        }
        let closed = self.contacts_closed();
        ContactorOutput {
            contacts_closed: closed,
            armature_closed: self.armature,
            aux_no: self.armature,
            aux_nc: !self.armature,
            resistance_ohm: self.resistance_ohm(),
            welded: self.welded,
            bouncing: self.armature && self.since_operate_us < *BOUNCE_EDGES_US.last().unwrap_or(&0),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn inp(coil_v: f64, dt_us: u64, current_a: f64) -> ContactorInput {
        ContactorInput {
            dt_us,
            coil_v,
            current_a,
        }
    }

    #[test]
    fn pickup_threshold_and_hysteresis() {
        let mut c = Contactor::new(ContactorConfig::default());
        for _ in 0..100 {
            let o = c.step(&inp(12.0, 1_000, 0.0), 28.0);
            assert!(!o.armature_closed);
        }
        for _ in 0..16 {
            c.step(&inp(28.0, 1_000, 0.0), 28.0);
        }
        for _ in 0..50 {
            let o = c.step(&inp(10.0, 1_000, 0.0), 28.0);
            if o.armature_closed {
                continue;
            }
            panic!("dropped between dropout and pickup");
        }
    }

    #[test]
    fn operate_and_release_times() {
        let mut c = Contactor::new(ContactorConfig::default());
        let mut t = 0u64;
        while !c.step(&inp(28.0, 100, 0.0), 28.0).armature_closed {
            t += 100;
        }
        assert!((14_900..=15_100).contains(&t), "{t}");
        for _ in 0..30 {
            c.step(&inp(28.0, 100, 0.0), 28.0);
        }
        let mut r = 0u64;
        while c.step(&inp(0.0, 100, 0.0), 28.0).armature_closed {
            r += 100;
        }
        assert!((7_900..=8_100).contains(&r), "{r}");
    }

    #[test]
    fn bounce_sequence() {
        let mut c = Contactor::new(ContactorConfig::default());
        let mut seen = Vec::new();
        let mut armature_seen = false;
        for _ in 0..400 {
            let o = c.step(&inp(28.0, 50, 0.0), 28.0);
            if o.armature_closed {
                armature_seen = true;
                assert!(!o.aux_nc && o.aux_no);
            }
            if armature_seen && seen.last() != Some(&o.contacts_closed) {
                seen.push(o.contacts_closed);
            }
        }
        assert_eq!(seen, vec![true, false, true, false, true]);
        let o = c.step(&inp(28.0, 50, 0.0), 28.0);
        assert!(!o.bouncing && o.contacts_closed);
    }

    #[test]
    fn wear_raises_resistance() {
        let mut c = Contactor::new(ContactorConfig::default());
        let r0 = c.resistance_ohm();
        for _ in 0..200 {
            for _ in 0..200 {
                c.step(&inp(28.0, 1_000, 50.0), 28.0);
            }
            for _ in 0..20 {
                c.step(&inp(0.0, 1_000, 50.0), 28.0);
            }
        }
        assert!(c.cycles() >= 190);
        assert!(c.arc_energy_j() > 100.0);
        assert!(c.resistance_ohm() > r0 * 1.1);
    }

    #[test]
    fn welded_contacts_stay_closed() {
        let mut c = Contactor::new(ContactorConfig::default());
        for _ in 0..20 {
            c.step(&inp(28.0, 1_000, 0.0), 28.0);
        }
        c.force_weld();
        let mut o = c.step(&inp(0.0, 1_000, 0.0), 28.0);
        for _ in 0..20 {
            o = c.step(&inp(0.0, 1_000, 0.0), 28.0);
        }
        assert!(!o.armature_closed && o.aux_nc);
        assert!(o.contacts_closed && o.welded);
    }

    #[test]
    fn arc_energy_can_weld() {
        let mut c = Contactor::new(ContactorConfig {
            weld_energy_j: 1.0,
            ..Default::default()
        });
        for _ in 0..20 {
            c.step(&inp(28.0, 1_000, 100.0), 28.0);
        }
        for _ in 0..20 {
            c.step(&inp(0.0, 1_000, 100.0), 28.0);
        }
        assert!(c.welded());
    }
}
