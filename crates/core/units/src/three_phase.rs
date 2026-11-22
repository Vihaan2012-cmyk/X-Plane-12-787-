use crate::complex::Complex;
use crate::impedance::{ComplexAmps, ComplexPower, ComplexVolts};
use crate::quantities::*;
use core::f64::consts::TAU;

pub fn alpha() -> Complex {
    Complex::from_polar(1.0, TAU / 3.0)
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct SequenceComponents {
    pub zero: Complex,
    pub positive: Complex,
    pub negative: Complex,
}

impl SequenceComponents {
    pub fn approx_eq(self, other: Self, tolerance: f64) -> bool {
        self.zero.approx_eq(other.zero, tolerance)
            && self.positive.approx_eq(other.positive, tolerance)
            && self.negative.approx_eq(other.negative, tolerance)
    }
    pub fn unbalance_ratio(self) -> f64 {
        let p = self.positive.magnitude();
        if p == 0.0 {
            0.0
        } else {
            self.negative.magnitude() / p
        }
    }
    pub fn zero_ratio(self) -> f64 {
        let p = self.positive.magnitude();
        if p == 0.0 { 0.0 } else { self.zero.magnitude() / p }
    }
}

pub fn fortescue(a: Complex, b: Complex, c: Complex) -> SequenceComponents {
    let al = alpha();
    let al2 = al * al;
    SequenceComponents {
        zero: (a + b + c) / 3.0,
        positive: (a + al * b + al2 * c) / 3.0,
        negative: (a + al2 * b + al * c) / 3.0,
    }
}

pub fn inverse_fortescue(s: SequenceComponents) -> [Complex; 3] {
    let al = alpha();
    let al2 = al * al;
    [
        s.zero + s.positive + s.negative,
        s.zero + al2 * s.positive + al * s.negative,
        s.zero + al * s.positive + al2 * s.negative,
    ]
}

macro_rules! three_phase {
    ($set:ident, $seq:ident, $phasor:ident, $unit:ident) => {
        #[derive(Clone, Copy, Debug, Default, PartialEq)]
        pub struct $set {
            pub a: $phasor,
            pub b: $phasor,
            pub c: $phasor,
        }

        #[derive(Clone, Copy, Debug, Default, PartialEq)]
        pub struct $seq {
            pub zero: $phasor,
            pub positive: $phasor,
            pub negative: $phasor,
        }

        impl $set {
            pub const ZERO: Self = Self {
                a: $phasor::ZERO,
                b: $phasor::ZERO,
                c: $phasor::ZERO,
            };
            pub fn new(a: $phasor, b: $phasor, c: $phasor) -> Self {
                Self { a, b, c }
            }
            pub fn balanced(magnitude: $unit, angle: f64) -> Self {
                Self {
                    a: $phasor::from_polar(magnitude, angle),
                    b: $phasor::from_polar(magnitude, angle - TAU / 3.0),
                    c: $phasor::from_polar(magnitude, angle + TAU / 3.0),
                }
            }
            pub fn balanced_negative_sequence(magnitude: $unit, angle: f64) -> Self {
                Self {
                    a: $phasor::from_polar(magnitude, angle),
                    b: $phasor::from_polar(magnitude, angle + TAU / 3.0),
                    c: $phasor::from_polar(magnitude, angle - TAU / 3.0),
                }
            }
            pub fn magnitudes(self) -> [$unit; 3] {
                [self.a.magnitude(), self.b.magnitude(), self.c.magnitude()]
            }
            pub fn sequence(self) -> $seq {
                let s = fortescue(self.a.0, self.b.0, self.c.0);
                $seq {
                    zero: $phasor(s.zero),
                    positive: $phasor(s.positive),
                    negative: $phasor(s.negative),
                }
            }
            pub fn sum(self) -> $phasor {
                self.a + self.b + self.c
            }
            pub fn rotate(self, angle: f64) -> Self {
                Self {
                    a: self.a.rotate(angle),
                    b: self.b.rotate(angle),
                    c: self.c.rotate(angle),
                }
            }
            pub fn approx_eq(self, other: Self, tolerance: f64) -> bool {
                self.a.approx_eq(other.a, tolerance)
                    && self.b.approx_eq(other.b, tolerance)
                    && self.c.approx_eq(other.c, tolerance)
            }
        }

        impl $seq {
            pub fn new(zero: $phasor, positive: $phasor, negative: $phasor) -> Self {
                Self {
                    zero,
                    positive,
                    negative,
                }
            }
            pub fn from_positive(positive: $phasor) -> Self {
                Self {
                    zero: $phasor::ZERO,
                    positive,
                    negative: $phasor::ZERO,
                }
            }
            pub fn phases(self) -> $set {
                let [a, b, c] = inverse_fortescue(SequenceComponents {
                    zero: self.zero.0,
                    positive: self.positive.0,
                    negative: self.negative.0,
                });
                $set {
                    a: $phasor(a),
                    b: $phasor(b),
                    c: $phasor(c),
                }
            }
            pub fn unbalance_ratio(self) -> f64 {
                SequenceComponents {
                    zero: self.zero.0,
                    positive: self.positive.0,
                    negative: self.negative.0,
                }
                .unbalance_ratio()
            }
            pub fn approx_eq(self, other: Self, tolerance: f64) -> bool {
                self.zero.approx_eq(other.zero, tolerance)
                    && self.positive.approx_eq(other.positive, tolerance)
                    && self.negative.approx_eq(other.negative, tolerance)
            }
        }
    };
}

three_phase!(ThreePhaseVolts, SequenceVolts, ComplexVolts, Volts);
three_phase!(ThreePhaseAmps, SequenceAmps, ComplexAmps, Amps);

impl ThreePhaseVolts {
    pub fn line_to_line(self) -> Self {
        Self {
            a: self.a - self.b,
            b: self.b - self.c,
            c: self.c - self.a,
        }
    }
    pub fn power(self, current: ThreePhaseAmps) -> ComplexPower {
        self.a.power(current.a) + self.b.power(current.b) + self.c.power(current.c)
    }
}

impl ThreePhaseAmps {
    pub fn power(self, voltage: ThreePhaseVolts) -> ComplexPower {
        voltage.power(self)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::impedance::Impedance;
    use core::f64::consts::{FRAC_PI_6, PI};

    const TOL: f64 = 1e-9;

    #[test]
    fn alpha_properties() {
        let a = alpha();
        assert!((a.magnitude() - 1.0).abs() < TOL);
        assert!((a.angle() - TAU / 3.0).abs() < TOL);
        assert!((a * a * a).approx_eq(Complex::ONE, TOL));
        assert!((Complex::ONE + a + a * a).magnitude() < TOL);
    }

    #[test]
    fn balanced_set_shape() {
        let s = ThreePhaseVolts::balanced(Volts(115.0), 0.3);
        for m in s.magnitudes() {
            assert!(m.approx_eq(Volts(115.0), TOL));
        }
        assert!(s.sum().magnitude().approx_eq(Volts(0.0), TOL));
        assert!((s.a.angle() - 0.3).abs() < TOL);
        assert!((s.b.angle() - (0.3 - TAU / 3.0)).abs() < TOL);
        assert!((s.c.angle() - (0.3 + TAU / 3.0)).abs() < TOL);
    }

    #[test]
    fn balanced_positive_has_only_positive_sequence() {
        let s = ThreePhaseVolts::balanced(Volts(115.0), 0.7);
        let q = s.sequence();
        assert!(q.zero.magnitude().approx_eq(Volts(0.0), TOL));
        assert!(q.negative.magnitude().approx_eq(Volts(0.0), TOL));
        assert!(q.positive.approx_eq(s.a, TOL));
        assert!(q.unbalance_ratio() < TOL);
    }

    #[test]
    fn balanced_negative_has_only_negative_sequence() {
        let s = ThreePhaseVolts::balanced_negative_sequence(Volts(115.0), 0.7);
        let q = s.sequence();
        assert!(q.zero.magnitude().approx_eq(Volts(0.0), TOL));
        assert!(q.positive.magnitude().approx_eq(Volts(0.0), TOL));
        assert!(q.negative.approx_eq(s.a, TOL));
    }

    #[test]
    fn zero_sequence_from_common_mode() {
        let c = ComplexVolts::from_polar(Volts(10.0), 0.2);
        let s = ThreePhaseVolts::new(c, c, c);
        let q = s.sequence();
        assert!(q.zero.approx_eq(c, TOL));
        assert!(q.positive.magnitude().approx_eq(Volts(0.0), TOL));
        assert!(q.negative.magnitude().approx_eq(Volts(0.0), TOL));
    }

    #[test]
    fn fortescue_round_trip_raw() {
        let a = Complex::new(100.0, 20.0);
        let b = Complex::new(-40.0, -90.0);
        let c = Complex::new(-70.0, 55.0);
        let s = fortescue(a, b, c);
        let [a2, b2, c2] = inverse_fortescue(s);
        assert!(a.approx_eq(a2, TOL) && b.approx_eq(b2, TOL) && c.approx_eq(c2, TOL));
        let s2 = fortescue(a2, b2, c2);
        assert!(s.approx_eq(s2, TOL));
    }

    #[test]
    fn fortescue_round_trip_typed() {
        let s = ThreePhaseAmps::new(
            ComplexAmps::new(Amps(10.0), Amps(2.0)),
            ComplexAmps::new(Amps(-3.0), Amps(-8.0)),
            ComplexAmps::new(Amps(-6.0), Amps(7.0)),
        );
        assert!(s.sequence().phases().approx_eq(s, TOL));
        let q = SequenceAmps::new(
            ComplexAmps::new(Amps(1.0), Amps(1.0)),
            ComplexAmps::new(Amps(20.0), Amps(-4.0)),
            ComplexAmps::new(Amps(2.0), Amps(3.0)),
        );
        assert!(q.phases().sequence().approx_eq(q, TOL));
    }

    #[test]
    fn unbalanced_sequence_values() {
        let s = ThreePhaseVolts::new(
            ComplexVolts::from_real(Volts(120.0)),
            ComplexVolts::from_real(Volts(0.0)),
            ComplexVolts::from_real(Volts(0.0)),
        );
        let q = s.sequence();
        assert!(q.zero.magnitude().approx_eq(Volts(40.0), TOL));
        assert!(q.positive.magnitude().approx_eq(Volts(40.0), TOL));
        assert!(q.negative.magnitude().approx_eq(Volts(40.0), TOL));
        assert!((q.unbalance_ratio() - 1.0).abs() < TOL);
        let raw = fortescue(Complex::from_real(120.0), Complex::ZERO, Complex::ZERO);
        assert!((raw.zero_ratio() - 1.0).abs() < TOL);
    }

    #[test]
    fn sequence_from_positive_and_phases() {
        let p = ComplexVolts::from_polar(Volts(115.0), 0.0);
        let s = SequenceVolts::from_positive(p).phases();
        assert!(s.approx_eq(ThreePhaseVolts::balanced(Volts(115.0), 0.0), TOL));
    }

    #[test]
    fn line_to_line_is_sqrt3_and_30_degrees() {
        let s = ThreePhaseVolts::balanced(Volts(115.0), 0.0);
        let ll = s.line_to_line();
        for m in ll.magnitudes() {
            assert!(m.approx_eq(Volts(115.0).line_to_line_from_phase(), TOL));
        }
        assert!((ll.a.angle() - FRAC_PI_6).abs() < TOL);
    }

    #[test]
    fn balanced_three_phase_power() {
        let v = ThreePhaseVolts::balanced(Volts(115.0), 0.0);
        let z = Impedance::new(Ohms(4.0), Ohms(3.0));
        let i = ThreePhaseAmps::new(v.a / z, v.b / z, v.c / z);
        let s = v.power(i);
        assert_eq!(s, i.power(v));
        let single = v.a.power(i.a);
        assert!(s.approx_eq(single * 3.0, TOL));
        assert!(
            s.active()
                .approx_eq(Watts(3.0 * 115.0 * 115.0 * 4.0 / 25.0), 1e-6)
        );
    }

    #[test]
    fn rotate_set() {
        let s = ThreePhaseVolts::balanced(Volts(115.0), 0.0).rotate(PI / 2.0);
        assert!(s.approx_eq(ThreePhaseVolts::balanced(Volts(115.0), PI / 2.0), TOL));
        assert_eq!(ThreePhaseAmps::ZERO.sum(), ComplexAmps::ZERO);
    }
}
