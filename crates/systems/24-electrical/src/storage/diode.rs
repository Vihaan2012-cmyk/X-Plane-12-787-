pub const APPROX_DIODE_VF_V: f64 = 0.45;
pub const APPROX_DIODE_R_ON_OHM: f64 = 0.004;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct BatteryDiodeModule {
    pub vf: f64,
    pub r_on_ohm: f64,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct DiodeOutput {
    pub conducting: bool,
    pub bus_side_v: f64,
    pub current_a: f64,
}

impl Default for BatteryDiodeModule {
    fn default() -> Self {
        Self {
            vf: APPROX_DIODE_VF_V,
            r_on_ohm: APPROX_DIODE_R_ON_OHM,
        }
    }
}

impl BatteryDiodeModule {
    pub fn conducts(&self, battery_v: f64, bus_v: f64) -> bool {
        battery_v - self.vf > bus_v
    }

    pub fn evaluate(
        &self,
        battery_v: f64,
        battery_r_ohm: f64,
        other_source_v: f64,
        load_a: f64,
    ) -> DiodeOutput {
        let v_out = battery_v - load_a * (battery_r_ohm + self.r_on_ohm) - self.vf;
        let conducting = v_out > other_source_v && load_a > 0.0;
        DiodeOutput {
            conducting,
            bus_side_v: if conducting { v_out } else { other_source_v },
            current_a: if conducting { load_a } else { 0.0 },
        }
    }

    pub fn forward_v(&self, battery_v: f64, load_a: f64) -> f64 {
        (battery_v - self.vf - load_a * self.r_on_ohm).max(0.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn blocks_reverse_and_small_forward() {
        let d = BatteryDiodeModule::default();
        assert!(!d.conducts(28.0, 28.0));
        assert!(!d.conducts(28.0, 28.3));
        assert!(d.conducts(28.0, 27.0));
        let o = d.evaluate(26.0, 0.05, 28.0, 10.0);
        assert!(!o.conducting && o.current_a == 0.0 && o.bus_side_v == 28.0);
    }

    #[test]
    fn conducts_when_other_source_collapses() {
        let d = BatteryDiodeModule::default();
        let o = d.evaluate(29.0, 0.05, 0.0, 40.0);
        assert!(o.conducting && o.current_a == 40.0);
        assert!((o.bus_side_v - (29.0 - 0.45 - 40.0 * 0.054)).abs() < 1e-9);
        assert!((d.forward_v(29.0, 10.0) - (29.0 - 0.45 - 0.04)).abs() < 1e-9);
    }
}
