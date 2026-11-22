#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Encoding {
    Bnr { bits: u32, range: f64, signed: bool },
    Discrete,
    Raw { bits: u32 },
}

#[derive(Clone, Debug, PartialEq)]
pub struct Param {
    pub name: String,
    pub offset_bits: u32,
    pub enc: Encoding,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Dataset {
    pub name: String,
    pub version: u16,
    pub params: Vec<Param>,
    pub bytes: usize,
}

pub const STATUS_BYTES: usize = 8;
pub const TS_OFFSET: usize = 4;
pub const FS_NO_DATA: u8 = 0;
pub const FS_NORMAL: u8 = 3;
pub const FS_NCD: u8 = 1;
pub const FS_TEST: u8 = 2;

fn put_bits(buf: &mut [u8], offset: u32, bits: u32, value: u64) {
    for i in 0..bits {
        let bit = (value >> i) & 1;
        let pos = offset + i;
        let (byte, sh) = ((pos / 8) as usize, pos % 8);
        if byte >= buf.len() {
            return;
        }
        if bit == 1 {
            buf[byte] |= 1 << sh;
        } else {
            buf[byte] &= !(1 << sh);
        }
    }
}

fn get_bits(buf: &[u8], offset: u32, bits: u32) -> u64 {
    let mut v = 0u64;
    for i in 0..bits {
        let pos = offset + i;
        let (byte, sh) = ((pos / 8) as usize, pos % 8);
        if byte < buf.len() && (buf[byte] >> sh) & 1 == 1 {
            v |= 1 << i;
        }
    }
    v
}

impl Encoding {
    pub fn bits(&self) -> u32 {
        match *self {
            Encoding::Bnr { bits, .. } | Encoding::Raw { bits } => bits,
            Encoding::Discrete => 1,
        }
    }

    pub fn resolution(&self) -> f64 {
        match *self {
            Encoding::Bnr { bits, range, signed } => {
                let mag = if signed { bits - 1 } else { bits };
                range / (1u64 << mag) as f64
            }
            _ => 1.0,
        }
    }

    pub fn encode(&self, value: f64) -> u64 {
        match *self {
            Encoding::Bnr { bits, range, signed } => {
                let res = self.resolution();
                let max = range - res;
                let min = if signed { -range } else { 0.0 };
                let q = (value.clamp(min, max) / res).round() as i64;
                let mask = if bits >= 64 { u64::MAX } else { (1u64 << bits) - 1 };
                (q as u64) & mask
            }
            Encoding::Discrete => u64::from(value != 0.0),
            Encoding::Raw { bits } => {
                let mask = if bits >= 64 { u64::MAX } else { (1u64 << bits) - 1 };
                (value.max(0.0) as u64) & mask
            }
        }
    }

    pub fn decode(&self, raw: u64) -> f64 {
        match *self {
            Encoding::Bnr { bits, signed, .. } => {
                let res = self.resolution();
                let v = if signed && bits < 64 && (raw >> (bits - 1)) & 1 == 1 {
                    raw as i64 - (1i64 << bits)
                } else {
                    raw as i64
                };
                v as f64 * res
            }
            Encoding::Discrete | Encoding::Raw { .. } => raw as f64,
        }
    }
}

impl Dataset {
    pub fn new(name: &str, version: u16, defs: &[(&str, Encoding)]) -> Dataset {
        let mut off = (STATUS_BYTES * 8) as u32;
        let mut params = Vec::new();
        for (n, e) in defs {
            params.push(Param {
                name: n.to_string(),
                offset_bits: off,
                enc: *e,
            });
            off += e.bits();
        }
        Dataset {
            name: name.to_string(),
            version,
            params,
            bytes: (off as usize).div_ceil(8),
        }
    }

    pub fn encode(&self, values: &[(&str, f64)], status: u8) -> Vec<u8> {
        self.encode_at(values, status, 0)
    }

    pub fn stamp(buf: &mut [u8], ts_ms: u32) {
        if buf.len() >= TS_OFFSET + 4 {
            buf[TS_OFFSET..TS_OFFSET + 4].copy_from_slice(&ts_ms.to_le_bytes());
        }
    }

    pub fn timestamp_of(buf: &[u8]) -> Option<u32> {
        let b = buf.get(TS_OFFSET..TS_OFFSET + 4)?;
        Some(u32::from_le_bytes([b[0], b[1], b[2], b[3]]))
    }

    pub fn encode_at(&self, values: &[(&str, f64)], status: u8, ts_ms: u32) -> Vec<u8> {
        let mut buf = vec![0u8; self.bytes];
        Dataset::stamp(&mut buf, ts_ms);
        buf[0] = status;
        buf[1] = (self.version & 0xFF) as u8;
        buf[2] = (self.version >> 8) as u8;
        for p in &self.params {
            if let Some((_, v)) = values.iter().find(|(n, _)| *n == p.name) {
                put_bits(&mut buf, p.offset_bits, p.enc.bits(), p.enc.encode(*v));
            }
        }
        buf
    }

    pub fn status(buf: &[u8]) -> u8 {
        buf.first().copied().unwrap_or(FS_NO_DATA)
    }

    pub fn version_of(buf: &[u8]) -> u16 {
        if buf.len() < 3 {
            return 0;
        }
        u16::from(buf[1]) | (u16::from(buf[2]) << 8)
    }

    pub fn decode(&self, buf: &[u8], name: &str) -> Option<f64> {
        let p = self.params.iter().find(|p| p.name == name)?;
        Some(p.enc.decode(get_bits(buf, p.offset_bits, p.enc.bits())))
    }
}

pub fn crc32(data: &[u8]) -> u32 {
    let mut crc = 0xFFFF_FFFFu32;
    for &b in data {
        crc ^= u32::from(b);
        for _ in 0..8 {
            let m = (crc & 1).wrapping_neg();
            crc = (crc >> 1) ^ (0xEDB8_8320 & m);
        }
    }
    !crc
}
