use std::fmt::Debug;
use std::ops::{Add, AddAssign, Div, Mul, Neg, Sub, SubAssign};

pub trait Scalar:
    Copy
    + PartialEq
    + Debug
    + Add<Output = Self>
    + Sub<Output = Self>
    + Mul<Output = Self>
    + Div<Output = Self>
    + Neg<Output = Self>
    + AddAssign
    + SubAssign
{
    fn zero() -> Self;
    fn one() -> Self;
    fn magnitude(self) -> f64;
    fn from_parts(re: f64, im: f64) -> Self;
    fn real(self) -> f64;
    fn imag(self) -> f64;
}

impl Scalar for f64 {
    fn zero() -> Self {
        0.0
    }
    fn one() -> Self {
        1.0
    }
    fn magnitude(self) -> f64 {
        self.abs()
    }
    fn from_parts(re: f64, _im: f64) -> Self {
        re
    }
    fn real(self) -> f64 {
        self
    }
    fn imag(self) -> f64 {
        0.0
    }
}
