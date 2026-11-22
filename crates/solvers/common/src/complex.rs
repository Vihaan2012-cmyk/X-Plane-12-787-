use crate::scalar::Scalar;
use std::ops::{Add, AddAssign, Div, Mul, Neg, Sub, SubAssign};

#[derive(Clone, Copy, Debug, PartialEq, Default)]
pub struct Complex {
    pub re: f64,
    pub im: f64,
}

impl Complex {
    pub const ZERO: Complex = Complex { re: 0.0, im: 0.0 };
    pub const ONE: Complex = Complex { re: 1.0, im: 0.0 };

    pub const fn new(re: f64, im: f64) -> Self {
        Complex { re, im }
    }

    pub fn from_polar(magnitude: f64, angle: f64) -> Self {
        Complex::new(magnitude * angle.cos(), magnitude * angle.sin())
    }

    pub fn conj(self) -> Self {
        Complex::new(self.re, -self.im)
    }

    pub fn abs(self) -> f64 {
        self.re.hypot(self.im)
    }

    pub fn arg(self) -> f64 {
        self.im.atan2(self.re)
    }

    pub fn norm_sqr(self) -> f64 {
        self.re * self.re + self.im * self.im
    }

    pub fn scale(self, k: f64) -> Self {
        Complex::new(self.re * k, self.im * k)
    }

    pub fn inv(self) -> Self {
        Complex::ONE / self
    }

    pub fn is_finite(self) -> bool {
        self.re.is_finite() && self.im.is_finite()
    }
}

impl Add for Complex {
    type Output = Complex;
    fn add(self, o: Complex) -> Complex {
        Complex::new(self.re + o.re, self.im + o.im)
    }
}

impl Sub for Complex {
    type Output = Complex;
    fn sub(self, o: Complex) -> Complex {
        Complex::new(self.re - o.re, self.im - o.im)
    }
}

impl Mul for Complex {
    type Output = Complex;
    fn mul(self, o: Complex) -> Complex {
        Complex::new(self.re * o.re - self.im * o.im, self.re * o.im + self.im * o.re)
    }
}

impl Mul<f64> for Complex {
    type Output = Complex;
    fn mul(self, k: f64) -> Complex {
        self.scale(k)
    }
}

impl Div for Complex {
    type Output = Complex;
    fn div(self, o: Complex) -> Complex {
        let d = o.norm_sqr();
        Complex::new(
            (self.re * o.re + self.im * o.im) / d,
            (self.im * o.re - self.re * o.im) / d,
        )
    }
}

impl Neg for Complex {
    type Output = Complex;
    fn neg(self) -> Complex {
        Complex::new(-self.re, -self.im)
    }
}

impl AddAssign for Complex {
    fn add_assign(&mut self, o: Complex) {
        self.re += o.re;
        self.im += o.im;
    }
}

impl SubAssign for Complex {
    fn sub_assign(&mut self, o: Complex) {
        self.re -= o.re;
        self.im -= o.im;
    }
}

impl Scalar for Complex {
    fn zero() -> Self {
        Complex::ZERO
    }
    fn one() -> Self {
        Complex::ONE
    }
    fn magnitude(self) -> f64 {
        self.abs()
    }
    fn from_parts(re: f64, im: f64) -> Self {
        Complex::new(re, im)
    }
    fn real(self) -> f64 {
        self.re
    }
    fn imag(self) -> f64 {
        self.im
    }
}
