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
        mix(self.state)
    }

    pub fn next_f64(&mut self) -> f64 {
        to_unit(self.next_u64())
    }
}

fn mix(x: u64) -> u64 {
    let mut z = x;
    z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
    z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
    z ^ (z >> 31)
}

fn to_unit(v: u64) -> f64 {
    (v >> 11) as f64 / (1u64 << 53) as f64
}

pub fn hash_at(seed: u64, index: u64) -> f64 {
    to_unit(mix(seed.wrapping_add(index.wrapping_mul(0x9E37_79B9_7F4A_7C15))))
}

pub fn hash_str(s: &str) -> u64 {
    let mut h: u64 = 0xCBF2_9CE4_8422_2325;
    for b in s.bytes() {
        h ^= u64::from(b);
        h = h.wrapping_mul(0x0000_0100_0000_01B3);
    }
    h
}

pub fn seed_for(serial: &str, mode: &str) -> u64 {
    mix(hash_str(serial) ^ hash_str(mode).rotate_left(21))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn known_first_value() {
        assert_eq!(SplitMix64::new(0).next_u64(), 0xE220_A839_7B1D_CDAF);
    }

    #[test]
    fn unit_interval() {
        let mut r = SplitMix64::new(5);
        for _ in 0..1000 {
            assert!((0.0..1.0).contains(&r.next_f64()));
        }
    }

    #[test]
    fn hash_at_stable_and_varied() {
        assert_eq!(hash_at(3, 9).to_bits(), hash_at(3, 9).to_bits());
        assert_ne!(hash_at(3, 9).to_bits(), hash_at(3, 10).to_bits());
        assert_ne!(seed_for("a", "m"), seed_for("b", "m"));
        assert_ne!(seed_for("a", "m"), seed_for("a", "n"));
    }
}
