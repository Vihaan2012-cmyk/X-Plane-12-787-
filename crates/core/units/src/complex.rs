#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Complex {
    pub re: f64,
    pub im: f64,
}

impl Complex {
    pub const ZERO: Self = Self { re: 0.0, im: 0.0 };
    pub const ONE: Self = Self { re: 1.0, im: 0.0 };
    pub const I: Self = Self { re: 0.0, im: 1.0 };

    pub const fn new(re: f64, im: f64) -> Self {
        Self { re, im }
    }
    pub fn from_polar(magnitude: f64, angle: f64) -> Self {
        Self {
            re: magnitude * angle.cos(),
            im: magnitude * angle.sin(),
        }
    }
    pub fn from_real(re: f64) -> Self {
        Self { re, im: 0.0 }
    }
    pub fn from_imag(im: f64) -> Self {
        Self { re: 0.0, im }
    }
    pub fn magnitude(self) -> f64 {
        self.re.hypot(self.im)
    }
    pub fn norm_sqr(self) -> f64 {
        self.re * self.re + self.im * self.im
    }
    pub fn angle(self) -> f64 {
        self.im.atan2(self.re)
    }
    pub fn to_polar(self) -> (f64, f64) {
        (self.magnitude(), self.angle())
    }
    pub fn conj(self) -> Self {
        Self {
            re: self.re,
            im: -self.im,
        }
    }
    pub fn recip(self) -> Self {
        let n = self.norm_sqr();
        Self {
            re: self.re / n,
            im: -self.im / n,
        }
    }
    pub fn exp(self) -> Self {
        Self::from_polar(self.re.exp(), self.im)
    }
    pub fn sqrt(self) -> Self {
        let (m, a) = self.to_polar();
        Self::from_polar(m.sqrt(), a / 2.0)
    }
    pub fn powi(self, n: i32) -> Self {
        let (m, a) = self.to_polar();
        Self::from_polar(m.powi(n), a * f64::from(n))
    }
    pub fn rotate(self, angle: f64) -> Self {
        self * Self::from_polar(1.0, angle)
    }
    pub fn is_finite(self) -> bool {
        self.re.is_finite() && self.im.is_finite()
    }
    pub fn approx_eq(self, other: Self, tolerance: f64) -> bool {
        (self - other).magnitude() <= tolerance
    }
}

impl core::ops::Add for Complex {
    type Output = Self;
    fn add(self, rhs: Self) -> Self {
        Self {
            re: self.re + rhs.re,
            im: self.im + rhs.im,
        }
    }
}
impl core::ops::Sub for Complex {
    type Output = Self;
    fn sub(self, rhs: Self) -> Self {
        Self {
            re: self.re - rhs.re,
            im: self.im - rhs.im,
        }
    }
}
impl core::ops::Mul for Complex {
    type Output = Self;
    fn mul(self, rhs: Self) -> Self {
        Self {
            re: self.re * rhs.re - self.im * rhs.im,
            im: self.re * rhs.im + self.im * rhs.re,
        }
    }
}
impl core::ops::Div for Complex {
    type Output = Self;
    fn div(self, rhs: Self) -> Self {
        let n = rhs.norm_sqr();
        Self {
            re: (self.re * rhs.re + self.im * rhs.im) / n,
            im: (self.im * rhs.re - self.re * rhs.im) / n,
        }
    }
}
impl core::ops::Neg for Complex {
    type Output = Self;
    fn neg(self) -> Self {
        Self {
            re: -self.re,
            im: -self.im,
        }
    }
}
impl core::ops::Add<f64> for Complex {
    type Output = Self;
    fn add(self, rhs: f64) -> Self {
        Self {
            re: self.re + rhs,
            im: self.im,
        }
    }
}
impl core::ops::Sub<f64> for Complex {
    type Output = Self;
    fn sub(self, rhs: f64) -> Self {
        Self {
            re: self.re - rhs,
            im: self.im,
        }
    }
}
impl core::ops::Mul<f64> for Complex {
    type Output = Self;
    fn mul(self, rhs: f64) -> Self {
        Self {
            re: self.re * rhs,
            im: self.im * rhs,
        }
    }
}
impl core::ops::Div<f64> for Complex {
    type Output = Self;
    fn div(self, rhs: f64) -> Self {
        Self {
            re: self.re / rhs,
            im: self.im / rhs,
        }
    }
}
impl core::ops::Add<Complex> for f64 {
    type Output = Complex;
    fn add(self, rhs: Complex) -> Complex {
        Complex {
            re: self + rhs.re,
            im: rhs.im,
        }
    }
}
impl core::ops::Sub<Complex> for f64 {
    type Output = Complex;
    fn sub(self, rhs: Complex) -> Complex {
        Complex {
            re: self - rhs.re,
            im: -rhs.im,
        }
    }
}
impl core::ops::Mul<Complex> for f64 {
    type Output = Complex;
    fn mul(self, rhs: Complex) -> Complex {
        Complex {
            re: self * rhs.re,
            im: self * rhs.im,
        }
    }
}
impl core::ops::Div<Complex> for f64 {
    type Output = Complex;
    fn div(self, rhs: Complex) -> Complex {
        Complex::from_real(self) / rhs
    }
}
impl core::ops::AddAssign for Complex {
    fn add_assign(&mut self, rhs: Self) {
        *self = *self + rhs;
    }
}
impl core::ops::SubAssign for Complex {
    fn sub_assign(&mut self, rhs: Self) {
        *self = *self - rhs;
    }
}
impl core::ops::MulAssign for Complex {
    fn mul_assign(&mut self, rhs: Self) {
        *self = *self * rhs;
    }
}
impl core::ops::DivAssign for Complex {
    fn div_assign(&mut self, rhs: Self) {
        *self = *self / rhs;
    }
}
impl core::ops::MulAssign<f64> for Complex {
    fn mul_assign(&mut self, rhs: f64) {
        *self = *self * rhs;
    }
}
impl core::ops::DivAssign<f64> for Complex {
    fn div_assign(&mut self, rhs: f64) {
        *self = *self / rhs;
    }
}
impl core::iter::Sum for Complex {
    fn sum<I: Iterator<Item = Self>>(iter: I) -> Self {
        iter.fold(Self::ZERO, |a, b| a + b)
    }
}
impl From<f64> for Complex {
    fn from(re: f64) -> Self {
        Self::from_real(re)
    }
}
impl From<(f64, f64)> for Complex {
    fn from((re, im): (f64, f64)) -> Self {
        Self { re, im }
    }
}
impl core::fmt::Display for Complex {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        if self.im < 0.0 {
            write!(f, "{}-j{}", self.re, -self.im)
        } else {
            write!(f, "{}+j{}", self.re, self.im)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use core::f64::consts::{FRAC_PI_2, FRAC_PI_4, PI};

    const TOL: f64 = 1e-12;

    #[test]
    fn rectangular_and_polar() {
        let z = Complex::new(3.0, 4.0);
        assert!((z.magnitude() - 5.0).abs() < TOL);
        assert!((z.norm_sqr() - 25.0).abs() < TOL);
        assert!((z.angle() - 4.0_f64.atan2(3.0)).abs() < TOL);
        let p = Complex::from_polar(2.0, FRAC_PI_2);
        assert!(p.approx_eq(Complex::new(0.0, 2.0), TOL));
        let (m, a) = Complex::from_polar(7.0, 0.3).to_polar();
        assert!((m - 7.0).abs() < TOL && (a - 0.3).abs() < TOL);
    }

    #[test]
    fn constants_and_constructors() {
        assert_eq!(Complex::ZERO, Complex::new(0.0, 0.0));
        assert_eq!(Complex::ONE, Complex::new(1.0, 0.0));
        assert_eq!(Complex::I * Complex::I, Complex::new(-1.0, 0.0));
        assert_eq!(Complex::from_real(2.0), Complex::from(2.0));
        assert_eq!(Complex::from_imag(2.0), Complex::new(0.0, 2.0));
        assert_eq!(Complex::from((1.0, 2.0)), Complex::new(1.0, 2.0));
        assert_eq!(Complex::default(), Complex::ZERO);
    }

    #[test]
    fn conj_recip_neg() {
        let z = Complex::new(1.0, -2.0);
        assert_eq!(z.conj(), Complex::new(1.0, 2.0));
        assert_eq!(-z, Complex::new(-1.0, 2.0));
        assert!((z * z.recip()).approx_eq(Complex::ONE, TOL));
        assert!((z * z.conj()).approx_eq(Complex::from_real(5.0), TOL));
    }

    #[test]
    fn complex_arithmetic() {
        let a = Complex::new(1.0, 2.0);
        let b = Complex::new(3.0, -1.0);
        assert_eq!(a + b, Complex::new(4.0, 1.0));
        assert_eq!(a - b, Complex::new(-2.0, 3.0));
        assert_eq!(a * b, Complex::new(5.0, 5.0));
        assert!(((a * b) / b).approx_eq(a, TOL));
        let mut c = a;
        c += b;
        assert_eq!(c, Complex::new(4.0, 1.0));
        c -= b;
        assert_eq!(c, a);
        c *= b;
        assert_eq!(c, Complex::new(5.0, 5.0));
        c /= b;
        assert!(c.approx_eq(a, TOL));
    }

    #[test]
    fn scalar_arithmetic() {
        let a = Complex::new(1.0, 2.0);
        assert_eq!(a + 1.0, Complex::new(2.0, 2.0));
        assert_eq!(a - 1.0, Complex::new(0.0, 2.0));
        assert_eq!(a * 2.0, Complex::new(2.0, 4.0));
        assert_eq!(a / 2.0, Complex::new(0.5, 1.0));
        assert_eq!(1.0 + a, Complex::new(2.0, 2.0));
        assert_eq!(1.0 - a, Complex::new(0.0, -2.0));
        assert_eq!(2.0 * a, Complex::new(2.0, 4.0));
        assert!((1.0 / a).approx_eq(a.recip(), TOL));
        let mut c = a;
        c *= 3.0;
        assert_eq!(c, Complex::new(3.0, 6.0));
        c /= 3.0;
        assert_eq!(c, a);
    }

    #[test]
    fn exp_sqrt_powi_rotate() {
        assert!(
            Complex::new(0.0, PI)
                .exp()
                .approx_eq(Complex::new(-1.0, 0.0), TOL)
        );
        assert!(
            Complex::new(-4.0, 0.0)
                .sqrt()
                .approx_eq(Complex::new(0.0, 2.0), TOL)
        );
        let z = Complex::new(1.0, 1.0);
        assert!(z.powi(2).approx_eq(Complex::new(0.0, 2.0), TOL));
        assert!(z.powi(-1).approx_eq(z.recip(), TOL));
        assert!(
            Complex::ONE
                .rotate(FRAC_PI_4)
                .approx_eq(Complex::from_polar(1.0, FRAC_PI_4), TOL)
        );
    }

    #[test]
    fn sum_and_display_and_finite() {
        let v = [Complex::new(1.0, 1.0), Complex::new(2.0, -3.0)];
        let s: Complex = v.iter().copied().sum();
        assert_eq!(s, Complex::new(3.0, -2.0));
        assert_eq!(format!("{}", Complex::new(1.0, 2.0)), "1+j2");
        assert_eq!(format!("{}", Complex::new(1.0, -2.0)), "1-j2");
        assert!(Complex::new(1.0, 2.0).is_finite());
        assert!(!Complex::new(f64::NAN, 2.0).is_finite());
    }
}
