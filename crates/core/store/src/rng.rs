#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SplitMix64 {
    state: u64,
}

impl SplitMix64 {
    pub fn new(seed: u64) -> Self {
        Self { state: seed }
    }

    pub fn next_u64(&mut self) -> u64 {
        self.state = self.state.wrapping_add(0x9E37_79B9_7F4A_7C15);
        let mut z = self.state;
        z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
        z ^ (z >> 31)
    }

    pub fn next_f64(&mut self) -> f64 {
        (self.next_u64() >> 11) as f64 / (1u64 << 53) as f64
    }

    pub fn range(&mut self, lo: f64, hi: f64) -> f64 {
        lo + (hi - lo) * self.next_f64()
    }

    pub fn gaussian(&mut self) -> f64 {
        let u1 = 1.0 - self.next_f64();
        let u2 = self.next_f64();
        (-2.0 * u1.ln()).sqrt() * (std::f64::consts::TAU * u2).cos()
    }
}

pub fn hash_str(s: &str) -> u64 {
    let mut h: u64 = 0xCBF2_9CE4_8422_2325;
    for b in s.bytes() {
        h ^= u64::from(b);
        h = h.wrapping_mul(0x0000_0100_0000_01B3);
    }
    h
}

pub fn seed_for(serial: &str, name: &str) -> u64 {
    let mut r = SplitMix64::new(hash_str(serial) ^ hash_str(name).rotate_left(21));
    r.next_u64()
}

pub fn tolerance_factor(serial: &str, name: &str, sigma: f64, clip_sigmas: f64) -> f64 {
    let mut r = SplitMix64::new(seed_for(serial, name));
    let g = r.gaussian().clamp(-clip_sigmas, clip_sigmas);
    1.0 + sigma * g
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn same_seed_same_sequence() {
        let mut a = SplitMix64::new(42);
        let mut b = SplitMix64::new(42);
        for _ in 0..100 {
            assert_eq!(a.next_u64(), b.next_u64());
        }
    }

    #[test]
    fn known_first_value() {
        let mut r = SplitMix64::new(0);
        assert_eq!(r.next_u64(), 0xE220_A839_7B1D_CDAF);
    }

    #[test]
    fn f64_in_unit_interval() {
        let mut r = SplitMix64::new(7);
        for _ in 0..1000 {
            let x = r.next_f64();
            assert!((0.0..1.0).contains(&x));
        }
    }

    #[test]
    fn range_bounds() {
        let mut r = SplitMix64::new(9);
        for _ in 0..1000 {
            let x = r.range(-2.0, 3.0);
            assert!((-2.0..3.0).contains(&x));
        }
    }

    #[test]
    fn gaussian_mean_and_spread() {
        let mut r = SplitMix64::new(11);
        let n = 20000;
        let xs: Vec<f64> = (0..n).map(|_| r.gaussian()).collect();
        let mean = xs.iter().sum::<f64>() / n as f64;
        let var = xs.iter().map(|x| (x - mean).powi(2)).sum::<f64>() / n as f64;
        assert!(mean.abs() < 0.05);
        assert!((var - 1.0).abs() < 0.08);
    }

    #[test]
    fn tolerance_stable_per_serial() {
        let a = tolerance_factor("SN-1", "gain", 0.05, 3.0);
        let b = tolerance_factor("SN-1", "gain", 0.05, 3.0);
        let c = tolerance_factor("SN-2", "gain", 0.05, 3.0);
        assert_eq!(a.to_bits(), b.to_bits());
        assert_ne!(a.to_bits(), c.to_bits());
    }

    #[test]
    fn tolerance_clipped() {
        for i in 0..500 {
            let f = tolerance_factor(&format!("SN-{i}"), "x", 0.1, 2.0);
            assert!((0.8..=1.2).contains(&f));
        }
    }

    #[test]
    fn hash_differs() {
        assert_ne!(hash_str("a"), hash_str("b"));
    }
}
