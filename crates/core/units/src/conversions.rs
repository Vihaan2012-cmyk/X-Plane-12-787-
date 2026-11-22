use crate::quantities::*;

pub const ZERO_CELSIUS_KELVIN: f64 = 273.15;

convert!(Hertz => RadiansPerSecond, core::f64::consts::TAU);
convert!(RadiansPerSecond => Hertz, 1.0 / core::f64::consts::TAU);
convert!(Rpm => Hertz, 1.0 / 60.0);
convert!(Hertz => Rpm, 60.0);
convert!(Rpm => RadiansPerSecond, core::f64::consts::TAU / 60.0);
convert!(RadiansPerSecond => Rpm, 60.0 / core::f64::consts::TAU);
convert!(AmpHours => Coulombs, 3600.0);
convert!(Coulombs => AmpHours, 1.0 / 3600.0);

impl Volts {
    pub fn from_millivolts(v: f64) -> Self {
        Self(v * 1e-3)
    }
    pub fn from_kilovolts(v: f64) -> Self {
        Self(v * 1e3)
    }
    pub fn millivolts(self) -> f64 {
        self.0 * 1e3
    }
    pub fn kilovolts(self) -> f64 {
        self.0 * 1e-3
    }
    pub fn rms_to_peak(self) -> Self {
        Self(self.0 * core::f64::consts::SQRT_2)
    }
    pub fn peak_to_rms(self) -> Self {
        Self(self.0 * core::f64::consts::FRAC_1_SQRT_2)
    }
    pub fn line_to_line_from_phase(self) -> Self {
        Self(self.0 * 3.0_f64.sqrt())
    }
    pub fn phase_from_line_to_line(self) -> Self {
        Self(self.0 / 3.0_f64.sqrt())
    }
}

impl Amps {
    pub fn from_milliamps(v: f64) -> Self {
        Self(v * 1e-3)
    }
    pub fn milliamps(self) -> f64 {
        self.0 * 1e3
    }
}

impl Ohms {
    pub fn from_milliohms(v: f64) -> Self {
        Self(v * 1e-3)
    }
    pub fn from_kilohms(v: f64) -> Self {
        Self(v * 1e3)
    }
    pub fn milliohms(self) -> f64 {
        self.0 * 1e3
    }
    pub fn recip(self) -> Siemens {
        Siemens(1.0 / self.0)
    }
}

impl Siemens {
    pub fn recip(self) -> Ohms {
        Ohms(1.0 / self.0)
    }
}

impl Watts {
    pub fn from_kilowatts(v: f64) -> Self {
        Self(v * 1e3)
    }
    pub fn from_megawatts(v: f64) -> Self {
        Self(v * 1e6)
    }
    pub fn kilowatts(self) -> f64 {
        self.0 * 1e-3
    }
    pub fn megawatts(self) -> f64 {
        self.0 * 1e-6
    }
    pub fn horsepower(self) -> f64 {
        self.0 / 745.699_871_582_270_2
    }
}

impl VoltAmps {
    pub fn from_kva(v: f64) -> Self {
        Self(v * 1e3)
    }
    pub fn kva(self) -> f64 {
        self.0 * 1e-3
    }
}

impl Hertz {
    pub fn period(self) -> Seconds {
        Seconds(1.0 / self.0)
    }
    pub fn to_radians_per_second(self) -> RadiansPerSecond {
        self.into()
    }
    pub fn to_rpm(self) -> Rpm {
        self.into()
    }
}

impl RadiansPerSecond {
    pub fn to_hertz(self) -> Hertz {
        self.into()
    }
    pub fn to_rpm(self) -> Rpm {
        self.into()
    }
    pub fn degrees_per_second(self) -> f64 {
        self.0.to_degrees()
    }
    pub fn from_degrees_per_second(v: f64) -> Self {
        Self(v.to_radians())
    }
}

impl Rpm {
    pub fn to_hertz(self) -> Hertz {
        self.into()
    }
    pub fn to_radians_per_second(self) -> RadiansPerSecond {
        self.into()
    }
}

impl Seconds {
    pub fn from_millis(v: f64) -> Self {
        Self(v * 1e-3)
    }
    pub fn from_micros(v: f64) -> Self {
        Self(v * 1e-6)
    }
    pub fn from_minutes(v: f64) -> Self {
        Self(v * 60.0)
    }
    pub fn from_hours(v: f64) -> Self {
        Self(v * 3600.0)
    }
    pub fn millis(self) -> f64 {
        self.0 * 1e3
    }
    pub fn micros(self) -> f64 {
        self.0 * 1e6
    }
    pub fn minutes(self) -> f64 {
        self.0 / 60.0
    }
    pub fn hours(self) -> f64 {
        self.0 / 3600.0
    }
    pub fn frequency(self) -> Hertz {
        Hertz(1.0 / self.0)
    }
}

impl Kelvin {
    pub fn from_celsius(c: f64) -> Self {
        Self(c + ZERO_CELSIUS_KELVIN)
    }
    pub fn to_celsius(self) -> f64 {
        self.0 - ZERO_CELSIUS_KELVIN
    }
    pub fn from_fahrenheit(f: f64) -> Self {
        Self::from_celsius((f - 32.0) * 5.0 / 9.0)
    }
    pub fn to_fahrenheit(self) -> f64 {
        self.to_celsius() * 9.0 / 5.0 + 32.0
    }
}

impl Joules {
    pub fn from_kilojoules(v: f64) -> Self {
        Self(v * 1e3)
    }
    pub fn from_watt_hours(v: f64) -> Self {
        Self(v * 3600.0)
    }
    pub fn from_kilowatt_hours(v: f64) -> Self {
        Self(v * 3.6e6)
    }
    pub fn kilojoules(self) -> f64 {
        self.0 * 1e-3
    }
    pub fn watt_hours(self) -> f64 {
        self.0 / 3600.0
    }
    pub fn kilowatt_hours(self) -> f64 {
        self.0 / 3.6e6
    }
}

impl Coulombs {
    pub fn to_amp_hours(self) -> AmpHours {
        self.into()
    }
}

impl AmpHours {
    pub fn to_coulombs(self) -> Coulombs {
        self.into()
    }
    pub fn from_milliamp_hours(v: f64) -> Self {
        Self(v * 1e-3)
    }
    pub fn milliamp_hours(self) -> f64 {
        self.0 * 1e3
    }
}

impl Farads {
    pub fn from_microfarads(v: f64) -> Self {
        Self(v * 1e-6)
    }
    pub fn microfarads(self) -> f64 {
        self.0 * 1e6
    }
}

impl Henries {
    pub fn from_millihenries(v: f64) -> Self {
        Self(v * 1e-3)
    }
    pub fn millihenries(self) -> f64 {
        self.0 * 1e3
    }
}

impl Kilograms {
    pub const POUND_KG: f64 = 0.453_592_37;
    pub fn from_pounds(v: f64) -> Self {
        Self(v * Self::POUND_KG)
    }
    pub fn pounds(self) -> f64 {
        self.0 / Self::POUND_KG
    }
}

impl KilogramsPerSecond {
    pub fn from_kg_per_hour(v: f64) -> Self {
        Self(v / 3600.0)
    }
    pub fn kg_per_hour(self) -> f64 {
        self.0 * 3600.0
    }
}

impl Pascals {
    pub const PSI_PA: f64 = 6_894.757_293_168;
    pub const INHG_PA: f64 = 3386.389;
    pub fn from_kilopascals(v: f64) -> Self {
        Self(v * 1e3)
    }
    pub fn from_hectopascals(v: f64) -> Self {
        Self(v * 100.0)
    }
    pub fn from_psi(v: f64) -> Self {
        Self(v * Self::PSI_PA)
    }
    pub fn from_inhg(v: f64) -> Self {
        Self(v * Self::INHG_PA)
    }
    pub fn kilopascals(self) -> f64 {
        self.0 * 1e-3
    }
    pub fn hectopascals(self) -> f64 {
        self.0 * 0.01
    }
    pub fn psi(self) -> f64 {
        self.0 / Self::PSI_PA
    }
    pub fn inhg(self) -> f64 {
        self.0 / Self::INHG_PA
    }
}

impl Meters {
    pub const FOOT_M: f64 = 0.3048;
    pub const NAUTICAL_MILE_M: f64 = 1852.0;
    pub fn from_feet(v: f64) -> Self {
        Self(v * Self::FOOT_M)
    }
    pub fn from_kilometers(v: f64) -> Self {
        Self(v * 1e3)
    }
    pub fn from_nautical_miles(v: f64) -> Self {
        Self(v * Self::NAUTICAL_MILE_M)
    }
    pub fn from_millimeters(v: f64) -> Self {
        Self(v * 1e-3)
    }
    pub fn feet(self) -> f64 {
        self.0 / Self::FOOT_M
    }
    pub fn kilometers(self) -> f64 {
        self.0 * 1e-3
    }
    pub fn nautical_miles(self) -> f64 {
        self.0 / Self::NAUTICAL_MILE_M
    }
    pub fn millimeters(self) -> f64 {
        self.0 * 1e3
    }
}

impl CubicMeters {
    pub fn from_liters(v: f64) -> Self {
        Self(v * 1e-3)
    }
    pub fn liters(self) -> f64 {
        self.0 * 1e3
    }
}

impl Ratio {
    pub fn from_percent(v: f64) -> Self {
        Self(v * 0.01)
    }
    pub fn percent(self) -> f64 {
        self.0 * 100.0
    }
    pub fn complement(self) -> Self {
        Self(1.0 - self.0)
    }
    pub fn clamp01(self) -> Self {
        Self(self.0.clamp(0.0, 1.0))
    }
}

pub fn apparent_power(p: Watts, q: Vars) -> VoltAmps {
    VoltAmps(p.0.hypot(q.0))
}

pub fn power_factor(p: Watts, s: VoltAmps) -> Ratio {
    if s.0 == 0.0 { Ratio(1.0) } else { Ratio(p.0 / s.0) }
}

pub fn watts_from_apparent(s: VoltAmps, pf: Ratio) -> Watts {
    Watts(s.0 * pf.0)
}

pub fn reactive_power_from_pf(s: VoltAmps, pf: Ratio) -> Vars {
    Vars(s.0 * (1.0 - pf.0 * pf.0).max(0.0).sqrt())
}
