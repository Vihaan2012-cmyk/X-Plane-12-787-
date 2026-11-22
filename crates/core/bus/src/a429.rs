use std::collections::BTreeMap;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Word(pub u32);

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BnrSsm {
    FailureWarning,
    NoComputedData,
    FunctionalTest,
    NormalOperation,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BcdSsm {
    Plus,
    NoComputedData,
    FunctionalTest,
    Minus,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DiscreteSsm {
    NormalOperation,
    NoComputedData,
    FunctionalTest,
    FailureWarning,
}

impl BnrSsm {
    pub fn bits(self) -> u32 {
        match self {
            BnrSsm::FailureWarning => 0,
            BnrSsm::NoComputedData => 1,
            BnrSsm::FunctionalTest => 2,
            BnrSsm::NormalOperation => 3,
        }
    }
    pub fn from_bits(b: u32) -> Self {
        match b & 3 {
            0 => BnrSsm::FailureWarning,
            1 => BnrSsm::NoComputedData,
            2 => BnrSsm::FunctionalTest,
            _ => BnrSsm::NormalOperation,
        }
    }
}

impl BcdSsm {
    pub fn bits(self) -> u32 {
        match self {
            BcdSsm::Plus => 0,
            BcdSsm::NoComputedData => 1,
            BcdSsm::FunctionalTest => 2,
            BcdSsm::Minus => 3,
        }
    }
    pub fn from_bits(b: u32) -> Self {
        match b & 3 {
            0 => BcdSsm::Plus,
            1 => BcdSsm::NoComputedData,
            2 => BcdSsm::FunctionalTest,
            _ => BcdSsm::Minus,
        }
    }
}

impl DiscreteSsm {
    pub fn bits(self) -> u32 {
        match self {
            DiscreteSsm::NormalOperation => 0,
            DiscreteSsm::NoComputedData => 1,
            DiscreteSsm::FunctionalTest => 2,
            DiscreteSsm::FailureWarning => 3,
        }
    }
    pub fn from_bits(b: u32) -> Self {
        match b & 3 {
            0 => DiscreteSsm::NormalOperation,
            1 => DiscreteSsm::NoComputedData,
            2 => DiscreteSsm::FunctionalTest,
            _ => DiscreteSsm::FailureWarning,
        }
    }
}

pub const DATA_BITS: u32 = 19;
pub const DATA_MASK: u32 = (1 << DATA_BITS) - 1;

impl Word {
    pub fn pack(label_octal: u8, sdi: u8, data: u32, ssm: u8) -> Word {
        let mut w = u32::from(label_octal.reverse_bits());
        w |= (u32::from(sdi) & 3) << 8;
        w |= (data & DATA_MASK) << 10;
        w |= (u32::from(ssm) & 3) << 29;
        if w.count_ones().is_multiple_of(2) {
            w |= 1 << 31;
        }
        Word(w)
    }
    pub fn label(self) -> u8 {
        (self.0 & 0xFF) as u8
    }
    pub fn label_octal(self) -> u8 {
        self.label().reverse_bits()
    }
    pub fn sdi(self) -> u8 {
        ((self.0 >> 8) & 3) as u8
    }
    pub fn data(self) -> u32 {
        (self.0 >> 10) & DATA_MASK
    }
    pub fn ssm_bits(self) -> u32 {
        (self.0 >> 29) & 3
    }
    pub fn parity_ok(self) -> bool {
        self.0.count_ones() % 2 == 1
    }
    pub fn corrupt(self, bit: u32) -> Word {
        Word(self.0 ^ (1 << (bit & 31)))
    }
    pub fn bnr_ssm(self) -> BnrSsm {
        BnrSsm::from_bits(self.ssm_bits())
    }
    pub fn bcd_ssm(self) -> BcdSsm {
        BcdSsm::from_bits(self.ssm_bits())
    }
    pub fn discrete_ssm(self) -> DiscreteSsm {
        DiscreteSsm::from_bits(self.ssm_bits())
    }
    pub fn discrete_bit(self, bit: u32) -> bool {
        (self.data() >> (bit - 11)) & 1 == 1
    }
    pub fn with_discrete_bit(self, bit: u32, on: bool) -> Word {
        let mut data = self.data();
        let m = 1 << (bit - 11);
        if on {
            data |= m;
        } else {
            data &= !m;
        }
        Word::pack(self.label_octal(), self.sdi(), data, self.ssm_bits() as u8)
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct BnrFormat {
    pub total_bits: u32,
    pub range: f64,
}

impl BnrFormat {
    pub fn new(total_bits: u32, range: f64) -> Self {
        BnrFormat {
            total_bits: total_bits.clamp(2, DATA_BITS),
            range,
        }
    }
    pub fn resolution(&self) -> f64 {
        self.range / f64::from(1u32 << (self.total_bits - 1))
    }
    pub fn encode(&self, value: f64) -> u32 {
        let half = i64::from(1u32 << (self.total_bits - 1));
        let counts = (value / self.resolution()).round();
        let counts = if counts.is_nan() { 0.0 } else { counts };
        let counts = (counts as i64).clamp(-half, half - 1);
        let mask = (1u64 << self.total_bits) - 1;
        (((counts as u64) & mask) as u32) << (DATA_BITS - self.total_bits)
    }
    pub fn decode(&self, data: u32) -> f64 {
        let raw = (data & DATA_MASK) >> (DATA_BITS - self.total_bits);
        let sign = 1u32 << (self.total_bits - 1);
        let counts = if raw & sign != 0 {
            i64::from(raw) - i64::from(sign << 1)
        } else {
            i64::from(raw)
        };
        counts as f64 * self.resolution()
    }
    pub fn pack_word(&self, label: u8, sdi: u8, value: f64, ssm: BnrSsm) -> Word {
        Word::pack(label, sdi, self.encode(value), ssm.bits() as u8)
    }
    pub fn read_word(&self, w: Word) -> Option<f64> {
        if w.bnr_ssm() == BnrSsm::NormalOperation && w.parity_ok() {
            Some(self.decode(w.data()))
        } else {
            None
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct BcdFormat {
    pub digits: u32,
    pub resolution: f64,
}

impl BcdFormat {
    pub fn new(digits: u32, resolution: f64) -> Self {
        BcdFormat {
            digits: digits.clamp(1, 5),
            resolution,
        }
    }
    pub fn max_counts(&self) -> u32 {
        let top = if self.digits == 5 { 7 } else { 9 };
        let mut v = top;
        for _ in 1..self.digits {
            v = v * 10 + 9;
        }
        v
    }
    pub fn encode(&self, value: f64) -> (u32, BcdSsm) {
        let neg = value < 0.0;
        let counts = (value.abs() / self.resolution).round() as u32;
        let mut counts = counts.min(self.max_counts());
        let mut data = 0u32;
        for i in 0..self.digits {
            data |= (counts % 10) << (4 * i);
            counts /= 10;
        }
        (data, if neg { BcdSsm::Minus } else { BcdSsm::Plus })
    }
    pub fn decode(&self, data: u32, ssm: BcdSsm) -> Option<f64> {
        let mut counts = 0u32;
        for i in (0..self.digits).rev() {
            let d = (data >> (4 * i)) & 0xF;
            if d > 9 {
                return None;
            }
            counts = counts * 10 + d;
        }
        let v = f64::from(counts) * self.resolution;
        match ssm {
            BcdSsm::Plus => Some(v),
            BcdSsm::Minus => Some(-v),
            _ => None,
        }
    }
    pub fn pack_word(&self, label: u8, sdi: u8, value: f64) -> Word {
        let (d, s) = self.encode(value);
        Word::pack(label, sdi, d, s.bits() as u8)
    }
    pub fn read_word(&self, w: Word) -> Option<f64> {
        if !w.parity_ok() {
            return None;
        }
        self.decode(w.data(), w.bcd_ssm())
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Speed {
    Low,
    High,
}

pub const GAP_BITS: u64 = 4;
pub const WORD_BITS: u64 = 32;

impl Speed {
    pub fn bits_per_second(self) -> u64 {
        match self {
            Speed::Low => 12_500,
            Speed::High => 100_000,
        }
    }
    pub fn bit_time_us(self) -> u64 {
        1_000_000 / self.bits_per_second()
    }
    pub fn word_time_us(self) -> u64 {
        WORD_BITS * self.bit_time_us()
    }
    pub fn slot_time_us(self) -> u64 {
        (WORD_BITS + GAP_BITS) * self.bit_time_us()
    }
    pub fn max_words_per_second(self) -> u64 {
        1_000_000 / self.slot_time_us()
    }
}

#[derive(Clone, Debug)]
struct Scheduled {
    label: u8,
    sdi: u8,
    interval_us: u64,
    next_due_us: u64,
    word: Option<Word>,
}

#[derive(Clone, Debug)]
pub struct Transmitter {
    speed: Speed,
    entries: Vec<Scheduled>,
    line_free_us: u64,
}

impl Transmitter {
    pub fn new(speed: Speed) -> Self {
        Transmitter {
            speed,
            entries: Vec::new(),
            line_free_us: 0,
        }
    }
    pub fn add_label(&mut self, label: u8, sdi: u8, interval_us: u64, offset_us: u64) {
        self.entries.push(Scheduled {
            label,
            sdi,
            interval_us: interval_us.max(self.speed.slot_time_us()),
            next_due_us: offset_us,
            word: None,
        });
    }
    pub fn set_word(&mut self, label: u8, sdi: u8, word: Word) {
        for e in &mut self.entries {
            if e.label == label && e.sdi == sdi {
                e.word = Some(word);
            }
        }
    }
    pub fn utilisation(&self) -> f64 {
        let slot = self.speed.slot_time_us() as f64;
        self.entries.iter().map(|e| slot / e.interval_us as f64).sum()
    }
    pub fn poll(&mut self, now_us: u64) -> Vec<(u64, u64, Word)> {
        let mut out = Vec::new();
        loop {
            let mut best: Option<usize> = None;
            for (i, e) in self.entries.iter().enumerate() {
                if e.word.is_some() && e.next_due_us <= now_us {
                    match best {
                        Some(b) if self.entries[b].next_due_us <= e.next_due_us => {}
                        _ => best = Some(i),
                    }
                }
            }
            let Some(i) = best else { break };
            let start = self.line_free_us.max(self.entries[i].next_due_us);
            if start > now_us {
                break;
            }
            let end = start + self.speed.word_time_us();
            self.line_free_us = start + self.speed.slot_time_us();
            let e = &mut self.entries[i];
            out.push((start, end, e.word.unwrap_or(Word(0))));
            e.next_due_us += e.interval_us;
            while e.next_due_us <= now_us.max(start) {
                e.next_due_us += e.interval_us;
            }
        }
        out
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Freshness {
    Fresh,
    Stale,
    Never,
}

#[derive(Clone, Debug)]
pub struct WordReceiver {
    timeout_us: u64,
    last: BTreeMap<(u8, u8), (u64, Word)>,
    parity_errors: u64,
}

impl WordReceiver {
    pub fn new(timeout_us: u64) -> Self {
        WordReceiver {
            timeout_us,
            last: BTreeMap::new(),
            parity_errors: 0,
        }
    }
    pub fn receive(&mut self, time_us: u64, word: Word) -> bool {
        if !word.parity_ok() {
            self.parity_errors += 1;
            return false;
        }
        self.last
            .insert((word.label_octal(), word.sdi()), (time_us, word));
        true
    }
    pub fn parity_errors(&self) -> u64 {
        self.parity_errors
    }
    pub fn freshness(&self, label: u8, sdi: u8, now_us: u64) -> Freshness {
        match self.last.get(&(label, sdi)) {
            None => Freshness::Never,
            Some((t, _)) if now_us.saturating_sub(*t) <= self.timeout_us => Freshness::Fresh,
            Some(_) => Freshness::Stale,
        }
    }
    pub fn get(&self, label: u8, sdi: u8, now_us: u64) -> Option<Word> {
        match (self.freshness(label, sdi, now_us), self.last.get(&(label, sdi))) {
            (Freshness::Fresh, Some((_, w))) => Some(*w),
            _ => None,
        }
    }
    pub fn age_us(&self, label: u8, sdi: u8, now_us: u64) -> Option<u64> {
        self.last
            .get(&(label, sdi))
            .map(|(t, _)| now_us.saturating_sub(*t))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn label_bit_reversed() {
        let w = Word::pack(0o310, 0, 0, 0);
        assert_eq!(w.label(), 0o310u8.reverse_bits());
        assert_eq!(w.label_octal(), 0o310);
        assert_eq!(Word::pack(0o001, 0, 0, 0).label(), 0x80);
    }

    #[test]
    fn known_reference_words() {
        let w = Word::pack(0o001, 0, 0, 0);
        assert_eq!(w.0, 0x80);
        assert!(w.parity_ok());
        let w2 = Word::pack(0o003, 0, 0, 0);
        assert_eq!(w2.0, 0xC0 | (1 << 31));
        assert!(w2.parity_ok());
    }

    #[test]
    fn field_positions() {
        let w = Word::pack(0o204, 2, 0x5A5A5, 3);
        assert_eq!(w.sdi(), 2);
        assert_eq!(w.data(), 0x5A5A5);
        assert_eq!(w.ssm_bits(), 3);
        assert_eq!((w.0 >> 8) & 3, 2);
        assert_eq!((w.0 >> 10) & DATA_MASK, 0x5A5A5);
        assert_eq!((w.0 >> 29) & 3, 3);
    }

    #[test]
    fn parity_odd_always() {
        for l in 0..=255u8 {
            for d in [0u32, 1, 0x7FFFF, 0x12345] {
                for s in 0..4u8 {
                    let w = Word::pack(l, s, d, s);
                    assert!(w.parity_ok());
                    assert!(!w.corrupt(17).parity_ok());
                }
            }
        }
    }

    #[test]
    fn ssm_meanings() {
        assert_eq!(BnrSsm::from_bits(0), BnrSsm::FailureWarning);
        assert_eq!(BnrSsm::from_bits(1), BnrSsm::NoComputedData);
        assert_eq!(BnrSsm::from_bits(2), BnrSsm::FunctionalTest);
        assert_eq!(BnrSsm::from_bits(3), BnrSsm::NormalOperation);
        assert_eq!(BcdSsm::from_bits(0), BcdSsm::Plus);
        assert_eq!(BcdSsm::from_bits(3), BcdSsm::Minus);
        assert_eq!(DiscreteSsm::from_bits(0), DiscreteSsm::NormalOperation);
        assert_eq!(DiscreteSsm::from_bits(3), DiscreteSsm::FailureWarning);
        for b in 0..4 {
            assert_eq!(BnrSsm::from_bits(b).bits(), b);
            assert_eq!(BcdSsm::from_bits(b).bits(), b);
            assert_eq!(DiscreteSsm::from_bits(b).bits(), b);
        }
    }

    #[test]
    fn bnr_round_trip() {
        let f = BnrFormat::new(15, 131_072.0);
        assert!((f.resolution() - 8.0).abs() < 1e-12);
        for v in [0.0, 8.0, -8.0, 35_000.0, -35_000.0, 131_000.0, -131_072.0] {
            let d = f.decode(f.encode(v));
            assert!((d - v).abs() <= f.resolution() / 2.0 + 1e-9, "{v} {d}");
        }
    }

    #[test]
    fn bnr_saturates_and_sign() {
        let f = BnrFormat::new(12, 1024.0);
        assert!((f.decode(f.encode(5000.0)) - (1024.0 - f.resolution())).abs() < 1e-9);
        assert!((f.decode(f.encode(-5000.0)) + 1024.0).abs() < 1e-9);
        let neg = f.encode(-1.0);
        assert_ne!(neg & (1 << 18), 0);
        let pos = f.encode(1.0);
        assert_eq!(pos & (1 << 18), 0);
        assert_eq!(f.encode(f64::NAN), 0);
    }

    #[test]
    fn bnr_word_ssm_gating() {
        let f = BnrFormat::new(18, 1000.0);
        let w = f.pack_word(0o203, 1, 123.4, BnrSsm::NormalOperation);
        assert!((f.read_word(w).unwrap() - 123.4).abs() < f.resolution());
        let w = f.pack_word(0o203, 1, 123.4, BnrSsm::NoComputedData);
        assert!(f.read_word(w).is_none());
        let w = f.pack_word(0o203, 1, 123.4, BnrSsm::NormalOperation).corrupt(12);
        assert!(f.read_word(w).is_none());
    }

    #[test]
    fn bcd_round_trip() {
        let f = BcdFormat::new(4, 0.1);
        let w = f.pack_word(0o013, 0, 123.4);
        assert_eq!(w.data() & 0xFFFF, 0x1234);
        assert!((f.read_word(w).unwrap() - 123.4).abs() < 1e-9);
        let w = f.pack_word(0o013, 0, -56.7);
        assert_eq!(w.bcd_ssm(), BcdSsm::Minus);
        assert!((f.read_word(w).unwrap() + 56.7).abs() < 1e-9);
    }

    #[test]
    fn bcd_limits() {
        let f = BcdFormat::new(5, 1.0);
        assert_eq!(f.max_counts(), 79_999);
        let w = f.pack_word(0o013, 0, 99_999.0);
        assert!((f.read_word(w).unwrap() - 79_999.0).abs() < 1e-9);
        let bad = Word::pack(0o013, 0, 0xA, 0);
        assert!(BcdFormat::new(4, 1.0).read_word(bad).is_none());
    }

    #[test]
    fn discrete_bits() {
        let w = Word::pack(0o270, 0, 0, 0);
        let w = w.with_discrete_bit(11, true).with_discrete_bit(29, true);
        assert!(w.discrete_bit(11));
        assert!(w.discrete_bit(29));
        assert!(!w.discrete_bit(12));
        assert_eq!(w.data(), 1 | (1 << 18));
        assert!(w.parity_ok());
        assert!(!w.with_discrete_bit(11, false).discrete_bit(11));
    }

    #[test]
    fn timing_numbers() {
        assert_eq!(Speed::Low.bit_time_us(), 80);
        assert_eq!(Speed::High.bit_time_us(), 10);
        assert_eq!(Speed::Low.word_time_us(), 2560);
        assert_eq!(Speed::Low.slot_time_us(), 2880);
        assert_eq!(Speed::High.word_time_us(), 320);
        assert_eq!(Speed::High.slot_time_us(), 360);
        assert_eq!(Speed::High.max_words_per_second(), 2777);
        assert_eq!(Speed::Low.max_words_per_second(), 347);
    }

    #[test]
    fn transmitter_schedule() {
        let mut t = Transmitter::new(Speed::Low);
        t.add_label(0o203, 0, 50_000, 0);
        t.add_label(0o204, 0, 50_000, 0);
        t.set_word(0o203, 0, Word::pack(0o203, 0, 1, 3));
        t.set_word(0o204, 0, Word::pack(0o204, 0, 2, 3));
        let out = t.poll(0);
        assert_eq!(out.len(), 1);
        let out2 = t.poll(2880);
        assert_eq!(out2.len(), 1);
        assert_eq!(out2[0].0, 2880);
        assert_eq!(out2[0].1, 2880 + 2560);
        assert!(t.poll(10_000).is_empty());
        let out3 = t.poll(50_000);
        assert_eq!(out3.len(), 1);
        assert_eq!(out3[0].0, 50_000);
        assert!(t.utilisation() > 0.1);
    }

    #[test]
    fn transmitter_skips_without_data() {
        let mut t = Transmitter::new(Speed::High);
        t.add_label(0o100, 0, 20_000, 0);
        assert!(t.poll(100_000).is_empty());
        t.set_word(0o100, 0, Word::pack(0o100, 0, 5, 3));
        let o = t.poll(100_000);
        assert_eq!(o.len(), 1);
    }

    #[test]
    fn receiver_freshness() {
        let mut r = WordReceiver::new(100_000);
        assert_eq!(r.freshness(0o203, 0, 0), Freshness::Never);
        let w = Word::pack(0o203, 0, 9, 3);
        assert!(r.receive(1_000, w));
        assert_eq!(r.freshness(0o203, 0, 101_000), Freshness::Fresh);
        assert_eq!(r.freshness(0o203, 0, 101_001), Freshness::Stale);
        assert_eq!(r.get(0o203, 0, 50_000), Some(w));
        assert_eq!(r.get(0o203, 0, 500_000), None);
        assert_eq!(r.age_us(0o203, 0, 11_000), Some(10_000));
        assert!(!r.receive(2_000, w.corrupt(3)));
        assert_eq!(r.parity_errors(), 1);
        assert_eq!(r.freshness(0o203, 1, 2_000), Freshness::Never);
    }
}
