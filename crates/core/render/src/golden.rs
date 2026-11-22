use crate::canvas::Canvas;

pub fn hash_bytes(data: &[u8]) -> u64 {
    let mut h = 0xcbf2_9ce4_8422_2325u64;
    for &b in data {
        h ^= b as u64;
        h = h.wrapping_mul(0x0000_0100_0000_01b3);
    }
    h
}

pub fn hash_canvas(canvas: &Canvas) -> u64 {
    let mut h = hash_bytes(&(canvas.width() as u64).to_le_bytes());
    h ^= hash_bytes(&(canvas.height() as u64).to_le_bytes()).rotate_left(17);
    h ^ hash_bytes(canvas.rgba()).rotate_left(1)
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct GoldenError {
    pub expected: u64,
    pub actual: u64,
}

impl std::fmt::Display for GoldenError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "golden mismatch: expected {:#018x}, got {:#018x}",
            self.expected, self.actual
        )
    }
}

impl std::error::Error for GoldenError {}

pub fn check_golden(canvas: &Canvas, expected: u64) -> Result<(), GoldenError> {
    let actual = hash_canvas(canvas);
    if actual == expected {
        Ok(())
    } else {
        Err(GoldenError { expected, actual })
    }
}
