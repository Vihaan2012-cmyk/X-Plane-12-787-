use crate::network::LoadKind;
use solver_common::Complex;

pub(crate) fn load_law(kind: LoadKind, v: Complex, ac: bool, frac: f64) -> (Complex, Complex) {
    match kind {
        LoadKind::Impedance { z } => {
            let y = if ac {
                if z.abs() < 1e-9 {
                    Complex::new(1e9, 0.0)
                } else {
                    z.inv()
                }
            } else {
                Complex::new(1.0 / z.re.max(1e-9), 0.0)
            };
            (y, y * v)
        }
        LoadKind::Current {
            amps,
            power_factor,
            v_nominal,
        } => {
            let mag = v.abs();
            if mag == 0.0 {
                return (Complex::ZERO, Complex::ZERO);
            }
            let lim = frac * v_nominal;
            let scale = if mag < lim { mag / lim } else { 1.0 };
            if ac {
                let phi = power_factor.clamp(0.0, 1.0).acos();
                let dir = v.scale(1.0 / mag);
                (Complex::ZERO, dir * Complex::from_polar(amps * scale, -phi))
            } else {
                (Complex::ZERO, Complex::new(amps * scale * v.re.signum(), 0.0))
            }
        }
        LoadKind::Power { p, q, v_nominal } => {
            let vn2 = v_nominal * v_nominal;
            let s = Complex::new(p, q);
            let y0 = if ac {
                s.conj().scale(1.0 / vn2)
            } else {
                Complex::new(p / vn2, 0.0)
            };
            let mag = if ac { v.abs() } else { v.re };
            if mag < frac * v_nominal || mag <= 0.0 {
                let f = frac.max(1e-3);
                (y0, y0.scale(1.0 / (f * f)) * v)
            } else if ac {
                (y0, s.conj() / v.conj())
            } else {
                (y0, Complex::new(p / v.re, 0.0))
            }
        }
    }
}
