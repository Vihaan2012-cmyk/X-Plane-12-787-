macro_rules! quantity {
    ($name:ident) => {
        #[derive(Clone, Copy, Debug, Default, PartialEq, PartialOrd)]
        pub struct $name(pub f64);

        impl $name {
            pub const ZERO: Self = Self(0.0);
            pub const fn new(value: f64) -> Self {
                Self(value)
            }
            pub const fn value(self) -> f64 {
                self.0
            }
            pub fn abs(self) -> Self {
                Self(self.0.abs())
            }
            pub fn min(self, other: Self) -> Self {
                Self(self.0.min(other.0))
            }
            pub fn max(self, other: Self) -> Self {
                Self(self.0.max(other.0))
            }
            pub fn clamp(self, low: Self, high: Self) -> Self {
                Self(self.0.clamp(low.0, high.0))
            }
            pub fn signum(self) -> f64 {
                self.0.signum()
            }
            pub fn is_finite(self) -> bool {
                self.0.is_finite()
            }
            pub fn is_zero(self) -> bool {
                self.0 == 0.0
            }
            pub fn lerp(self, other: Self, t: f64) -> Self {
                Self(self.0 + (other.0 - self.0) * t)
            }
            pub fn approx_eq(self, other: Self, tolerance: f64) -> bool {
                (self.0 - other.0).abs() <= tolerance
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
        impl core::ops::Mul<f64> for $name {
            type Output = Self;
            fn mul(self, rhs: f64) -> Self {
                Self(self.0 * rhs)
            }
        }
        impl core::ops::Mul<$name> for f64 {
            type Output = $name;
            fn mul(self, rhs: $name) -> $name {
                $name(self * rhs.0)
            }
        }
        impl core::ops::Div<f64> for $name {
            type Output = Self;
            fn div(self, rhs: f64) -> Self {
                Self(self.0 / rhs)
            }
        }
        impl core::ops::Div for $name {
            type Output = f64;
            fn div(self, rhs: Self) -> f64 {
                self.0 / rhs.0
            }
        }
        impl core::ops::MulAssign<f64> for $name {
            fn mul_assign(&mut self, rhs: f64) {
                self.0 *= rhs;
            }
        }
        impl core::ops::DivAssign<f64> for $name {
            fn div_assign(&mut self, rhs: f64) {
                self.0 /= rhs;
            }
        }
        impl core::iter::Sum for $name {
            fn sum<I: Iterator<Item = Self>>(iter: I) -> Self {
                iter.fold(Self::ZERO, |a, b| a + b)
            }
        }
        impl<'a> core::iter::Sum<&'a $name> for $name {
            fn sum<I: Iterator<Item = &'a $name>>(iter: I) -> Self {
                iter.fold(Self::ZERO, |a, b| a + *b)
            }
        }
        impl core::fmt::Display for $name {
            fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
                write!(f, "{} {}", self.0, stringify!($name))
            }
        }
        impl core::ops::Mul<crate::quantities::Ratio> for $name {
            type Output = $name;
            fn mul(self, rhs: crate::quantities::Ratio) -> $name {
                $name(self.0 * rhs.0)
            }
        }
    };
}

macro_rules! product {
    ($a:ident, $b:ident => $c:ident) => {
        impl core::ops::Mul<$b> for $a {
            type Output = $c;
            fn mul(self, rhs: $b) -> $c {
                $c(self.0 * rhs.0)
            }
        }
        impl core::ops::Mul<$a> for $b {
            type Output = $c;
            fn mul(self, rhs: $a) -> $c {
                $c(self.0 * rhs.0)
            }
        }
        impl core::ops::Div<$a> for $c {
            type Output = $b;
            fn div(self, rhs: $a) -> $b {
                $b(self.0 / rhs.0)
            }
        }
        impl core::ops::Div<$b> for $c {
            type Output = $a;
            fn div(self, rhs: $b) -> $a {
                $a(self.0 / rhs.0)
            }
        }
    };
}

macro_rules! square {
    ($a:ident => $c:ident) => {
        impl core::ops::Mul<$a> for $a {
            type Output = $c;
            fn mul(self, rhs: $a) -> $c {
                $c(self.0 * rhs.0)
            }
        }
        impl core::ops::Div<$a> for $c {
            type Output = $a;
            fn div(self, rhs: $a) -> $a {
                $a(self.0 / rhs.0)
            }
        }
    };
}

macro_rules! convert {
    ($from:ident => $to:ident, $factor:expr) => {
        impl From<$from> for $to {
            fn from(v: $from) -> $to {
                $to(v.0 * $factor)
            }
        }
    };
}
