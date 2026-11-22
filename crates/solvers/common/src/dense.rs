use crate::lu::LuError;
use crate::scalar::Scalar;

pub fn dense_solve<T: Scalar>(a: &[T], n: usize, b: &[T]) -> Result<Vec<T>, LuError> {
    let mut m = a.to_vec();
    let mut x = b.to_vec();
    for k in 0..n {
        let mut p = k;
        let mut best = m[k * n + k].magnitude();
        for r in k + 1..n {
            let v = m[r * n + k].magnitude();
            if v > best {
                best = v;
                p = r;
            }
        }
        if best <= 1e-300 || !best.is_finite() {
            return Err(LuError::Singular { step: k });
        }
        if p != k {
            for c in 0..n {
                m.swap(k * n + c, p * n + c);
            }
            x.swap(k, p);
        }
        let piv = m[k * n + k];
        for r in k + 1..n {
            let f = m[r * n + k] / piv;
            if f == T::zero() {
                continue;
            }
            for c in k..n {
                let t = m[k * n + c];
                m[r * n + c] -= f * t;
            }
            let t = x[k];
            x[r] -= f * t;
        }
    }
    for k in (0..n).rev() {
        let mut s = x[k];
        for c in k + 1..n {
            s -= m[k * n + c] * x[c];
        }
        x[k] = s / m[k * n + k];
    }
    Ok(x)
}
