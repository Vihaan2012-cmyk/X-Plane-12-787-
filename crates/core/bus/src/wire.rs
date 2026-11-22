#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DiscreteWire {
    Open,
    Ground,
    Volts28,
}

impl DiscreteWire {
    pub fn voltage(self, supply_v: f64) -> f64 {
        match self {
            DiscreteWire::Open => 0.0,
            DiscreteWire::Ground => 0.0,
            DiscreteWire::Volts28 => supply_v,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct DiscreteInput {
    pub on_above_v: f64,
    pub off_below_v: f64,
    pub state: bool,
}

impl DiscreteInput {
    pub fn new(on_above_v: f64, off_below_v: f64) -> Self {
        DiscreteInput {
            on_above_v,
            off_below_v,
            state: false,
        }
    }
    pub fn new_28v() -> Self {
        Self::new(16.0, 8.0)
    }
    pub fn update(&mut self, volts: f64) -> bool {
        if volts >= self.on_above_v {
            self.state = true;
        } else if volts <= self.off_below_v {
            self.state = false;
        }
        self.state
    }
    pub fn update_wire(&mut self, wire: DiscreteWire, supply_v: f64) -> bool {
        self.update(wire.voltage(supply_v))
    }
    pub fn ground_sense(&mut self, wire: DiscreteWire) -> bool {
        self.state = wire == DiscreteWire::Ground;
        self.state
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct AdcReading {
    pub code: u32,
    pub value: f64,
    pub valid: bool,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct AnalogInput {
    pub bits: u32,
    pub min_v: f64,
    pub max_v: f64,
    pub fault_margin_v: f64,
}

impl AnalogInput {
    pub fn new(bits: u32, min_v: f64, max_v: f64) -> Self {
        AnalogInput {
            bits: bits.clamp(1, 24),
            min_v,
            max_v,
            fault_margin_v: 0.05 * (max_v - min_v),
        }
    }
    pub fn lsb(&self) -> f64 {
        (self.max_v - self.min_v) / f64::from((1u32 << self.bits) - 1)
    }
    pub fn sample(&self, volts: f64) -> AdcReading {
        let valid = volts.is_finite()
            && volts >= self.min_v - self.fault_margin_v
            && volts <= self.max_v + self.fault_margin_v;
        let v = if volts.is_finite() { volts } else { self.min_v };
        let clamped = v.clamp(self.min_v, self.max_v);
        let full = f64::from((1u32 << self.bits) - 1);
        let code = ((clamped - self.min_v) / (self.max_v - self.min_v) * full).round() as u32;
        AdcReading {
            code,
            value: self.min_v + f64::from(code) * self.lsb(),
            valid,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn discrete_hysteresis() {
        let mut d = DiscreteInput::new_28v();
        assert!(!d.update(10.0));
        assert!(!d.update(15.9));
        assert!(d.update(16.0));
        assert!(d.update(10.0));
        assert!(d.update(8.1));
        assert!(!d.update(8.0));
        assert!(!d.update(12.0));
    }

    #[test]
    fn discrete_wires() {
        let mut d = DiscreteInput::new_28v();
        assert!(d.update_wire(DiscreteWire::Volts28, 28.0));
        assert!(!d.update_wire(DiscreteWire::Open, 28.0));
        assert!(!d.update_wire(DiscreteWire::Ground, 28.0));
        assert!(d.update_wire(DiscreteWire::Volts28, 24.0));
        assert!(!d.update_wire(DiscreteWire::Volts28, 5.0));
        assert!(d.ground_sense(DiscreteWire::Ground));
        assert!(!d.ground_sense(DiscreteWire::Open));
    }

    #[test]
    fn adc_quantisation() {
        let a = AnalogInput::new(12, 0.0, 10.0);
        assert!((a.lsb() - 10.0 / 4095.0).abs() < 1e-12);
        let r = a.sample(5.0);
        assert!(r.valid);
        assert_eq!(r.code, 2048);
        assert!((r.value - 5.0).abs() <= a.lsb());
        assert_eq!(a.sample(0.0).code, 0);
        assert_eq!(a.sample(10.0).code, 4095);
    }

    #[test]
    fn adc_validity() {
        let a = AnalogInput::new(10, 0.5, 4.5);
        assert!(a.sample(0.4).valid);
        assert!(!a.sample(0.1).valid);
        assert!(!a.sample(5.0).valid);
        assert!(!a.sample(f64::NAN).valid);
        assert_eq!(a.sample(100.0).code, 1023);
    }
}
