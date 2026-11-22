use crate::complex::Complex;
use crate::quantities::*;

macro_rules! phasor {
    ($name:ident, $unit:ident) => {
        #[derive(Clone, Copy, Debug, Default, PartialEq)]
        pub struct $name(pub Complex);

        impl $name {
            pub const ZERO: Self = Self(Complex::ZERO);
            pub fn new(re: $unit, im: $unit) -> Self {
                Self(Complex::new(re.0, im.0))
            }
            pub fn from_polar(magnitude: $unit, angle: f64) -> Self {
                Self(Complex::from_polar(magnitude.0, angle))
            }
            pub fn from_real(re: $unit) -> Self {
                Self(Complex::from_real(re.0))
            }
            pub fn from_imag(im: $unit) -> Self {
                Self(Complex::from_imag(im.0))
            }
            pub fn real(self) -> $unit {
                $unit(self.0.re)
            }
            pub fn imag(self) -> $unit {
                $unit(self.0.im)
            }
            pub fn magnitude(self) -> $unit {
                $unit(self.0.magnitude())
            }
            pub fn angle(self) -> f64 {
                self.0.angle()
            }
            pub fn to_polar(self) -> ($unit, f64) {
                (self.magnitude(), self.angle())
            }
            pub fn conj(self) -> Self {
                Self(self.0.conj())
            }
            pub fn rotate(self, angle: f64) -> Self {
                Self(self.0.rotate(angle))
            }
            pub fn approx_eq(self, other: Self, tolerance: f64) -> bool {
                self.0.approx_eq(other.0, tolerance)
            }
        }

        impl core::ops::Add for $name {
            type Output = Self;
            fn add(self, rhs: Self) -> Self {
                Self(self.0 + rhs.0)
            }
        }
        impl core::ops::Sub for $name {
            type Output = Self;
            fn sub(self, rhs: Self) -> Self {
                Self(self.0 - rhs.0)
            }
        }
        impl core::ops::Neg for $name {
            type Output = Self;
            fn neg(self) -> Self {
                Self(-self.0)
            }
        }
        impl core::ops::Mul<f64> for $name {
            type Output = Self;
            fn mul(self, rhs: f64) -> Self {
                Self(self.0 * rhs)
            }
        }
        impl core::ops::Mul<$name> for f64 {
            type Output = $name;
            fn mul(self, rhs: $name) -> $name {
                $name(rhs.0 * self)
            }
        }
        impl core::ops::Div<f64> for $name {
            type Output = Self;
            fn div(self, rhs: f64) -> Self {
                Self(self.0 / rhs)
            }
        }
        impl core::ops::Mul<Complex> for $name {
            type Output = Self;
            fn mul(self, rhs: Complex) -> Self {
                Self(self.0 * rhs)
            }
        }
        impl core::ops::Div<Complex> for $name {
            type Output = Self;
            fn div(self, rhs: Complex) -> Self {
                Self(self.0 / rhs)
            }
        }
        impl core::ops::AddAssign for $name {
            fn add_assign(&mut self, rhs: Self) {
                self.0 += rhs.0;
            }
        }
        impl core::ops::SubAssign for $name {
            fn sub_assign(&mut self, rhs: Self) {
                self.0 -= rhs.0;
            }
        }
        impl core::iter::Sum for $name {
            fn sum<I: Iterator<Item = Self>>(iter: I) -> Self {
                iter.fold(Self::ZERO, |a, b| a + b)
            }
        }
    };
}

phasor!(ComplexVolts, Volts);
phasor!(ComplexAmps, Amps);
phasor!(Impedance, Ohms);
phasor!(Admittance, Siemens);

impl core::ops::Mul<Impedance> for ComplexAmps {
    type Output = ComplexVolts;
    fn mul(self, rhs: Impedance) -> ComplexVolts {
        ComplexVolts(self.0 * rhs.0)
    }
}
impl core::ops::Mul<ComplexAmps> for Impedance {
    type Output = ComplexVolts;
    fn mul(self, rhs: ComplexAmps) -> ComplexVolts {
        ComplexVolts(self.0 * rhs.0)
    }
}
impl core::ops::Div<Impedance> for ComplexVolts {
    type Output = ComplexAmps;
    fn div(self, rhs: Impedance) -> ComplexAmps {
        ComplexAmps(self.0 / rhs.0)
    }
}
impl core::ops::Div<ComplexAmps> for ComplexVolts {
    type Output = Impedance;
    fn div(self, rhs: ComplexAmps) -> Impedance {
        Impedance(self.0 / rhs.0)
    }
}
impl core::ops::Mul<Admittance> for ComplexVolts {
    type Output = ComplexAmps;
    fn mul(self, rhs: Admittance) -> ComplexAmps {
        ComplexAmps(self.0 * rhs.0)
    }
}
impl core::ops::Mul<ComplexVolts> for Admittance {
    type Output = ComplexAmps;
    fn mul(self, rhs: ComplexVolts) -> ComplexAmps {
        ComplexAmps(self.0 * rhs.0)
    }
}
impl core::ops::Div<Admittance> for ComplexAmps {
    type Output = ComplexVolts;
    fn div(self, rhs: Admittance) -> ComplexVolts {
        ComplexVolts(self.0 / rhs.0)
    }
}

impl Impedance {
    pub fn resistor(r: Ohms) -> Self {
        Self::new(r, Ohms::ZERO)
    }
    pub fn inductor(l: Henries, f: Hertz) -> Self {
        Self::new(Ohms::ZERO, Ohms(core::f64::consts::TAU * f.0 * l.0))
    }
    pub fn capacitor(c: Farads, f: Hertz) -> Self {
        Self::new(Ohms::ZERO, Ohms(-1.0 / (core::f64::consts::TAU * f.0 * c.0)))
    }
    pub fn resistance(self) -> Ohms {
        self.real()
    }
    pub fn reactance(self) -> Ohms {
        self.imag()
    }
    pub fn admittance(self) -> Admittance {
        Admittance(self.0.recip())
    }
    pub fn parallel(self, other: Self) -> Self {
        Self(self.0 * other.0 / (self.0 + other.0))
    }
    pub fn series(self, other: Self) -> Self {
        self + other
    }
}

impl Admittance {
    pub fn conductance(self) -> Siemens {
        self.real()
    }
    pub fn susceptance(self) -> Siemens {
        self.imag()
    }
    pub fn impedance(self) -> Impedance {
        Impedance(self.0.recip())
    }
}

impl ComplexVolts {
    pub fn power(self, current: ComplexAmps) -> ComplexPower {
        ComplexPower(self.0 * current.0.conj())
    }
}

impl ComplexAmps {
    pub fn power(self, voltage: ComplexVolts) -> ComplexPower {
        voltage.power(self)
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct ComplexPower(pub Complex);

impl ComplexPower {
    pub const ZERO: Self = Self(Complex::ZERO);
    pub fn from_pq(p: Watts, q: Vars) -> Self {
        Self(Complex::new(p.0, q.0))
    }
    pub fn from_apparent_and_factor(s: VoltAmps, power_factor: Ratio, lagging: bool) -> Self {
        let p = s.0 * power_factor.0;
        let q = s.0 * (1.0 - power_factor.0 * power_factor.0).max(0.0).sqrt();
        Self(Complex::new(p, if lagging { q } else { -q }))
    }
    pub fn active(self) -> Watts {
        Watts(self.0.re)
    }
    pub fn reactive(self) -> Vars {
        Vars(self.0.im)
    }
    pub fn apparent(self) -> VoltAmps {
        VoltAmps(self.0.magnitude())
    }
    pub fn power_factor(self) -> Ratio {
        let s = self.0.magnitude();
        if s == 0.0 {
            Ratio(1.0)
        } else {
            Ratio(self.0.re / s)
        }
    }
    pub fn phase_angle(self) -> f64 {
        self.0.angle()
    }
    pub fn approx_eq(self, other: Self, tolerance: f64) -> bool {
        self.0.approx_eq(other.0, tolerance)
    }
}

impl core::ops::Add for ComplexPower {
    type Output = Self;
    fn add(self, rhs: Self) -> Self {
        Self(self.0 + rhs.0)
    }
}
impl core::ops::Sub for ComplexPower {
    type Output = Self;
    fn sub(self, rhs: Self) -> Self {
        Self(self.0 - rhs.0)
    }
}
impl core::ops::Neg for ComplexPower {
    type Output = Self;
    fn neg(self) -> Self {
        Self(-self.0)
    }
}
impl core::ops::Mul<f64> for ComplexPower {
    type Output = Self;
    fn mul(self, rhs: f64) -> Self {
        Self(self.0 * rhs)
    }
}
impl core::ops::Div<f64> for ComplexPower {
    type Output = Self;
    fn div(self, rhs: f64) -> Self {
        Self(self.0 / rhs)
    }
}
impl core::iter::Sum for ComplexPower {
    fn sum<I: Iterator<Item = Self>>(iter: I) -> Self {
        iter.fold(Self::ZERO, |a, b| a + b)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use core::f64::consts::FRAC_PI_2;

    const TOL: f64 = 1e-9;

    #[test]
    fn phasor_construction() {
        let v = ComplexVolts::from_polar(Volts(115.0), FRAC_PI_2);
        assert!(v.real().approx_eq(Volts(0.0), TOL));
        assert!(v.imag().approx_eq(Volts(115.0), TOL));
        assert!(v.magnitude().approx_eq(Volts(115.0), TOL));
        assert!((v.angle() - FRAC_PI_2).abs() < TOL);
        let (m, a) = v.to_polar();
        assert!(m.approx_eq(Volts(115.0), TOL) && (a - FRAC_PI_2).abs() < TOL);
        assert_eq!(
            ComplexVolts::new(Volts(1.0), Volts(2.0)).0,
            Complex::new(1.0, 2.0)
        );
        assert_eq!(ComplexVolts::from_real(Volts(3.0)).0, Complex::new(3.0, 0.0));
        assert_eq!(ComplexVolts::from_imag(Volts(3.0)).0, Complex::new(0.0, 3.0));
    }

    #[test]
    fn phasor_ops() {
        let a = ComplexAmps::new(Amps(1.0), Amps(2.0));
        let b = ComplexAmps::new(Amps(3.0), Amps(-1.0));
        assert_eq!((a + b).0, Complex::new(4.0, 1.0));
        assert_eq!((a - b).0, Complex::new(-2.0, 3.0));
        assert_eq!((-a).0, Complex::new(-1.0, -2.0));
        assert_eq!((a * 2.0).0, Complex::new(2.0, 4.0));
        assert_eq!((2.0 * a).0, Complex::new(2.0, 4.0));
        assert_eq!((a / 2.0).0, Complex::new(0.5, 1.0));
        assert_eq!(a.conj().0, Complex::new(1.0, -2.0));
        assert_eq!((a * Complex::I).0, Complex::new(-2.0, 1.0));
        assert!((a / Complex::I).approx_eq(ComplexAmps(Complex::new(2.0, -1.0)), TOL));
        let mut c = a;
        c += b;
        c -= b;
        assert_eq!(c, a);
        let s: ComplexAmps = [a, b].into_iter().sum();
        assert_eq!(s.0, Complex::new(4.0, 1.0));
        assert!(
            a.rotate(FRAC_PI_2)
                .approx_eq(ComplexAmps(Complex::new(-2.0, 1.0)), TOL)
        );
    }

    #[test]
    fn ohms_law_complex() {
        let i = ComplexAmps::new(Amps(2.0), Amps(1.0));
        let z = Impedance::new(Ohms(10.0), Ohms(5.0));
        let v = i * z;
        assert_eq!(v, z * i);
        assert!(v.approx_eq(ComplexVolts(Complex::new(15.0, 20.0)), TOL));
        assert!((v / z).approx_eq(i, TOL));
        assert!((v / i).approx_eq(z, TOL));
        let y = z.admittance();
        assert!((v * y).approx_eq(i, TOL));
        assert!((y * v).approx_eq(i, TOL));
        assert!((i / y).approx_eq(v, TOL));
        assert!(y.impedance().approx_eq(z, TOL));
    }

    #[test]
    fn impedance_elements() {
        let r = Impedance::resistor(Ohms(10.0));
        assert_eq!(r.resistance(), Ohms(10.0));
        assert_eq!(r.reactance(), Ohms(0.0));
        let l = Impedance::inductor(Henries(0.01), Hertz(400.0));
        assert!(l.reactance().approx_eq(Ohms(core::f64::consts::TAU * 4.0), TOL));
        let c = Impedance::capacitor(Farads(1e-6), Hertz(400.0));
        assert!(c.reactance().0 < 0.0);
        assert!(
            c.reactance()
                .approx_eq(Ohms(-1.0 / (core::f64::consts::TAU * 400.0 * 1e-6)), TOL)
        );
        let total = r.series(l);
        assert!(total.resistance().approx_eq(Ohms(10.0), TOL));
        let p = Impedance::resistor(Ohms(10.0)).parallel(Impedance::resistor(Ohms(10.0)));
        assert!(p.approx_eq(Impedance::resistor(Ohms(5.0)), TOL));
        let y = Admittance::new(Siemens(0.1), Siemens(0.2));
        assert_eq!(y.conductance(), Siemens(0.1));
        assert_eq!(y.susceptance(), Siemens(0.2));
    }

    #[test]
    fn resonance_cancels() {
        let f = Hertz(400.0);
        let l = Henries(0.01);
        let c = Farads(1.0 / (core::f64::consts::TAU * f.0).powi(2) / l.0);
        let z = Impedance::inductor(l, f).series(Impedance::capacitor(c, f));
        assert!(z.magnitude().approx_eq(Ohms(0.0), 1e-9));
    }

    #[test]
    fn complex_power() {
        let v = ComplexVolts::from_polar(Volts(115.0), 0.0);
        let z = Impedance::new(Ohms(4.0), Ohms(3.0));
        let i = v / z;
        let s = v.power(i);
        assert_eq!(s, i.power(v));
        assert!(s.active().approx_eq(Watts(115.0 * 115.0 * 4.0 / 25.0), TOL));
        assert!(s.reactive().approx_eq(Vars(115.0 * 115.0 * 3.0 / 25.0), TOL));
        assert!(s.apparent().approx_eq(VoltAmps(115.0 * 115.0 / 5.0), TOL));
        assert!(s.power_factor().approx_eq(Ratio(0.8), TOL));
        assert!((s.phase_angle() - 3.0_f64.atan2(4.0)).abs() < TOL);
        let direct = v.magnitude() * i.magnitude();
        assert!(direct.approx_eq(Watts(s.apparent().0), TOL));
    }

    #[test]
    fn complex_power_ops() {
        let a = ComplexPower::from_pq(Watts(100.0), Vars(50.0));
        let b = ComplexPower::from_pq(Watts(20.0), Vars(-10.0));
        assert_eq!((a + b).active(), Watts(120.0));
        assert_eq!((a - b).reactive(), Vars(60.0));
        assert_eq!((-a).active(), Watts(-100.0));
        assert_eq!((a * 2.0).active(), Watts(200.0));
        assert_eq!((a / 2.0).reactive(), Vars(25.0));
        let s: ComplexPower = [a, b].into_iter().sum();
        assert_eq!(s.active(), Watts(120.0));
        assert_eq!(ComplexPower::ZERO.power_factor(), Ratio(1.0));
        let lag = ComplexPower::from_apparent_and_factor(VoltAmps(1000.0), Ratio(0.8), true);
        assert!(lag.active().approx_eq(Watts(800.0), TOL));
        assert!(lag.reactive().approx_eq(Vars(600.0), TOL));
        let lead = ComplexPower::from_apparent_and_factor(VoltAmps(1000.0), Ratio(0.8), false);
        assert!(lead.reactive().approx_eq(Vars(-600.0), TOL));
        assert!(a.approx_eq(a, 0.0));
    }
}
