#[macro_use]
mod macros;
mod complex;
mod conversions;
mod impedance;
mod quantities;
mod relations;
mod three_phase;

pub use complex::Complex;
pub use conversions::{
    ZERO_CELSIUS_KELVIN, apparent_power, power_factor, reactive_power_from_pf, watts_from_apparent,
};
pub use impedance::{Admittance, ComplexAmps, ComplexPower, ComplexVolts, Impedance};
pub use quantities::*;
pub use three_phase::{
    SequenceAmps, SequenceComponents, SequenceVolts, ThreePhaseAmps, ThreePhaseVolts, alpha, fortescue,
    inverse_fortescue,
};

pub fn approx_eq(a: f64, b: f64, tolerance: f64) -> bool {
    (a - b).abs() <= tolerance
}

#[cfg(test)]
mod tests;
