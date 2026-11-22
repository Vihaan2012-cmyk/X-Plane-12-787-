use super::contactor::{Contactor, ContactorConfig, ContactorInput};
use super::thermal_breaker::{BreakerInput, BreakerState, ThermalBreaker};

pub const MAX_P_PER_C: usize = 6;
pub const MAX_ELCC_PER_P: usize = 6;
pub const COIL_DRIVE_V: f64 = 28.0;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SupplyClass {
    Ac235,
    Dc270,
    Ac115,
}

impl SupplyClass {
    pub fn on_threshold(self) -> f64 {
        match self {
            SupplyClass::Ac235 => 165.0,
            SupplyClass::Dc270 => 205.0,
            SupplyClass::Ac115 => 85.0,
        }
    }

    pub fn off_threshold(self) -> f64 {
        match self {
            SupplyClass::Ac235 => 130.0,
            SupplyClass::Dc270 => 170.0,
            SupplyClass::Ac115 => 50.0,
        }
    }
}

#[derive(Clone, Debug)]
pub struct ElcuPowerSense {
    class: SupplyClass,
    powered: bool,
}

impl ElcuPowerSense {
    pub fn new(class: SupplyClass) -> Self {
        Self {
            class,
            powered: false,
        }
    }

    pub fn powered(&self) -> bool {
        self.powered
    }

    pub fn update(&mut self, volts: f64) -> bool {
        if volts > self.class.on_threshold() {
            self.powered = true;
        } else if volts < self.class.off_threshold() {
            self.powered = false;
        }
        self.powered
    }
}

#[derive(Clone, Copy, Debug)]
pub struct ElccInput {
    pub dt_us: u64,
    pub close_cmd: bool,
    pub current_a: f64,
    pub line_v: f64,
    pub ambient_c: f64,
    pub elcu_powered: bool,
    pub pull: bool,
    pub push: bool,
}

#[derive(Clone, Copy, Debug)]
pub struct ElccOutput {
    pub closed: bool,
    pub breaker: BreakerState,
    pub current_a: f64,
    pub welded: bool,
}

#[derive(Clone, Debug)]
pub struct Elcc {
    pub breaker: ThermalBreaker,
    pub contactor: Contactor,
}

impl Elcc {
    pub fn new(rating_a: f64) -> Self {
        Self {
            breaker: ThermalBreaker::new(rating_a),
            contactor: Contactor::new(ContactorConfig::default()),
        }
    }

    pub fn step(&mut self, input: &ElccInput) -> ElccOutput {
        let coil = if input.elcu_powered && input.close_cmd && self.breaker.state() == BreakerState::Closed {
            COIL_DRIVE_V
        } else {
            0.0
        };
        let c = self.contactor.step(
            &ContactorInput {
                dt_us: input.dt_us,
                coil_v: coil,
                current_a: input.current_a,
            },
            input.line_v,
        );
        let flowing = if c.contacts_closed { input.current_a } else { 0.0 };
        let b = self.breaker.step(&BreakerInput {
            dt_us: input.dt_us,
            current_a: flowing,
            ambient_c: input.ambient_c,
            pull: input.pull,
            push: input.push,
        });
        ElccOutput {
            closed: c.contacts_closed && b.closed,
            breaker: b.state,
            current_a: if c.contacts_closed { b.current_a } else { 0.0 },
            welded: c.welded,
        }
    }
}

#[derive(Clone, Debug)]
pub struct ElcuP {
    pub elccs: Vec<Elcc>,
}

impl ElcuP {
    pub fn new(ratings_a: &[f64]) -> Self {
        Self {
            elccs: ratings_a
                .iter()
                .take(MAX_ELCC_PER_P)
                .map(|r| Elcc::new(*r))
                .collect(),
        }
    }
}

#[derive(Clone, Debug)]
pub struct ElcuC {
    pub sense: ElcuPowerSense,
    pub modules: Vec<ElcuP>,
}

impl ElcuC {
    pub fn new(class: SupplyClass) -> Self {
        Self {
            sense: ElcuPowerSense::new(class),
            modules: Vec::new(),
        }
    }

    pub fn add_module(&mut self, module: ElcuP) -> bool {
        if self.modules.len() >= MAX_P_PER_C {
            return false;
        }
        self.modules.push(module);
        true
    }

    pub fn step(
        &mut self,
        supply_v: f64,
        dt_us: u64,
        close_cmds: &[Vec<bool>],
        currents_a: &[Vec<f64>],
        line_v: f64,
        ambient_c: f64,
    ) -> Vec<Vec<ElccOutput>> {
        let powered = self.sense.update(supply_v);
        let mut out = Vec::new();
        for (m, module) in self.modules.iter_mut().enumerate() {
            let mut row = Vec::new();
            for (c, elcc) in module.elccs.iter_mut().enumerate() {
                let close = close_cmds.get(m).and_then(|r| r.get(c)).copied().unwrap_or(false);
                let cur = currents_a.get(m).and_then(|r| r.get(c)).copied().unwrap_or(0.0);
                row.push(elcc.step(&ElccInput {
                    dt_us,
                    close_cmd: close,
                    current_a: cur,
                    line_v,
                    ambient_c,
                    elcu_powered: powered,
                    pull: false,
                    push: false,
                }));
            }
            out.push(row);
        }
        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn thresholds_with_hysteresis() {
        for (class, on, off) in [
            (SupplyClass::Ac235, 165.0, 130.0),
            (SupplyClass::Dc270, 205.0, 170.0),
            (SupplyClass::Ac115, 85.0, 50.0),
        ] {
            let mut s = ElcuPowerSense::new(class);
            assert!(!s.update(on - 1.0));
            assert!(!s.update(on));
            assert!(s.update(on + 1.0));
            assert!(s.update((on + off) / 2.0));
            assert!(s.update(off));
            assert!(!s.update(off - 1.0));
            assert!(!s.update((on + off) / 2.0));
        }
    }

    #[test]
    fn elcc_closes_after_operate_time_and_trips_on_overload() {
        let mut e = Elcc::new(20.0);
        let base = ElccInput {
            dt_us: 10_000,
            close_cmd: true,
            current_a: 10.0,
            line_v: 235.0,
            ambient_c: 25.0,
            elcu_powered: true,
            pull: false,
            push: false,
        };
        let mut o = e.step(&base);
        assert!(!o.closed);
        for _ in 0..100 {
            o = e.step(&base);
        }
        assert!(o.closed);
        let over = ElccInput {
            current_a: 60.0,
            ..base
        };
        let mut n = 0;
        while e.step(&over).breaker == BreakerState::Closed {
            n += 1;
            assert!(n < 100_000);
        }
        let o = e.step(&over);
        assert!(!o.closed);
    }

    #[test]
    fn unpowered_elcu_keeps_contactor_open() {
        let mut e = Elcc::new(20.0);
        let inp = ElccInput {
            dt_us: 10_000,
            close_cmd: true,
            current_a: 0.0,
            line_v: 235.0,
            ambient_c: 25.0,
            elcu_powered: false,
            pull: false,
            push: false,
        };
        for _ in 0..100 {
            assert!(!e.step(&inp).closed);
        }
    }

    #[test]
    fn c_module_controls_up_to_six_p_modules() {
        let mut c = ElcuC::new(SupplyClass::Ac235);
        for _ in 0..6 {
            assert!(c.add_module(ElcuP::new(&[20.0; 8])));
        }
        assert!(!c.add_module(ElcuP::new(&[20.0])));
        assert_eq!(c.modules[0].elccs.len(), MAX_ELCC_PER_P);
    }

    #[test]
    fn c_module_step_requires_supply() {
        let mut c = ElcuC::new(SupplyClass::Ac235);
        c.add_module(ElcuP::new(&[20.0, 20.0]));
        let cmds = vec![vec![true, false]];
        let cur = vec![vec![5.0, 0.0]];
        let mut last = Vec::new();
        for _ in 0..100 {
            last = c.step(100.0, 10_000, &cmds, &cur, 235.0, 25.0);
        }
        assert!(!last[0][0].closed);
        for _ in 0..100 {
            last = c.step(235.0, 10_000, &cmds, &cur, 235.0, 25.0);
        }
        assert!(last[0][0].closed);
        assert!(!last[0][1].closed);
    }
}
