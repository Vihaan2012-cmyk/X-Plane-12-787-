use crate::codec::{Decoder, Encoder};
use crate::rng::tolerance_factor;
use std::collections::BTreeMap;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct PartId(pub u32);

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Grade {
    Verified,
    Derived,
    Standard,
    Approx,
}

impl Grade {
    pub fn as_str(self) -> &'static str {
        match self {
            Grade::Verified => "VERIFIED",
            Grade::Derived => "DERIVED",
            Grade::Standard => "STANDARD",
            Grade::Approx => "APPROX",
        }
    }

    pub fn parse(s: &str) -> Option<Grade> {
        match s {
            "VERIFIED" => Some(Grade::Verified),
            "DERIVED" => Some(Grade::Derived),
            "STANDARD" => Some(Grade::Standard),
            "APPROX" => Some(Grade::Approx),
            _ => None,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Source(pub String);

impl Source {
    pub fn new(s: &str) -> Self {
        Source(s.to_string())
    }

    pub fn is_empty(&self) -> bool {
        self.0.trim().is_empty()
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Location {
    pub station_in: f64,
    pub buttline_in: f64,
    pub waterline_in: f64,
}

impl Location {
    pub fn new(station_in: f64, buttline_in: f64, waterline_in: f64) -> Self {
        Self {
            station_in,
            buttline_in,
            waterline_in,
        }
    }

    pub fn distance_in(&self, other: &Location) -> f64 {
        let dx = self.station_in - other.station_in;
        let dy = self.buttline_in - other.buttline_in;
        let dz = self.waterline_in - other.waterline_in;
        (dx * dx + dy * dy + dz * dz).sqrt()
    }
}

pub const AMBIENT_K: f64 = 288.15;

#[derive(Clone, Debug, PartialEq)]
pub struct PartCommon {
    pub name: String,
    pub kind: String,
    pub zone: String,
    pub location: Option<Location>,
    pub temperature_k: f64,
    pub wear: f64,
    pub tolerances: BTreeMap<String, f64>,
    pub serial: String,
    pub installed_at_us: u64,
    pub hours: f64,
    pub cycles: u64,
    pub grade: Grade,
    pub source: Source,
}

impl PartCommon {
    pub fn new(name: &str, kind: &str, zone: &str) -> Self {
        Self {
            name: name.to_string(),
            kind: kind.to_string(),
            zone: zone.to_string(),
            location: None,
            temperature_k: AMBIENT_K,
            wear: 0.0,
            tolerances: BTreeMap::new(),
            serial: String::new(),
            installed_at_us: 0,
            hours: 0.0,
            cycles: 0,
            grade: Grade::Approx,
            source: Source::new(""),
        }
    }

    pub fn with_location(mut self, loc: Location) -> Self {
        self.location = Some(loc);
        self
    }

    pub fn with_serial(mut self, serial: &str) -> Self {
        self.serial = serial.to_string();
        self
    }

    pub fn with_grade(mut self, grade: Grade, source: &str) -> Self {
        self.grade = grade;
        self.source = Source::new(source);
        self
    }

    pub fn with_install(mut self, at_us: u64, hours: f64, cycles: u64) -> Self {
        self.installed_at_us = at_us;
        self.hours = hours;
        self.cycles = cycles;
        self
    }

    pub fn draw_tolerance(&mut self, name: &str, sigma: f64, clip_sigmas: f64) -> f64 {
        let f = tolerance_factor(&self.serial, name, sigma, clip_sigmas);
        self.tolerances.insert(name.to_string(), f);
        f
    }

    pub fn tolerance(&self, name: &str) -> f64 {
        self.tolerances.get(name).copied().unwrap_or(1.0)
    }

    pub fn set_wear(&mut self, wear: f64) {
        self.wear = wear.clamp(0.0, 1.0);
    }

    pub fn add_wear(&mut self, delta: f64) {
        self.set_wear(self.wear + delta);
    }

    pub fn accrue(&mut self, hours: f64, cycles: u64) {
        self.hours += hours;
        self.cycles += cycles;
    }

    pub fn age_us(&self, now_us: u64) -> u64 {
        now_us.saturating_sub(self.installed_at_us)
    }

    pub fn has_source(&self) -> bool {
        !self.source.is_empty()
    }

    pub fn encode(&self, e: &mut Encoder) {
        e.str(&self.name).str(&self.kind).str(&self.zone);
        match self.location {
            Some(l) => {
                e.bool(true)
                    .f64(l.station_in)
                    .f64(l.buttline_in)
                    .f64(l.waterline_in);
            }
            None => {
                e.bool(false);
            }
        }
        e.f64(self.temperature_k).f64(self.wear);
        e.u64(self.tolerances.len() as u64);
        for (k, v) in &self.tolerances {
            e.str(k).f64(*v);
        }
        e.str(&self.serial)
            .u64(self.installed_at_us)
            .f64(self.hours)
            .u64(self.cycles)
            .str(self.grade.as_str())
            .str(&self.source.0);
    }

    pub fn decode(d: &mut Decoder<'_>) -> Option<Self> {
        let name = d.str()?;
        let kind = d.str()?;
        let zone = d.str()?;
        let location = if d.bool()? {
            Some(Location::new(d.f64()?, d.f64()?, d.f64()?))
        } else {
            None
        };
        let temperature_k = d.f64()?;
        let wear = d.f64()?;
        let n = d.u64()?;
        let mut tolerances = BTreeMap::new();
        for _ in 0..n {
            let k = d.str()?;
            let v = d.f64()?;
            tolerances.insert(k, v);
        }
        let serial = d.str()?;
        let installed_at_us = d.u64()?;
        let hours = d.f64()?;
        let cycles = d.u64()?;
        let grade = Grade::parse(&d.str()?)?;
        let source = Source(d.str()?);
        Some(Self {
            name,
            kind,
            zone,
            location,
            temperature_k,
            wear,
            tolerances,
            serial,
            installed_at_us,
            hours,
            cycles,
            grade,
            source,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample() -> PartCommon {
        let mut p = PartCommon::new("L MAIN BAT", "battery", "119")
            .with_location(Location::new(400.5, -30.0, 120.25))
            .with_serial("SN\t42")
            .with_grade(Grade::Verified, "eps24.txt:27610")
            .with_install(5_000_000, 12.5, 3);
        p.draw_tolerance("capacity", 0.03, 3.0);
        p.draw_tolerance("resistance", 0.05, 3.0);
        p.set_wear(0.25);
        p.temperature_k = 300.5;
        p
    }

    #[test]
    fn defaults() {
        let p = PartCommon::new("a", "k", "z");
        assert_eq!(p.temperature_k, AMBIENT_K);
        assert_eq!(p.wear, 0.0);
        assert_eq!(p.grade, Grade::Approx);
        assert!(!p.has_source());
        assert_eq!(p.tolerance("none"), 1.0);
    }

    #[test]
    fn grade_round_trip() {
        for g in [Grade::Verified, Grade::Derived, Grade::Standard, Grade::Approx] {
            assert_eq!(Grade::parse(g.as_str()), Some(g));
        }
        assert_eq!(Grade::parse("x"), None);
    }

    #[test]
    fn tolerance_deterministic_per_serial() {
        let mut a = PartCommon::new("a", "k", "z").with_serial("S1");
        let mut b = PartCommon::new("b", "k", "z").with_serial("S1");
        let mut c = PartCommon::new("c", "k", "z").with_serial("S2");
        let fa = a.draw_tolerance("g", 0.05, 3.0);
        let fb = b.draw_tolerance("g", 0.05, 3.0);
        let fc = c.draw_tolerance("g", 0.05, 3.0);
        assert_eq!(fa.to_bits(), fb.to_bits());
        assert_ne!(fa.to_bits(), fc.to_bits());
        assert_eq!(a.tolerance("g"), fa);
    }

    #[test]
    fn wear_clamped() {
        let mut p = PartCommon::new("a", "k", "z");
        p.add_wear(2.0);
        assert_eq!(p.wear, 1.0);
        p.add_wear(-5.0);
        assert_eq!(p.wear, 0.0);
    }

    #[test]
    fn accrue_and_age() {
        let mut p = PartCommon::new("a", "k", "z").with_install(100, 0.0, 0);
        p.accrue(1.5, 2);
        assert_eq!(p.hours, 1.5);
        assert_eq!(p.cycles, 2);
        assert_eq!(p.age_us(150), 50);
        assert_eq!(p.age_us(50), 0);
    }

    #[test]
    fn location_distance() {
        let a = Location::new(0.0, 0.0, 0.0);
        let b = Location::new(3.0, 4.0, 0.0);
        assert_eq!(a.distance_in(&b), 5.0);
    }

    #[test]
    fn encode_decode_round_trip() {
        let p = sample();
        let mut e = Encoder::new();
        p.encode(&mut e);
        let text = e.finish();
        let mut d = Decoder::new(&text);
        let q = PartCommon::decode(&mut d).unwrap();
        assert_eq!(p, q);
    }

    #[test]
    fn encode_decode_no_location() {
        let p = PartCommon::new("x", "y", "z");
        let mut e = Encoder::new();
        p.encode(&mut e);
        let text = e.finish();
        let q = PartCommon::decode(&mut Decoder::new(&text)).unwrap();
        assert_eq!(p, q);
    }

    #[test]
    fn decode_truncated_fails() {
        let p = sample();
        let mut e = Encoder::new();
        p.encode(&mut e);
        let text = e.finish();
        let cut = &text[..text.len() / 2];
        assert!(PartCommon::decode(&mut Decoder::new(cut)).is_none());
    }
}
